# API naming review — `fixtures/api.rs`

Reviewed: `/Users/shaun/Code/DEVELOPMENT/polydata/.claude/skills/rex-code-smells-workspace/fixtures/api.rs` (59 lines).

**Headline:** six of the seven public methods on `Document` carry a conversion prefix that
contradicts what the signature actually does. A reader who trusts the Rust API guidelines (C-CONV,
C-GETTER, C-ITER) will predict the wrong cost, the wrong ownership, and in one case the wrong return
type from nearly every one. Two findings are outright defects rather than naming: `skipped` is never
incremented, and `as_summary` panics on non-ASCII input.

Findings are ordered by how badly a caller is misled.

---

## Blocking — the name states the opposite of the signature

### 1. `into_tags(&self)` — line 47

```rust
pub fn into_tags(&self) -> Vec<String> {
    self.tags.clone()
}
```

`into_` is reserved for conversions that **consume `self` by value** and transfer ownership without
copying. This one borrows and clones. The name promises the `Document` is gone afterwards; the
compiler happily lets the caller keep using it, so the contradiction surfaces as a surprise
allocation in a loop rather than a compile error — the worst kind, because it never shows up in
review.

Pick one and mean it:

- `pub fn to_tags(&self) -> Vec<String>` — keeps the clone, names it honestly, or
- `pub fn into_tags(self) -> Vec<String>` — actually consumes, returns `self.tags`, no clone at all.

### 2. `as_summary(&self) -> String` — line 38

`as_` means a free, borrowed view — a pointer cast at most. Returning an owned `String` breaks that
twice: it allocates, and it *truncates* to 40 bytes, which nobody expects from a name in the `as_`
family. Rename to `to_summary()` at minimum.

Better, say what the 40 is — `summary_prefix()`, or take the width as a parameter
(`summary(&self, max_chars: usize)`). A hardcoded 40 buried behind `as_` is the sort of constant
callers discover from a truncated log line in production.

### 3. `to_len(&self) -> usize` — line 43

`to_` signals a non-trivial, usually allocating conversion into another type. This is a plain field
read, so every reader who trusts the prefix hoists it out of a loop for no reason.

It should be `len()`. Two follow-ons once it is:

- Clippy's `len_without_is_empty` will (correctly) demand an `is_empty()` beside it.
- On a type holding both `body: String` and `tags: Vec<String>`, a bare `len()` is ambiguous —
  length of the body, or the number of tags? If both are ever exposed, name it `body_len()` and let
  the tag count come from `tags().count()`.

### 4. `get_body(&self)` — line 34

The `get_` prefix is un-idiomatic in Rust (C-GETTER); an accessor takes the field's own name, so
this is `body()`. Reserve `get_` for the fallible indexed form (`get(i) -> Option<&T>`), which is the
only place the standard library uses it.

---

## Blocking — a name that lies about behaviour

### 5. `ParserResult.skipped` is always zero — lines 11, 17, 20

```rust
.filter(|chunk| !config.strict || !chunk.is_empty())   // line 17 — chunks ARE dropped here
...
ParserResult { documents, skipped: 0 }                  // line 20 — but the count is hardcoded
```

The field exists to report how many chunks were discarded, line 17 discards them, and the
constructor hardcodes `0`. A caller writing `if result.skipped > 0 { warn!(...) }` gets silence on
every input. This is a defect, not a rename: either wire the count (fold or partition instead of
`filter`) or delete the field until it works.

If it stays, `skipped` alone does not say skipped-what. `skipped_chunks: u32` is the minimum; if the
reason ever matters, a small enum-keyed count beats a bare number.

### 6. `ParserConfig.strict: bool` does not mean strict — lines 5, 17

`strict` reads as "reject malformed input, fail loudly". What line 17 does is *silently drop empty
chunks*. A caller enabling `strict` to get stricter error reporting instead gets quieter output —
the opposite of what the name advertises.

Name the behaviour: `skip_empty_documents: bool`. Better, since this is a public surface and the
policy will grow a third case, an enum:

```rust
pub enum EmptyDocument { Skip, Keep }
```

which also removes the bare `bool` from the public API.

Related: `chunk.is_empty()` catches only the zero-length string, so a chunk of `"   \n"` survives
"strict" mode. Use `trim().is_empty()` if whitespace-only should count as empty; if not, the name
must be narrower still.

### 7. `ParserConfig.max_depth` has no reader — line 6

`max_depth` appears nowhere in `parser_run` (lines 14–21), or anywhere else in the file. It is a
public knob that does nothing. A caller will set it, believe they have bounded something, and ship.
Delete it, or wire it before publishing — an inert public field is a compatibility promise you gain
nothing for.

---

## Significant — stutter and vocabulary drift

### 8. Module-name stutter — lines 3, 4, 9, 14

Every item in `mod parser` repeats the module name, so the call site reads:

```rust
let config = parser::ParserConfig { strict: true, max_depth: 4 };
let out = parser::parser_run(&config, input);
```

The path already carries `parser`. Convention (C-STUTTER) is to strip it:

