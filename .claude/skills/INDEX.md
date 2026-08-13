# Skill Index

Catalog of the rex skill set in `.claude/skills/`. The `rex-code-lead` team lead reads this index to decide which skills to equip each subagent with. Skills here are plain files — an agent is "equipped" with a skill by being told to Read its `SKILL.md` (and any reference files) in full before starting work.

## Canonical home and cross-tool discovery

`.claude/skills/` is the canonical, version-controlled home for every skill — one folder, no sync, no symlinks. Claude Code reads it natively and Cursor loads it as a compatibility location, so both tools share it directly. OpenAI Codex is NOT covered by this layout (Codex only reads `.agents/skills/` and drops symlinks); to support Codex later, reintroduce `.agents/skills/` as the canonical home with a copy-sync into `.claude/skills/`, plus per-skill `agents/openai.yaml` display metadata.

## Must-haves for every coding subagent

These three are loaded by EVERY coding subagent, every time, no exceptions:

| Skill               | Path                                          | Purpose                                                                                                                  |
| ------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| rex-code-philosophy | `.claude/skills/rex-code-philosophy/SKILL.md` | Complexity-averse, ergonomics-first programming philosophy (grugbrain + Worse-is-Better). Governs every design decision. |
| rex-code-ergonomics | `.claude/skills/rex-code-ergonomics/SKILL.md` | Hard rules for idiomatic, ergonomic Rust: naming, types, traits, iterators, lifetimes, panics, signatures.               |
| rex-code-commenting | `.claude/skills/rex-code-commenting/SKILL.md` | Hard rules for comments: best comment is none, WHY-only, short plain English with no jargon, plan references, or file pointers. Code-smell/hazard/SAFETY flags are permanent.  |

## Engineering skills

| Skill                                  | Path                                                                                                                | Purpose                                                                                               | Relevant when                                                                  |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| rex-code-tdd                           | `.claude/skills/rex-code-tdd/SKILL.md` (+ 5 reference files)                                                        | Red-green-refactor working method.                                                                    | Always for the engineer — this is HOW code gets written.                       |
| rex-code-error-writing                 | `.claude/skills/rex-code-error-writing/SKILL.md`                                                                    | Rust error handling: thiserror in libs, anyhow in bins, typed errors with context, no unwrap in prod. | Any code that can fail — i.e. nearly all engineering and auditing work.        |
| rex-code-external-libraries            | `.claude/skills/rex-code-external-libraries/SKILL.md`                                                               | Adding crates: latest stable, minimal features, verify via docs.rs.                                   | Task adds or upgrades a dependency.                                            |
| rex-code-improve-codebase-architecture | `.claude/skills/rex-code-improve-codebase-architecture/SKILL.md` (+ DEEPENING.md, INTERFACE-DESIGN.md, LANGUAGE.md) | Find deepening/refactor opportunities; module boundaries; domain language.                            | Refactors, architecture reviews, and ALWAYS for the auditor's module-shape review. |
| rex-code-smells                        | `.claude/skills/rex-code-smells/SKILL.md`                                                                           | Detection playbook: naming, smell catalog, brittleness signals, grep/clippy sweeps, findings table. Detection only — never fixes. | ALWAYS for the auditor (Stage 2) and the cleaner's final sweep (Stage 3, opus). Any smell hunt or quality audit. |

## Testing skills

| Skill                              | Path                                                         | Purpose                                                                                                                        | Relevant when                                                                      |
| ---------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| rex-code-tests-unit-testing        | `.claude/skills/rex-code-tests-unit-testing/SKILL.md`        | Unit tests = inline TDD scaffolds. Delete when contract/integration coverage is green; promote critical invariants to fitness. | Engineer (while TDD-ing) and cleaner (deciding what to remove).                    |
| rex-code-tests-integration-testing | `.claude/skills/rex-code-tests-integration-testing/SKILL.md` | Tests against the real outside world in `tests/integration/`. Real-funds tests need a human-in-the-loop runbook.               | Code touches APIs, DBs, sockets, the file system, or time.                         |
| rex-code-tests-contract-seams      | `.claude/skills/rex-code-tests-contract-seams/SKILL.md`      | Tests of stable module-to-module interfaces in `tests/contract/`, real impls on both sides.                                    | Task defines or changes a module boundary.                                         |
| rex-code-tests-fitness-functions   | `.claude/skills/rex-code-tests-fitness-functions/SKILL.md`   | Few, immortal architectural invariants in `tests/fitness/`, proptest for large input spaces.                                   | Load-bearing invariants exist; also tells the cleaner which tests are untouchable. |

## Utility skills

| Skill                      | Path                                                                            | Purpose                                                                   | Relevant when                                                         |
| -------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| rex-grill-me               | `.claude/skills/rex-grill-me/SKILL.md`                                          | Relentlessly interview the user about a plan until shared understanding.  | User wants their plan stress-tested. Not part of the coding pipeline. |
| rex-caveman          | `.claude/skills/rex-caveman/SKILL.md`                                     | Ultra-compressed communication mode (~75% fewer tokens).                  | User asks for caveman mode / brevity. Never changes code style.       |
| rex-mermaid-diagrams | `.claude/skills/rex-mermaid-diagrams/SKILL.md` (+ README.md, references/) | Author Mermaid diagrams: class, sequence, flowchart, ERD, C4, state, etc. | Task asks for diagrams or architecture documentation.                 |

## Orchestration

| Skill         | Path                                    | Purpose                                                                                                                                                                                                           |
| ------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| rex-code-lead | `.claude/skills/rex-code-lead/SKILL.md` | Team-lead playbook: plan (tmp dir) → advisor sense-check → engineer → auditor → cleaner. Takes a coding task from planned to written to audited to polished. The lead consults THIS index to equip each subagent. |
