# Audit: `backoff.rs`, `chunk.rs`, `rest_quiet.rs`

Scope: the three named files, read against their call sites and the fitness suite. Read-only — no repo files were modified.

Headline: all three are small and clean-reading, but each holds an invariant it does not enforce and leaves the enforcement to its callers. None of the three has a single fitness test. The two most consequential findings are that `backoff_delay`'s jitter does not decorrelate *different* actors (the case it exists for), and that a remote-controlled `Retry-After` header reaches `RestQuiet::open` with no upper clamp, on a path that includes order placement and cancel.

---

## High

### 1. `backoff_delay`'s jitter decorrelates successive attempts, not concurrent actors

`src/adapters/backoff.rs:30-33`

```rust
let entropy = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_nanos() as u64);
Duration::from_micros(entropy % (ceiling_us + 1))
```

The "randomness" is nanoseconds-since-epoch — a value every caller in the process reads from the same clock. Two adapters that lose their connection to the same venue at the same moment (a network blip, a venue restart, a TLS reset — precisely the correlated failure that full jitter exists to spread out) call this within microseconds of each other. Their `entropy` values differ by ~10^3–10^4 nanoseconds, so `entropy % 30_000_001` differs by the same few thousand: their delays land within **microseconds of each other inside a 30-second window**, and they do so again on every subsequent round. The herd is not thinned, it is merely displaced.

Callers reconnecting off this shared clock: `src/adapters/binance/actor/mod.rs:228` and `:295`, `src/adapters/binance/exec/actor/mod.rs:261`, `src/adapters/binance/exec/actor/resync.rs:149`, `src/adapters/polymarket/actor/mod.rs:168`, `:175`, `:213`, and `src/adapters/binance/rest/signed.rs:233`.

The module header (`backoff.rs:1`) advertises "Exponential reconnect backoff, full jitter" and "(no rand)". The exponential ceiling is correct; the jitter term is what does not hold up. Mixing a per-caller discriminator into the entropy (an actor/source id, or a counter that advances per call) would restore decorrelation without adding a dependency.

### 2. `Retry-After` is remote-controlled, unbounded, and reaches a sleep on the order path

`src/adapters/rest_quiet.rs:21-22`, `src/adapters/binance/rest/signed.rs:228-230`

```rust
let secs = retry_after_secs.unwrap_or(0).max(QUIET_FLOOR_SECS);
self.until = Some(now + Duration::from_secs(secs));
```

`retry_after_secs` is parsed straight from the response header with no ceiling on either venue — `src/adapters/binance/rest/mod.rs:406` (`headers.get(name)?.to_str().ok()?.parse().ok()`) and `src/adapters/polymarket/rest.rs:230`. There is a *floor* (`QUIET_FLOOR_SECS`) and no *cap*.

Two consequences, in order of likelihood:

- A bogus or hostile `Retry-After: 86400` parks the client for a day. On the signed path this is not just a quiet window — `signed.rs:230` does `tokio::time::sleep(Duration::from_secs(secs)).await` *inside the retry loop*, so the header value directly suspends a task that carries order placement and cancellation (`send_signed` is reached from `CancelOrder`, `signed.rs:165-170`). The only signal is one WARN line.
- At the extreme tail, `now + Duration::from_secs(u64::MAX)` overflows; `impl Add<Duration> for Instant` panics rather than saturating. `18446744073709551615` parses cleanly as `u64`, so the input that triggers it is reachable from the wire.

§6 classifies remote input as "external, expected — handled by policy". An unbounded remote value that suspends the execution path is policy delegated to the venue. A `QUIET_CEILING_SECS` clamp beside the existing floor closes both cases in one line.

### 3. `open()` replaces the window, so a later 429 can *shorten* an active cool-off

`src/adapters/rest_quiet.rs:20-24`

`self.until = Some(now + ...)` is an unconditional overwrite. If a 418 ban opens a 60-second window and a second rate-limited response arrives 2 seconds later carrying no `Retry-After` header, the floor applies and the window is rewritten to **2 seconds** — the cool-off is discarded at exactly the moment the venue is signalling harder. Both venues can hit this: Binance re-enters through `src/adapters/binance/actor/mod.rs:392` and `signed.rs:228`, Polymarket through `src/adapters/polymarket/actor/mod.rs:342` and `:356`, and none of the four checks the existing state first.