| now | should be |
| --- | --- |
| `parser::ParserConfig` | `parser::Config` |
| `parser::ParserResult` | `parser::Output` (see finding 9) |
| `parser::parser_run` | `parser::parse` |

`run` is also the weaker verb — it says a thing happened, not what. `parse` is the domain verb, and
the module is already named for it.

### 9. `ParserResult` is not a `Result` — line 9

A public type suffixed `Result` reads, in Rust, as the crate's error-carrying alias — the
`io::Result<T>` / `fmt::Result` pattern. This one is an infallible struct. Callers will reach for
`?`, get a type error, and have to read the definition to learn the function cannot fail at all.
Rename to `parser::Output`, `ParsedDocuments`, or `ParseSummary`.

Worth fixing before publishing specifically because it is unfixable afterwards without a major
version: the moment `parse` does become fallible, `Result` is the name you will want back.

### 10. Three vocabularies for one field — lines 47, 51, 55

`tags` is reachable via `into_tags`, `iterate`, and `push_tag`. Nothing in `iterate`'s name connects
it to tags, and nothing tells a caller that `into_tags` and `iterate` are two views of the same
data. Settle on one family:

```rust
pub fn tags(&self) -> impl Iterator<Item = &str> + '_   // replaces iterate
pub fn into_tags(self) -> Vec<String>                    // consuming, per finding 1
pub fn push_tag(&mut self, tag: &str)                    // already fine
```

`push_tag` is the one name in the file that is already right — `push` correctly implies ordered,
append-at-end semantics, matching the `Vec` underneath.

### 11. `iterate` — line 51

Two problems past the vocabulary drift above.

*The name:* the convention is `iter()` (C-ITER), and on a type with more than one collection inside,
the iterator method is named for what it yields — `tags()`.

*The signature:* returning `std::slice::Iter<'_, String>` welds the internal representation into the
public API. Switching `tags` to a `SmallVec`, a `BTreeSet`, or interned handles becomes a breaking
change — for a type being published precisely so it can evolve. Return
`impl Iterator<Item = &str> + '_` and the storage stays yours. Yielding `&str` rather than `&String`
is the ergonomic default too; callers almost never want the extra indirection.

---

## Worth fixing while you are in here

### 12. `as_summary` can panic on valid input — lines 39–40

```rust
let end = self.body.len().min(40);
self.body[..end].to_string()
```

`String::len` is **bytes**, and slicing a `str` at a non-char-boundary panics. Any body whose 40th
byte lands mid-codepoint — one emoji, one accented character, any CJK text — panics at runtime.
`self.body.chars().take(40).collect::<String>()` is the direct fix; `char_indices` if you want to
bound bytes rather than characters. Worth a regression pin, since it is reachable from public input.

### 13. Free function where a method belongs — line 14

`parser_run(config: &ParserConfig, input: &str)` puts the configuration in a free function's
parameter list. Behaviour belongs on the type:

```rust
impl parser::Config {
    pub fn parse(&self, source: &str) -> parser::Output
}
```

Call site becomes `config.parse(source)`. While there: `input` is vague for
"newline-`---`-delimited multi-document text" — `source` says more.

### 14. Missing common traits — lines 4, 9, 24

None of `ParserConfig`, `ParserResult`, or `Document` derive anything. `Debug` is effectively
mandatory on a public API (C-COMMON-TRAITS) — without it, callers cannot put your types inside their
own `#[derive(Debug)]` structs, a viral limitation they cannot work around. Add
`Debug, Clone, PartialEq, Eq` wherever the semantics allow, plus `Default` on `ParserConfig` so
adding a field later is not a breaking change for every caller with a struct literal.

### 15. `Document::new(body: &str)` — line 30

Defensible as-is, and the only name in the file I would leave alone. A reader may wonder whether
tags are required given `new` takes a body only; `Document::from_body(...)` would say the absence is
deliberate. Empty-tags-by-default is a reasonable reading of `new`, so this is low priority.

---

## Suggested shape after the renames

```rust
pub mod parser {
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Config {
        pub empty_documents: EmptyDocument,
        // max_depth deleted until it is wired
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Output {
        pub documents: Vec<super::Document>,
        pub skipped_chunks: u32,   // actually counted
    }

    impl Config {
        pub fn parse(&self, source: &str) -> Output { ... }
    }
}

impl Document {
    pub fn body(&self) -> &str;
    pub fn to_summary(&self) -> String;              // char-safe
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn tags(&self) -> impl Iterator<Item = &str> + '_;
    pub fn into_tags(self) -> Vec<String>;
    pub fn push_tag(&mut self, tag: &str);
}
```

## Priority

1. **Findings 5, 6, 12** — `skipped` never counts, `strict` means something other than strict, and
   `as_summary` panics on non-ASCII. Wrong behaviour, and two of the three are silent.
2. **Findings 1–4, 9** — the prefix contradictions and `ParserResult`. Free to rename now, a major
   version later.
3. **Findings 7, 8, 10, 11** — the dead `max_depth`, the stutter, the tag vocabulary, and the leaked
   `slice::Iter`. That last one permanently locks your storage choice.
4. **Findings 13–15** — ergonomics and derives.
