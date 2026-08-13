# Order gateway review — `planted.rs`

**Verdict: reject. Do not merge in any form.**

This is 210 lines that claims to be an order gateway. It loses money in four
distinct ways, panics on hostile input, silently discards every write it claims
to persist, and ships a green two-test suite that covers none of it. The naming
and structure problems are real but secondary — the correctness defects below
are the reason this cannot land.

Every behavioural claim marked **[verified]** was reproduced by compiling the
file and running it, not inferred from reading.

| # | Severity | Finding | Line |
|---|---|---|---|
| 1 | Critical | `transfer` mints money from nonexistent accounts | 164 |
| 2 | Critical | Persistence is a silent no-op | 62, 136 |
| 3 | Critical | `NaN` price passes validation and poisons all totals | 105, 152 |
| 4 | Critical | Cap `break` returns money it never posted | 119 |
| 5 | Critical | Balances silently clamped, money vanishes | 127 |
| 6 | High | Unchecked indexing + `unwrap` on wire input | 101 |
| 7 | High | `f64` money accumulators throughout | 6, 16, 17 |
| 8 | High | Lossy `u64 as f64` on transfer amount | 165 |
| 9 | High | No overdraft check; balances go negative | 166 |
| 10 | High | Tests are tautological; money paths untested | 185 |
| 11 | Medium | `load_and_process` does five unrelated jobs | 93 |
| 12 | Medium | Blocking `sleep` buried in a parse function | 141 |
| 13 | Medium | Duplicated validation, two return shapes | 105, 152 |
| 14 | Medium | Two-phase construction guarded by bare `assert!` | 84, 94 |
| 15 | Medium | Temporary field `parse_buffer`, written never read | 10 |
| 16 | Medium | Message chain / Law of Demeter | 172 |
| 17 | Medium | `Backend<T>` — type parameter pays no rent | 58 |
| 18 | Medium | Broken `checksum`, trivially collidable, dead | 177 |
| 19 | Low | Magic number `86_400.0` doing two unrelated jobs | 119, 127 |
| 20 | Low | Naming: `DataManager`, `usr_cnt`, `get_name`, `RetryCfg` | various |

---

## Critical — money defects

### 1. `transfer` creates money out of nothing — `planted.rs:164`

```rust
pub fn transfer(&mut self, from_account: u64, to_account: u64, amount: u64) {
    let value = amount as f64;
    if let Some(balance) = self.accounts.get_mut(&from_account) {
        *balance -= value;                                    // conditional
    }
    *self.accounts.entry(to_account).or_insert(0.0) += value; // unconditional
}
```

The debit is guarded by `if let Some`. The credit is not. Transfer from an
account that does not exist and the destination is credited anyway, funded by
nobody.

**[verified]** `transfer(999, 1, 1_000)` on an empty ledger yields
`{1: 1000.0}` — a thousand units conjured from an unknown account id.

This is also non-atomic and returns `()`, so a caller has no way to learn the
transfer failed. It cannot fail; it can only be wrong. The function needs to
resolve both accounts before mutating either, and return
`Result<(), TransferError>` with variants for unknown-source and
insufficient-funds.

### 2. Persistence is a lie — `planted.rs:62`, `planted.rs:136`

The file header says "persists totals". It does not.

```rust
pub struct FileBackend;

impl Backend<String> for FileBackend {
    fn store(&mut self, _key: u64, _value: f64) {}   // :65 — empty body
}
```

`FileBackend` has no file, no handle, no path, and an empty `store`. The
`!dry_run` branch at `:135` walks the whole account map and throws every balance
away. Worse, `store` returns `()`, so a persistence API that *cannot report
failure* is baked into the trait at `:59` — even a real implementation could
never signal a failed write.

Compounding it, `:136` constructs `FileBackend` inline. The `Backend` trait
exists but is never used for injection, so it buys nothing: the deletion test
from our own §1 applies — remove the trait and behaviour is identical.

`store` must return `Result`, and the backend must be a constructor parameter,
not a local.

### 3. `NaN` price is accepted as valid — `planted.rs:105`, `planted.rs:152`