Given the module's stated purpose — "instead of hammering into a harder ban" (`rest_quiet.rs:2-3`) — the intended semantics is almost certainly extend-only: `self.until = max(self.until, now + secs)`.

### 4. The doc header promises a group cool-off; the code gives one window per owner

`src/adapters/rest_quiet.rs:1-3` describes a "**Shared** REST cool-off, used by every venue adapter" where "**the group** holds off ... so drivers back off **together**".

There is no sharing. Each owner constructs its own instance: `src/adapters/binance/actor/mod.rs:199`, `src/adapters/binance/rest/signed.rs:65`, and `src/adapters/polymarket/actor/mod.rs:131`. A single Binance venue therefore runs **two independent quiet windows** — the market-data actor's and the signed-REST client's — while Binance's rate limit is shared per-IP and per-key. A 429 earned by kline backfill does not quiet order placement, and vice versa; each learns about the limit only by hitting it itself.

This is the most misleading item in the three files: a reader who trusts the header will believe coordination exists and will not add it. Either the sharing is real (an `Arc`/handle threaded to both owners) or the header should describe a per-driver window.

### 5. A both-sides-empty update emits no chunks at all, so no terminator ever arrives

`src/adapters/chunk.rs:56-59`, `:69-71`

```rust
let bid_side_is_last = asks.is_empty();
self.emit_side(plan, Side::Buy, bids, bid_side_is_last, emit);
self.emit_side(plan, Side::Sell, asks, true, emit);
```

with `emit_side` returning early on `levels.is_empty()`. One-sided updates are handled correctly. When **both** sides are empty, `emit_book` emits zero chunks — so nothing carries `is_last_chunk = true`.

That flag is load-bearing downstream: `src/hot/book.rs:248` promotes the book to `BookState::Valid` only on the last chunk of a snapshot, `src/hot/book.rs:262` gates the crossed-book check, `src/hot/dispatch/market.rs:73` gates the hot publish, and `src/adapters/exchange_sim/core/market.rs:254` gates `snapshot_complete`.

The asymmetry is the brittleness. Binance defends the case *in the caller*, with a comment explaining why — `src/adapters/binance/depth.rs:151-157`:

```rust
// Both-sides-empty -> Stale (would flip live with empty book, drop deltas). One-sided OK.
if snapshot.bids.is_empty() && snapshot.asks.is_empty() {
    return SnapshotOutcome::Stale;
}
```

Polymarket does not. `BookNormaliser::emit_price_change` (`src/adapters/polymarket/book.rs:48-64`) sorts deltas into two `Vec`s and forwards them unchecked; an empty `changes` slice, or the FSM handing over a no-op update, produces a silent no-emit. The file's own header (`chunk.rs:1-4`) states that where a side splits and which chunk closes it "must not be taken twice" — but this particular decision is exactly what each caller is left to take for itself, and the two callers took it differently. The guard belongs in `emit_book`.

### 6. None of the three modules has a fitness test, and the book oracle reimplements the cutter

A grep across `tests/` for `backoff`, `RestQuiet`, `rest_quiet`, and `ChunkEmitter` returns exactly one file — `tests/integration/poly_rotation.rs:134` — and it only constructs `BackoffCaps::default()` as config. No test calls `backoff_delay`, drives a `RestQuiet` through a window, or builds a `ChunkEmitter`.

The chunker's absence is the sharpest of the three, because the suite *looks* like it covers it. `tests/fitness/book_reconstruction.rs:47-58` hand-rolls a second cutter:

```rust
levels
    .chunks(chunk_size.clamp(1, BOOK_CHUNK_LEVELS))
    .map(|group| one_chunk(side, kind, group, false))
    .collect()
```

with its own zero-fill in `one_chunk` (`:22-32`) and `is_last_chunk: false` on every chunk it produces. So the book-reconstruction invariant validates the hot book against a *test-local* split, never against `ChunkEmitter::emit_book`. The module that exists so the cut is not made twice is cut twice — `src/adapters/chunk.rs:73` and the test — and the two copies can drift without any test going red. Finding 5 is invisible to the suite for exactly this reason: the oracle never emits the both-empty case, and never sets the terminator at all.

