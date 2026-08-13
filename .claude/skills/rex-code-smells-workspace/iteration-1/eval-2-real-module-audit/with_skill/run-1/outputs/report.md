# Smell audit: src/adapters/backoff.rs, src/adapters/chunk.rs, src/adapters/rest_quiet.rs

All three files read whole. Every finding below was confirmed by reading the call sites, not from grep
output alone — the two most interesting candidates changed severity once the callers were read (one was
already guarded on the Binance side; one turned out to be a false claim in a module header rather than a
local bug).

| # | Where | Lens | Smell | Sev | Evidence | Fix direction |
|---|-------|------|-------|-----|----------|---------------|
| 1 | `rest_quiet.rs:20-23` | brittleness | external input → unbounded stall, then panic | BLOCK | `retry_after_secs` is a raw header parse (`polymarket/rest.rs:224-232`, `binance/rest/mod.rs:402-407`), `.max(FLOOR)` puts a floor on it and nothing puts a ceiling; `now + Duration::from_secs(secs)` panics on `Instant` overflow | clamp to a `QUIET_CEILING_SECS`; return `Duration`, not bare `u64` |
| 2 | `chunk.rs:49-59` | brittleness | rule encoded in one caller, missing in the other | BLOCK | both sides empty → `emit_side` early-returns twice → zero chunks → no `is_last_chunk`. Binance guards it at `binance/depth.rs:151-155`; `polymarket/book.rs:35` (called `leg.rs:75`, `leg.rs:85`) does not | hoist the empty check into `emit_book`; return an outcome instead of silently emitting nothing |
| 3 | `rest_quiet.rs:1-3` | brittleness | header documents sharing that does not exist | BLOCK | "Shared … used by every venue adapter … the group holds off" — but three private instances (`binance/actor/mod.rs:199`, `binance/rest/signed.rs:65`, `polymarket/actor/mod.rs:131`), no `Arc`, no handle passed. `binance/actor/mod.rs:386` repeats the claim | share one instance across the Binance clients, or correct both comments to "per-client" |
| 4 | `rest_quiet.rs:20-23` | brittleness | ordering dependency not enforced by the type | REFACTOR | `open` is last-write-wins on a monotone deadline, so it can only ever *shorten* a live cool-off; safe today only because five call sites each remember to gate first (`kline_backfill.rs:55,92`, `depth_resync.rs:118`, `effects.rs:128,152`) | `self.until = self.until.max(Some(now + d))` — make the deadline monotone in the type |
| 5 | `backoff.rs:21` | naming | undocumented parameter base | REFACTOR | four call sites pass a raw attempt count, `signed.rs:233` passes `attempt - 1`; unguarded `u32` subtraction safe only because `signed.rs:208` increments first | document the base or take an `Attempt` newtype that cannot underflow |
| 6 | `backoff.rs:30-33` | brittleness | "full jitter" that is not an independent draw | REFACTOR | entropy is the monotonic wall clock, so delay is a deterministic sawtooth of call time; also `as_nanos()` fed into `from_micros` | per-connection PRNG state (xorshift on a stored `u64`), seeded once from the clock |
| 7 | `backoff.rs:21` | naming | stutter + free fn over its own type | REFACTOR | `backoff::backoff_delay(caps, attempt)` takes `&BackoffCaps` as first param; §9 puts behaviour on the type | `caps.delay(attempt)` |
| 8 | `backoff.rs:30-32` | brittleness | fallback removes the protection | NIT | `map_or(0, …)` on a pre-epoch clock ⇒ entropy 0 ⇒ `Duration::ZERO` ⇒ reconnect loop spins with no backoff | fall back to the ceiling, not zero |
| 9 | `backoff.rs:24` | naming | unnamed magic carrying a safety contract | NIT | `1u32 << attempt.min(16)` — the `16` is what keeps the shift from overflowing | named const with the overflow reason |
| 10 | `chunk.rs:44` | brittleness | mutate-and-return with no `#[must_use]` | NIT | `clamp_emit_ts` updates the floor *and* returns the value the caller must write into `ChunkPlan`; all four sites use it today (`depth.rs:135,179,236`, `polymarket/book.rs:88`) | `#[must_use]`, so dropping it cannot compile |
| 11 | `chunk.rs:89` | naming | two §5 stamps silently interchanged | NIT | `queued_ts_us: plan.received_ts_us` is correct only because a distant `set_queued_ts_us` always follows (`binance/actor/mod.rs:358`, `polymarket/actor/effects.rs:77`, `exchange_sim/actor/mod.rs:185,244`) | leave it unset in the type, or name the placeholder |
| 12 | `chunk.rs:61-68` | naming | 5 params, one a bare bool | NIT | reads as `emit_side(plan, Side::Sell, asks, true, emit)` at line 58 | fold `side_is_last` into an enum or into `ChunkPlan` |
| 13 | `rest_quiet.rs:10` | brittleness | doc advertises a test that cannot exist | NIT | `/// Deterministic in now (fitness-testable)` — nothing under `tests/` references `RestQuiet`, and `adapters/mod.rs:12` declares the module `pub(crate)`, so a `tests/` target cannot reach it (§12) | drop the parenthetical, or promote and write the pin |
| 14 | `rest_quiet.rs:26-34` | duplication | same predicate encoded twice | NIT | `is_active` and `remaining` both spell `now < until` | `is_active` = `self.remaining(now).is_some()` |