```rust
if price <= 0.0 { rejected += 1; continue; }
```

`NaN <= 0.0` is `false`, so a `NaN` price is not rejected — it is *validated*.
It then flows into `total += notional` at `:118` and poisons the accumulator
permanently. Because `NaN > 86_400.0` is also `false`, the cap check at `:119`
can never fire again either: the one safety rail in the function is disabled by
the same value.

**[verified]** feeding `1,NaN,5\n2,10.0,3\n` returns `NaN` as the batch total.
The literal string `NaN` parses successfully via `f64::from_str`, so this is
reachable directly from the wire — no arithmetic overflow required.

`revalidate` at `:151` has the identical hole: a `NaN`-priced order is reported
valid.

Validation must be `if !(price > 0.0)`, or better, parse into a type that cannot
hold a non-finite value.

### 4. The cap `break` returns money it never credited — `planted.rs:119`

```rust
total += notional;
if total > 86_400.0 {
    ...
    break;                                    // :123
}
let account = self.accounts.entry(id).or_insert(0.0);   // :125 — skipped
*account += notional;
```

`total` is incremented at `:118`, then the loop breaks at `:123` *before* the
account is credited at `:125-126`. The returned total therefore includes a
notional that exists in no account.

**[verified]** two 50,000-notional lines return `total = 100000` while the
ledger holds `{1: 50000.0}`. Half the reported volume was never posted anywhere.

The counters have the same problem: `seen` at `:100` counts the broken line as
processed and `rejected` at `:98` does not count it as rejected, so the summary
printed at `:146` is wrong on every capped batch.

### 5. Balances are silently clamped — `planted.rs:127`

```rust
if *account > 86_400.0 {
    *account = 86_400.0;
}
```

An account crossing the threshold has the excess deleted with no error, no
counter, no log. Money entering the system disappears from the ledger, and the
next `transfer` out of that account operates on a fabricated balance. A capacity
limit is a legitimate design choice; silently overwriting state to satisfy it
is not. This must be a rejected order or a typed error, never an assignment.

---

## High — brittleness

### 6. Unchecked indexing and `unwrap` on untrusted input — `planted.rs:101-104`

```rust
let parts: Vec<&str> = line.split(',').collect();
let id = parts[0].parse::<u64>().unwrap();
let price = parts[1].parse::<f64>().unwrap();
let qty = parts[2].parse::<f64>().unwrap();
```

Four separate panic sites per line, on data the header comment says arrives
"over the wire". A short line, a blank line, a trailing comma, a non-numeric
field, or a partial frame takes the process down.

**[verified]** the single line `1,10.0` panics with
`index out of bounds: the len is 2 but the index is 2`.

Malformed input from a remote peer is an *expected external* condition, not an
invariant violation, and must be handled by policy — count it as rejected and
continue, the way `:106` already handles a bad price. As written, anyone who can
send bytes to this gateway can halt it.

The function signature makes this unfixable in place: it returns bare `f64`,
so there is no channel to report a parse failure even if one were caught. It
needs `Result<BatchOutcome, ParseError>` where the outcome carries
`total`/`seen`/`rejected` as fields rather than as `println!` side effects.

Secondary: `:101` allocates a `Vec` per line inside the parse loop.
`split(',')` with a destructuring `next()` triad avoids it entirely.

### 7. `f64` as a money accumulator — `planted.rs:6`, `:16`, `:17`

`accounts: HashMap<u64, f64>`, `price: f64`, `qty: f64`. Balances accumulate
`price * qty` across unbounded batches with no exact representation, so
positions drift, and equality against a balance is meaningless. Fixed-point
integer mantissas are the only correct representation for a ledger.

### 8. Lossy `amount as f64` — `planted.rs:165`

`u64` values above 2^53 lose precision on the cast, and `as` performs it
silently.

**[verified]** `transfer(1, 2, u64::MAX)` credits `1.8446744073709552e19` —
not the requested amount.

### 9. No overdraft check — `planted.rs:166`

`*balance -= value` with no comparison against `value`. Balances go arbitrarily
negative and nothing downstream notices.