`RestQuiet` is the ironic one: `rest_quiet.rs:9` carries the docstring "Deterministic in `now` (fitness-testable)". The `now` parameter was threaded through all three methods specifically to make the module testable, and no test was written. Findings 2 and 3 are both cheap pins against that existing seam.

---

## Medium

### 7. The monotonic-stamp invariant is caller-maintained, not type-enforced

`src/adapters/chunk.rs:13-18`, `:44-47`, `:49-59`

`clamp_emit_ts` takes `&mut self` and is the sole guardian of the "never below already emitted" floor, but `emit_book` takes `&self` and accepts a `ChunkPlan` whose `received_ts_us` is a plain `pub(crate)` field. Nothing stops a caller from constructing a `ChunkPlan` with a raw stamp and never touching the emitter — the floor would silently stop applying, pushing out-of-order stamps into the hot path, which §2 replay determinism depends on.

All three current call sites comply (`src/adapters/binance/depth.rs:179`, `:236`, `src/adapters/polymarket/book.rs:88`), and two of them do it inline in the struct literal, which reads well. A fourth caller has nothing to warn it. Having `emit_book` take the raw receipt and clamp internally would make the invariant unbypassable and remove the field from the plan.

Note also that `clamp_emit_ts` is named like a pure transform but mutates the emitter's floor — the mutation is the point, and the name hides it.

### 8. A clock read failure silently disables backoff entirely

`src/adapters/backoff.rs:30-32`

`.map_or(0, ...)` turns a `duration_since(UNIX_EPOCH)` failure into `entropy = 0`, and `0 % n == 0`, so the function returns `Duration::ZERO` — no delay at all. Every reconnect loop listed in finding 1 then spins against the venue as fast as the runtime allows, which is the surest way to earn the ban the module exists to avoid. The trigger (system clock before 1970) is unlikely; the consequence is unbounded. §6 forbids swallowing a failure into a value, and this swallow lands on the safety mechanism itself. Falling back to the full `ceiling` rather than zero would fail safe.

The same zero-return appears deliberately at `backoff.rs:27-29` for `ceiling_us == 0`, so the two cases are indistinguishable at the call site.

### 9. `attempt` has no documented base, and one call site disagrees with the rest

`src/adapters/backoff.rs:21` — `backoff_delay(caps: &BackoffCaps, attempt: u32)` carries no docstring and no statement of whether `attempt` is 0-based.

Seven call sites pass `attempt` directly; `src/adapters/binance/rest/signed.rs:233` passes `attempt - 1`. That subtraction is safe *today* only because of a precondition ~28 lines away in a different function: `send_signed` increments `attempt` at `signed.rs:205` before calling `wait_before_retry`, and returns early at `:206-208`. Nothing local says so. If the increment ever moves, the underflow panics in debug and wraps in release to `1u32 << 16`, silently jumping straight to the max backoff.

Either document the base on the function, or take a newtype so the two conventions cannot be confused (§9 — bare integers should not impersonate one another).

### 10. `queued_ts_us` is seeded from `received_ts_us`, and only convention restamps it

`src/adapters/chunk.rs:89` — `queued_ts_us: plan.received_ts_us`

§5 requires that receive time and queue time never be silently interchanged; here they are equal by construction. Every current shell does restamp before the message reaches the ring (`src/adapters/binance/actor/mod.rs:358`, `src/adapters/binance/actor/tap.rs:16`, `src/adapters/polymarket/actor/effects.rs:77`, `src/adapters/exchange_sim/actor/mod.rs:185` and `:244`), so the value is a placeholder in practice rather than a live defect.

The brittleness is the failure mode when a future adapter forgets: `src/hot/metrics/mod.rs:74` reads `chunk.queued_ts_us` for latency, so the result is a *plausible zero* rather than a detectable error. A queue latency of exactly 0 µs across a whole run is the kind of number that reads as "fast" rather than "unstamped".

### 11. `len: group.len() as u8` couples silently to a constant in another file

`src/adapters/chunk.rs:84`, against `BOOK_CHUNK_LEVELS = 20` at `src/msg/inbound.rs:11`

