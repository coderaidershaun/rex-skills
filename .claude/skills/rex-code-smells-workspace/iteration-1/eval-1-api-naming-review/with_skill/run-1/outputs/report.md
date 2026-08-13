# Smell audit: `fixtures/api.rs` (public document API)

Scope = one file, read whole. Detection only — no edits made to the fixture.

| # | Where | Lens | Smell | Sev | Evidence | Fix direction |
|---|-------|------|-------|-----|----------|---------------|
| 1 | api.rs:11, api.rs:20 | brittleness | data with no writer | BLOCK | `skipped: u32` is a public result field, but `parser_run` hardcodes `ParserResult { documents, skipped: 0 }` while line 17's `.filter(...)` genuinely drops chunks. A caller reading `result.skipped == 0` concludes "nothing was dropped" while chunks were silently discarded. | Count what the filter rejects and write it, or delete the field until it is wired. |
| 2 | api.rs:6 | brittleness | config knob with no reader | BLOCK | `ParserConfig::max_depth` is `pub` and never read in `parser_run` (lines 14–21). A caller sets `max_depth: 3`, gets no depth limiting and no error — the guard they think they installed does not exist. | Enforce it in the parse loop, or remove it from the public config until it is enforced. |
| 3 | api.rs:38–41 | brittleness | unchecked byte slicing on external input | BLOCK | `as_summary` does `self.body[..end]` with `end = self.body.len().min(40)`. `min` bounds the length but not the UTF-8 char boundary, so any body whose 40th byte lands mid-character panics. Bodies come from `parser_run`'s `input: &str`, i.e. caller data. | Truncate on a char boundary (`char_indices`, or `chars().take(40)`). |
| 4 | api.rs:47 | naming | `into_` prefix lies about cost | REFACTOR | `pub fn into_tags(&self) -> Vec<String>` takes `&self` and does `self.tags.clone()`. `into_` promises a consuming move (free); this allocates a fresh `Vec<String>` on every call. Called in a loop it is O(n) heap traffic the name says is free. | `fn tags(&self) -> &[String]`, plus `to_tags()` if an owned copy is really needed. |
| 5 | api.rs:43 | naming | `to_` prefix lies + non-standard name | REFACTOR | `pub fn to_len(&self) -> usize` returns `self.body.len()` — a free field read, but `to_` signals conversion work. It is also the one method in Rust with a fixed conventional name, and this is not it. | Rename to `len()`; add `is_empty()` beside it (clippy's `len_without_is_empty`). |
| 6 | api.rs:38 | naming | `as_` prefix lies about cost | REFACTOR | `pub fn as_summary(&self) -> String` allocates. `as_` means a free borrowed view; a caller reasonably calls this per render frame expecting zero cost. | Rename to `to_summary()`, or return `&str` and make the prefix true. |
| 7 | api.rs:34 | naming | `get_` prefix on a plain getter | REFACTOR | `pub fn get_body(&self) -> &str`. Rust reserves `get` for fallible/indexed access (`slice::get`); a plain accessor takes the field name. | Rename to `body()`. |
| 8 | api.rs:51 | naming | verb says nothing about the subject | REFACTOR | `pub fn iterate(&self)` on a type owning *both* `body` and `tags` (lines 25–26). Nothing in the name tells the caller which one is being iterated; the return type is the only clue. | Rename to `tags()`; `iter()` is only right when a type has one obvious sequence. |
| 9 | api.rs:51 | brittleness | concrete std type leaked in a public API | REFACTOR | Returning `std::slice::Iter<'_, String>` freezes the private `Vec<String>` representation into the published contract — switching `tags` to `VecDeque`/`SmallVec`/interned ids becomes a breaking change. It also hands callers `&String` where `&str` is wanted. | Return `impl Iterator<Item = &str>` or `&[String]`. |
| 10 | api.rs:14 | naming | stuttering | REFACTOR | `parser::parser_run` — the module path already carries `parser`, so call sites read `parser::parser_run(...)`. `run` alone is also contentless for a parse. | `parser::parse(config, input)`. |
| 11 | api.rs:4, api.rs:9 | naming | stuttering + a `Result` that cannot fail | REFACTOR | `parser::ParserConfig` / `parser::ParserResult` restate the module. Worse, `ParserResult` is not a `Result`: it is infallible output, so `parser_run(..)?` will not compile and a reader must open the file to learn that. | `parser::Config`; rename the output to `Parsed` / `ParseOutput` and reserve `*Result` for fallible returns. |
| 12 | api.rs:5, api.rs:17 | naming | bool field is not a predicate, and hides its own meaning | REFACTOR | `pub strict: bool`. §9 wants `is_`/`has_`/`can_`. More costly: line 17's `!config.strict \|\| !chunk.is_empty()` means `strict` does exactly one thing — drop empty chunks. A caller setting `strict: true` expects validation errors, not silent filtering. | Replace with an enum that names the behaviour: `EmptyChunks::{Skip, Keep}`. |
| 13 | api.rs:11 | naming | unit-less / noun-less quantity | NIT | `skipped: u32` — skipped what? Chunks, bytes, documents? | `skipped_chunks`. |
| 14 | api.rs:39 | brittleness | unexplained magic literal | NIT | `.min(40)` — 40 what, and why 40? Bytes here, though the name `summary` implies characters (see finding 3). | Named `const SUMMARY_LEN_CHARS: usize`. |
| 15 | api.rs:4, 9, 24 | ergonomics | no derives on published types | NIT | `ParserConfig`, `ParserResult`, `Document` derive nothing. Callers cannot `{:?}` them in their own tests or compare two `Document`s. | `#[derive(Debug, Clone, PartialEq, Eq)]` where semantics allow; `Default` on `ParserConfig`. |

**Clean:** `Document::new` (line 30) and `push_tag` (line 55) are correctly named and correctly signatured — both take `&str` rather than forcing a caller allocation, and `push_tag` says exactly what it does to what. Parameter counts are all ≤ 2, so no long-param-list or data-clump findings. No `unwrap`/`expect`, no float-in-state logic, no wall-clock reads, no stringly-typed matching, no duplicated blocks, no commented-out code.

**Not checked:** call sites (none exist in this fixture, so "does any caller depend on the current names" is unanswered — findings 4–11 are all source-breaking renames); no compile or clippy run, since the file is a standalone fixture outside a crate target.

**15 findings: 3 BLOCK / 9 REFACTOR / 3 NIT**

---

## The three a caller misreads today

Findings 1, 2 and 3 are not style. They are the places where the API states something untrue:

- `skipped` always reports `0` while the parser is dropping chunks (line 20 versus line 17).
- `max_depth` is a knob wired to nothing (line 6).
- `as_summary` panics on the first non-ASCII document (line 40).

Findings 4–6 are the next tier: `into_`, `to_` and `as_` are the prefixes a Rust caller uses to infer *cost* without reading the body, and all three currently lie, in three different directions.