### 10. The tests are tautological — `planted.rs:185-210`

Two tests, both green, both worthless:

- `checksum_stable` at `:207` asserts `checksum(&[1,2,3]) == checksum(&[1,2,3])`
  — comparing a pure function against itself. This passes for *any*
  implementation, including one that returns `0` unconditionally. It pins
  nothing.
- `order_debug_shape` at `:190` asserts the exact `{:?}` string of a derived
  `Debug`. It tests the derive macro, not this code, and breaks on any field
  rename or addition — maximum brittleness for zero coverage. `Debug` output is
  explicitly not a stable contract.

**[verified]** both tests pass against the current file. Meanwhile
`load_and_process`, `transfer`, and `revalidate` — every function that touches
money — have no test at all. The suite exists to be green, not to catch
regressions.

---

## Medium — structure and design

### 11. `load_and_process` is five functions — `planted.rs:93`

```rust
pub fn load_and_process(&mut self, raw: &str, timeout: u64, dry_run: bool, verbose: bool) -> f64
```

The `and` in the name is the tell. It parses, validates, accumulates, mutates
the ledger, persists, sleeps, and prints — then returns one of the five values
it computed and drops the rest.

The signature has its own problems: four parameters, two of them `bool`, giving
call sites like `load_and_process(raw, 0, false, true)` where nothing is
readable without opening the definition. `verbose` should not be a parameter at
all — it threads a logging concern through the signature and produces the
`println!` calls at `:121`, `:132`, and `:146`. `dry_run` selects between two
genuinely different operations and belongs in the type or in separate methods.

### 12. Blocking sleep inside a parse function — `planted.rs:140-141`

```rust
if timeout > 0 {
    std::thread::sleep(std::time::Duration::from_millis(timeout));
}
```

A parameter named `timeout` that is used as an unconditional sleep duration is
misnamed — a timeout bounds how long you *wait for something*; this just blocks.
Blocking the caller's thread as a side effect of processing a batch is
indefensible in a gateway. The units live only in the `from_millis` call, not in
the name or the type.

### 13. Duplicated validation with divergent shapes — `planted.rs:105-116` vs `:152-160`

The same three predicates (`price <= 0.0`, `qty <= 0.0`, `id == 0`) appear
twice, once counting rejections inline and once returning `bool`. Two copies of
a money-safety rule will diverge — and both already share the `NaN` hole in
finding 3, so a fix has to land in two places. Extract one predicate returning a
typed reason and call it from both.

Related: `id == 0` treats zero as a sentinel for "invalid" rather than making
invalid ids unrepresentable (`NonZeroU64`, or a validated `OrderId` newtype).

### 14. Two-phase construction guarded by a bare assert — `planted.rs:74-91`, `:94`

`new()` returns an object that cannot be used, `init()` at `:84` makes it
usable, and `assert!(self.initialized)` at `:94` panics if a caller gets the
order wrong. This is a type problem solved with a runtime crash. `new()` should
return a ready object, or an unready one should be a distinct type whose only
method produces the ready one.

The two flags at `:8-9` are always written together at `:85-86` and always read
together at `:90`, so three of the four states they encode are unreachable —
one field, or better, no field.

`is_ready()` at `:89` checks `initialized && active`, but the assert at `:94`
checks only `initialized`. Two different definitions of ready.

### 15. Temporary field — `planted.rs:10`

`parse_buffer: Option<String>` is set at `:95`, never read anywhere, and cleared
at `:144`. It exists only for the duration of one call, which makes it a local
variable that escaped into the struct. It also clones the entire input for no
reason. Delete it.

### 16. Message chain — `planted.rs:172-174`

```rust
pub fn shipping_city(order: &Order) -> &str {
    order.customer().address().city()
}
```

Three hops through two intermediate types, which is why the pass-through getters
at `:33`, `:39`, and `:45` exist at all — they are there to serve this one
chain. Any change to how an `Order` stores its destination breaks all four
sites.