The cast is correct at 20 and truncates silently above 255. Nothing at this line ties it to the constant, and the constant lives in a different module that a future capacity bump would naturally edit. The existing check is a `debug_assert` downstream (`src/msg/inbound.rs:107`) — debug-only, and it fires after the value has already been narrowed, so it would catch a `len` of 300 -> 44 only by luck. A `const` assertion next to the cast, or `u8::try_from(...).expect(...)` at this boundary, makes the coupling explicit.

---

## Low

### 12. Two names and two vocabularies for one field

`src/adapters/chunk.rs:39-47` — `note_queue_floor` and `clamp_emit_ts` have the same one-line body (`self.last_emitted_ts_us = self.last_emitted_ts_us.max(x)`) and differ only in return value. One speaks "queue floor", the other "emit ts", for the same `last_emitted_ts_us`. §1's one-design-language rule; a reader has to check both to learn they are the same operation.

### 13. Visibility differs across the three sibling files, and two are promises to nobody

- `src/adapters/rest_quiet.rs:11,16,20,26,30` are `pub` items inside a `pub(crate) mod` (`src/adapters/mod.rs:12`). Unreachable externally, so the `pub` promises nothing — §8 says "`pub(crate)` default, `pub` = promise".
- `src/adapters/backoff.rs:7` is `pub struct BackoffCaps` in a `pub mod` while `:21` `backoff_delay` is `pub(crate)`. An external caller can build the config but cannot call the only function that consumes it. `BackoffCaps` genuinely is reached externally from `tests/integration/poly_rotation.rs:20`, so this one is at least load-bearing.
- `src/adapters/chunk.rs` is consistently `pub(crate)` throughout — correct, and worth matching.

### 14. `RestQuiet` minor cleanups

- `rest_quiet.rs:21` — `retry_after_secs.unwrap_or(0).max(QUIET_FLOOR_SECS)`: the `unwrap_or(0)` is inert, since 0 always loses to the floor. It reads as "no header means no wait" and means "no header means 2 s". `unwrap_or(QUIET_FLOOR_SECS)` says it once.
- `rest_quiet.rs:26-34` — `is_active` and `remaining` each spell out `now < until`. `is_active` could be `self.remaining(now).is_some()`; as written the two comparisons must stay in sync by hand.
- `rest_quiet.rs:20` — `open` returns a bare `u64` of seconds, against §5's "bare `i64` never crosses an API" and §9's newtype rule. Both call sites use it only to interpolate into a WARN string (`binance/actor/mod.rs:392-393`, `signed.rs:228-229`), so the return could be dropped entirely in favour of the caller reading `remaining()`.
- `rest_quiet.rs:37-41` — the manual `Default` impl is boilerplate; `Option<Instant>` already defaults to `None`, so `#[derive(Debug, Default)]` covers it.

### 15. `backoff` minor cleanups

- `backoff.rs:24` — `1u32 << attempt.min(16)` has a bare `16` with no WHY. Under the default caps (250 ms initial, 30 s max) the ceiling saturates by attempt 7, so the cap is unreachable in practice; but `BackoffCaps` fields are `pub` and unvalidated, so a caller with a microsecond `initial` reaches it. Either name the constant or note why 16.
- `backoff.rs:26` — `ceiling.as_micros() as u64` narrows `u128` to `u64` unchecked. Only reachable via an extreme `caps.max`, but it is an unchecked cast fed by an unvalidated `pub` struct.
- `backoff.rs:7-10` — `BackoffCaps` has public unvalidated fields; `initial > max` is expressible. The `.min(caps.max)` at `:25` makes it harmless today, which is worth a line of doc rather than leaving the next reader to derive it.

---

## Suggested order of work

1. Clamp `Retry-After` and make `open` extend-only (findings 2, 3) — smallest diff, largest exposure reduction, and the `now` seam for pinning both already exists.
2. Move the both-empty guard into `emit_book` (finding 5), then delete Polymarket's exposure to it.
3. Mix a caller discriminator into the backoff entropy and fail safe on clock error (findings 1, 8).
4. Pin all three modules in `tests/fitness/`, and point `book_reconstruction.rs` at the real `ChunkEmitter` instead of its private copy (finding 6). Per §12 this is the step that keeps the rest from regressing.