**Clean:** no `.unwrap()`/`.expect()` anywhere in the three files; no float in state logic (§4 clean — `chunk.rs` carries `Price`/`Qty` newtypes throughout); no allocation on the emit path (`chunk.rs:74` fills a stack array, §3 clean); no `get_` prefixes; no dead items (all three modules have live callers); no wall clock deciding hot-path state (`backoff.rs` and `rest_quiet.rs` are edge-only, and `chunk.rs` reads no clock at all — stamps arrive on the message, §2 clean); `len as u8` at `chunk.rs:84` is safe today and debug-asserted downstream at `msg/inbound.rs:107`, with `BOOK_CHUNK_LEVELS = 20`.

**Not checked:** `cargo clippy` was not run. Other agents are working in this checkout, so a red gate could not be attributed to these files — findings above are all from reading. The `_ts_us`/`_secs` naming conventions were taken as given rather than re-litigated.

---

## The three BLOCKs, in more detail

**#1 — a floor with no ceiling.** `RestQuiet::open` guarantees the cool-off is at least 2 s and says nothing
about the maximum. `retry_after_secs` reaches it straight from an HTTP header on both venues, parsed with
`.parse().ok()` into `Option<u64>` and never bounded. A plausible real value (`Retry-After: 3600` after an
IP ban) stalls that REST client for an hour with nothing but a `warn!`; the extreme value `u64::MAX` parses
fine and then panics inside `now + Duration::from_secs(secs)`, because `Instant`'s `Add` panics rather than
saturating. §6 puts external, expected input under typed policy and reserves panics for internal invariant
violations, so both ends of that range are wrong. `binance/rest/signed.rs:230` then sleeps the returned
`secs` directly, so the same unbounded value drives a second wait.

**#2 — the terminator that is never emitted.** `emit_book` emits a chunk per side and marks the last one
`is_last_chunk`. When both sides are empty, `emit_side` early-returns for each and *nothing at all* is
emitted — no terminator, no chunk. That flag is load-bearing: `hot/book.rs:246-251` makes
`is_last_chunk` the only transition from `AwaitingSnapshot` to `Valid`, and `apply_delta_chunk`
(`hot/book.rs:255-259`) drops every delta while the book is still `AwaitingSnapshot`. So an empty snapshot
leaves the book wedged while the adapter believes it went live.

The Binance sequencer already knows this: `binance/depth.rs:151-155` returns `SnapshotOutcome::Stale` for a
both-sides-empty snapshot, with the comment *"would flip live with empty book, drop deltas"*. The
Polymarket producer has no equivalent — `polymarket/book.rs:35` passes `book.bids`/`book.asks` through
unchecked. The worst path is `polymarket/actor/leg.rs:79-85`: on shadow divergence it emits a `BookReset`
(pushing the hot book to `AwaitingSnapshot`) and then immediately calls `emit_snapshot`; if that book is
empty, the reset lands and the recovery snapshot does not, and the instrument stays wedged. Empty and
one-sided Polymarket books are a modelled state in this codebase — `polymarket/shadow.rs:57-61` has
`has_both_sides`/`is_one_sided` — and the venue returns `{"error":"No orderbook exists…"}` at teardown
(`tests/fixtures/polymarket/clob_book_teardown.json`), so this is not a theoretical input.

This is the shape the `chunk.rs` module header explicitly claims to prevent: *"where a side splits and
which chunk closes it are decisions the hot book depends on — they must not be taken twice."* The decision
is currently taken in the callers, and one of the two callers does not take it. The fitness suite asserts
the invariant (`tests/fitness/binance_depth_sequencing.rs:72`, "exactly one is_last_chunk per event") but
only over non-empty recorded fixtures, so the hole is green.

**#3 — "shared" is not shared.** The module header sells `RestQuiet` as a group cool-off — *"used by every
venue adapter … the group holds off … so drivers back off together instead of hammering into a harder
ban."* There is no sharing mechanism: three call sites each construct their own private instance and no
handle is ever passed between them. Within one actor the claim holds (the Binance market-data actor's
`rest_quiet` does gate both `kline_backfill` and `depth_resync`), but across adapters it is false, and the
two Binance instances spend the *same* per-IP weight budget — the exec `SignedRestClient` keeps issuing
signed requests while the market-data actor sits in cool-off. `binance/actor/mod.rs:386` repeats the claim
("429/418 opens shared quiet window -> all REST back off together"). The code is locally correct; what
makes this a BLOCK is that the next person wiring an adapter will read the header and assume the cool-off
is global.

**14 findings: 3 BLOCK / 4 REFACTOR / 7 NIT**