Worse, `shipping_city` is an associated function on `DataManager` that never
touches `self`. It has nothing to do with the ledger; it belongs on `Order`.
`get_name` at `:49` has the same problem in miniature — it reaches into
`self.customer.name`, and its name says "order name" while it returns a
customer's.

### 17. `Backend<T>` — a type parameter that pays no rent — `planted.rs:58`

```rust
pub trait Backend<T> {
    fn store(&mut self, key: u64, value: f64);
}
```

`T` appears in no method signature. It cannot be inferred, which is why the call
site at `:138` is forced into
`Backend::<String>::store(&mut backend, *id, *balance)` instead of
`backend.store(id, balance)`. The parameter provides nothing and taxes every
caller. `RetryCfg` at `:68` is the same species — a public type with no
constructor, no reference, and no user anywhere in the file. Both are
speculative generality; delete them.

Also at `:59`, `store` takes `key: u64` and `value: f64` — the same primitive
obsession as the map itself, with nothing preventing the two from being swapped
if the signature ever grows.

### 18. `checksum` does not checksum — `planted.rs:177-183`

```rust
sum = sum.wrapping_add((*byte as u32) << (i % 4));
```

The shift wraps every four bytes, so the function is blind to position beyond a
four-byte window and to trailing zeros entirely.

**[verified]** all three of `[1,2,3,4]`, `[1,2,3,4,0,0,0,0]`, and
`[0,0,0,0,1,2,3,4]` hash to `49`. Padding and leading zeros are invisible;
length is invisible. If anything downstream ever relies on this to detect
corruption, it will not.

It is also dead — the compiler flags `function 'checksum' is never used`, and
its only caller is the tautological test in finding 10. Delete it, or replace it
with a real digest if a caller is actually planned.

---

## Low — naming

- `DataManager` (`:5`) — "Data" and "Manager" both mean nothing. It is an
  account ledger; name it one.
- `usr_cnt` (`:7`) — two abbreviations in one field. It is also derived state,
  recomputed from `self.accounts.len()` at `:130` *inside the loop* on every
  iteration, narrowed through a silent `as u32`, and then never read by anything.
  Delete the field; call `.len()` if a caller ever needs it.
- `get_name` (`:49`) — Rust getters drop the `get_` prefix by convention. See
  also finding 16: the name is wrong on top of the convention.
- `as_json` (`:53`) — `as_*` signals a cheap borrowed conversion; this allocates,
  so it should be `to_json`. It is also hand-rolled JSON with no escaping (a
  quote or backslash in any future string field produces invalid output) and it
  silently omits `qty` and `customer`, so it is not a serialization of the type
  it claims to serialize.
- `RetryCfg` (`:68`) — `Cfg`; and see finding 17, it is dead anyway.
- `revalidate` (`:151`) — the `re-` prefix implies a prior validation pass that
  has no name in this module. It also takes `&self` without using it, so it is
  not a method on the ledger; it is a predicate on `Order` and should live there
  as `is_valid`, returning a typed reason rather than a bare `bool` that tells
  the caller nothing about which of three checks failed.
- `86_400.0` (`:119`, `:127`) — an unnamed constant used for two unrelated
  limits: a per-batch total cap and a per-account balance ceiling. That these
  are the same number is almost certainly a copy-paste, since 86,400 is seconds
  in a day, not a money bound. Two named constants.
- `Order` (`:13-19`) mixes three `pub` fields with one private one, so
  encapsulation is decided per field rather than by design.
- `new()` at `:74` without a `Default` impl — clippy's `new_without_default`
  fires on this.
- The header comment at `:1` describes *what* the file does and is factually
  wrong about the persistence half (finding 2). A comment that lies is worse
  than none.

---

## Recommendation

The naming and structural findings are ordinary review debt. Findings 1–6 are
not: this module gives away money on an unknown-account transfer, accepts `NaN`
as a valid price, drops orders it has already counted, silently deletes balance
above a cap, throws away every write, and can be halted by a malformed line from
a remote peer. Its own test suite passes.

Do not adopt this file. If the contractor's work is needed, rewrite the ledger
and the parse loop against integer money and a `Result`-returning API, and treat
the existing tests as absent.
