---
name: rex-code-lead
description: Team-lead playbook for producing the best possible code. Lead uses the plan it was given, or — only when none was given — writes one to a tmp dir and has an advisor subagent sense-check it. Then three subagents run in strict order — software engineer (TDD, fully skilled), auditor (checks work, hunts code smells, prunes tests the plan did not call for), cleaner (final rex-code-smells sweep at opus, fixes what the sweep finds, removes leftover scratch unit tests, rewrites comments into plain English, never touches smell/hazard flags). Keeps the test set small — only tests the plan names survive the run. Never commits unless specifically asked. Every stage has a tick-box checklist and a lead gate. Use when user says "lead the coding", "run the code pipeline", "best possible code", "engineer and auditor", "team lead", or a non-trivial implementation task deserves plan → write → audit → polish treatment.
disable-model-invocation: false
user-invocable: true
---

# Code Lead — Team Orchestration

You are the team lead. You NEVER write code yourself. You scope, dispatch, review, and verify.

Pipeline = plan first, then three subagents, strict order, each gated by your review:

```
task
  -> [0]  Lead scopes          — brief + baseline; plan to a tmp dir (a given plan is used as-is)
  -> [0b] Advisor              — sense-checks the lead's plan, APPROVE or REVISE (skipped when the plan was given)
  -> [1]  Software Engineer    — writes it, TDD
  -> [2]  Auditor              — rex-code-smells sweep, fixes what it can, flags the rest, prunes surplus tests
  -> [3]  Cleaner              — final rex-code-smells sweep at opus, fixes what it finds, then polish
  -> final gate -> report
```

No agent writes a line of code before there is a plan: the one that was given, or the lead's own after it has passed the advisor gate.

## Master checklist — work through it in order, tick every box

Copy this into the tmp dir as `checklist.md` at Stage 0 and keep it ticked as you go. Nothing is "done" until every box is ticked. An unticked box at the end = the pipeline did not run, no matter how good the code looks. A box marked SKIP is ticked as `SKIPPED` only when its stated condition holds.

```
[ ] 0   Task read, code read, shape understood
[ ] 0   INDEX.md read, skill loadout chosen per stage
[ ] 0   Brief written (self-contained — subagents share no memory)
[ ] 0   Baseline recorded in <tmp>/rex-code-lead/<task-slug>/baseline.md
[ ] 0   Plan in <tmp>/rex-code-lead/<task-slug>/plan.md — given plan copied verbatim, else the lead's own
[ ] 0   Test list settled — every durable test named with its reason, smallest set that proves "done"
[ ] 0b  Advisor dispatched (fresh subagent, opus/inherit)             — SKIP when the plan was given
[ ] 0b  Verdict APPROVE in review.md — REVISE loops back, max 3 rounds — SKIP when the plan was given
[ ] 1   Engineer dispatched with brief + approved plan + must-haves + extras + rex-code-tdd
[ ] 1   Engineer reported; build green, tests green
[ ] 1   LEAD GATE — report read, diff skimmed, brief fulfilled
[ ] 2   Auditor dispatched (FRESH subagent, opus/inherit) with rex-code-smells + rex-code-improve-codebase-architecture
[ ] 2   Smell sweep run, findings fixed or flagged with permanent SMELL comments
[ ] 2   Test set audited against the test list — unplanned, redundant, wrong-lane tests deleted
[ ] 2   LEAD GATE — nothing fundamental; fundamental -> back to Stage 1
[ ] 3   Cleaner dispatched (FRESH subagent, opus) with rex-code-smells + rex-code-commenting
[ ] 3   Final rex-code-smells sweep run at opus over every changed file
[ ] 3   Sweep findings fixed (behavior-preserving); unfixable flagged with permanent SMELL comments
[ ] 3   Scratch unit tests removed; durable lanes and pre-existing tests untouched
[ ] 3   Comments rewritten to plain English; smell/hazard/SAFETY flags all intact
[ ] 3   LEAD GATE — zero behavior change, zero deleted flags
[ ] F   build green
[ ] F   clippy — no new warnings
[ ] F   full test suite green; pre-existing tests intact; tests added = the test list, nothing extra
[ ] F   final diff skimmed
[ ] F   nothing committed or pushed — unless specifically asked, then done exactly as asked
[ ] F   report written to user, tests added + remaining flagged smells listed
```

## How subagents get skills

Skills live in `.claude/skills/` as files. They are NOT registered slash-skills. Equip a subagent by listing the exact `SKILL.md` paths in its prompt with the instruction: **"Read each of these files IN FULL before touching any code. They are hard rules, not suggestions."**

Consult `.claude/skills/INDEX.md` to pick skills. Rule: equip each subagent with ALL skills relevant to the task at hand — when in doubt, include. One exception: the test-lane skills (integration, fitness, contract) go to the engineer only for lanes on the test list — a loaded lane skill reads as an invitation to write tests in that lane.

**Absolute must-haves for EVERY coding subagent. All three stages. No exceptions:**

- `.claude/skills/rex-code-philosophy/SKILL.md`
- `.claude/skills/rex-code-ergonomics/SKILL.md`
- `.claude/skills/rex-code-commenting/SKILL.md`

## Test budget — few tests, each one earns its place

Default answer to "should this test still exist after the run?" is NO. A test survives only when it is on the test list. The budget is pasted into every coding subagent's prompt.

- **The test list is the complete set of tests that survive the run.** It lives in the plan (or in the brief, when a given plan names no tests). Each entry: file, test name, lane, and one line on what would break unnoticed without it. No entry, no test.
- **Smallest set that proves the brief's "done" criteria.** One test per behavior the task adds or changes, through the public interface. No test per function, no test per edge case, no near-duplicate inputs, no tests for code the task did not touch.
- **Lanes are not a quota.** A lane the task does not need stays empty. Most tasks add ZERO fitness functions — add one only when the task introduces an invariant that passes all five conditions in `rex-code-tests-fitness-functions`. Contract test only when the task creates or changes a seam between modules. Integration test only when the task adds or changes a path to the outside world.
- **Extend before adding.** An existing test that already walks the path gets one more case or assertion. A new test, and above all a new test file, is the last resort.
- **Inline `#[cfg(test)]` scaffolds are temporary.** They drive TDD and the cleaner deletes them. A changed behavior that fits no durable lane keeps ONE inline test; the rest of its scaffold goes.
- **Tests that existed before the run are off-limits.** The budget governs what this run adds. No stage deletes or weakens a pre-existing test unless the task itself removes the feature under it.

## No commits unless asked

The pipeline leaves all work uncommitted in the working tree. No `git commit`, no `git push`, no pull request — from the lead or from any subagent — unless the user or the calling agent specifically asked for it, in the task or during the run. Finishing the pipeline is not a request to commit. A request for one does not cover the others: asked to commit is not asked to push.

- Every dispatch prompt carries the words: **"Do NOT run git commit or git push, and do NOT open a pull request. Leave every change uncommitted in the working tree."** Subagents never commit, even when the lead was asked to.
- Asked to commit -> the lead does it, after the final gate is green, unless the request says otherwise.
- Not asked -> the final report says the work is uncommitted.

## Dispatch checklist — every stage, every redispatch

Tick all ten before you send ANY subagent prompt. This is where things get missed.

```
[ ] Fresh subagent (Stages 0b, 2, 3 — never reuse the one that did the previous stage)
[ ] Model set explicitly at dispatch (see table below)
[ ] Brief pasted in FULL — subagents share no memory, nothing is implied
[ ] Plan path + baseline path + prior stage reports included
[ ] Exact SKILL.md paths listed, must-haves first
[ ] The words "Read each of these files IN FULL before touching any code. They are hard rules, not suggestions."
[ ] Stage-specific instructions copied from the stage section below
[ ] Test budget section pasted in (coding stages), with the test list
[ ] The words "Do NOT run git commit or git push, and do NOT open a pull request. Leave every change uncommitted in the working tree."
[ ] Report-back shape stated, and "build + tests green before you report"
```

## Model per stage

When running on Claude models, set the subagent model at dispatch:

| Stage | Model |
|---|---|
| 0b. Advisor | `opus` — or inherit the lead's model |
| 1. Engineer | `sonnet` |
| 2. Auditor | `opus` — or inherit the lead's model |
| 3. Cleaner | `opus` — always. The final smell sweep runs here and is the last chance to catch anything. |

On other model families, map to the equivalent tiers: a fast capable model writes, the strongest available model audits and cleans.

## Stage 0 — Lead scopes, and plans only when no plan was given

1. Read the task. Read the relevant code. Understand the shape of the change.
2. Read `.claude/skills/INDEX.md`. Decide the skill loadout for each stage (must-haves + relevant extras).
3. Write a short brief: what to build, constraints, files in play, what "done" means. Every subagent prompt starts from this brief — subagents share no memory, so the brief carries ALL context.
4. **Record the baseline** in `<tmp>/rex-code-lead/<task-slug>/baseline.md`: the output of `git rev-parse HEAD` and `git status --short`. Nothing gets committed during the run, so "what this run changed" and "tests this run added" = everything that differs from the baseline.
5. **Settle the plan — given or written.** The tmp dir is the harness scratchpad or system temp, NOT the repo. One dir per task: `<tmp>/rex-code-lead/<task-slug>/`.
   - **A plan was given** — the user or the calling agent supplied an implementation plan: inline in the task, as a file path, or approved earlier in the conversation. Do NOT write a new plan and do NOT run the advisor. Copy it verbatim to `plan.md`. Never rewrite it. A gap the engineer would have to guess through -> answer it in the brief as a lead note when the code answers it, otherwise ask whoever gave the plan. A given plan that names no tests -> the lead writes the test list into the brief, per the test budget.
   - **No plan was given** — write `plan.md`. It covers: approach and why over the obvious alternative, files to create/change, module boundaries touched, the test list per the test budget, dependencies to add, risks, and the "done" criteria. Concrete enough that the engineer never has to guess.
   - A plan says HOW: approach and files to change. A task description, feature request, or design brief says WHAT — that is not a plan, so the lead writes one.
   - Either way, `plan.md` is "the approved plan" for every later stage.
6. **Copy the master checklist to `<tmp>/rex-code-lead/<task-slug>/checklist.md`.** Tick boxes as stages complete. It is the lead's memory — without it, a stage gets skipped.

## Stage 0b — Advisor gate (only when the lead wrote the plan)

**Plan was given -> skip this stage entirely.** Mark both 0b boxes `SKIPPED — plan given` and go to Stage 1. The advisor checks the lead's planning, not a decision the user already made.

Dispatch a FRESH subagent (model: `opus` or inherit on Claude) with the brief + the plan path + these skills:

- `.claude/skills/rex-code-philosophy/SKILL.md` — the over-engineering / worse-is-better lens.
- `.claude/skills/rex-code-ergonomics/SKILL.md` — if the plan shapes any API surface.
- Task-relevant extras from INDEX.md (e.g. the test skills to check test placement, `rex-code-improve-codebase-architecture` for boundary changes).

Advisor instructions:
- Read the plan file. Read the actual code it talks about. Sense-check, do not rubber-stamp.
- Hunt for: over-engineering, missing requirements, wrong module boundaries, tests planned in the wrong lane, a test list bigger than the "done" criteria need, a lane used with no reason, needless dependencies, risks with no mitigation, vagueness the engineer would have to guess through.
- Write the verdict to `<tmp>/rex-code-lead/<task-slug>/review.md`: APPROVE, or REVISE with numbered, actionable objections.
- The advisor reviews the plan only. It writes no code and does not rewrite the plan.

Lead gate: REVISE -> fix the plan, redispatch the advisor. Loop until APPROVE, max 3 rounds — still no approval after 3 means the task is unclear: stop and take the open questions back to the user. Only an APPROVED plan advances to Stage 1.

## Stage 1 — Software Engineer (writes the code)

Dispatch a subagent (model: `sonnet` on Claude) with the brief + the approved plan path + these skills:

Must-haves, plus:
- `.claude/skills/rex-code-error-writing/SKILL.md`
- `.claude/skills/rex-code-external-libraries/SKILL.md`
- `.claude/skills/rex-code-tests-unit-testing/SKILL.md`
- Working method — this is HOW the engineer works, not optional: `.claude/skills/rex-code-tdd/SKILL.md` (red-green-refactor)
- Test-lane skills ONLY for lanes on the test list: `.claude/skills/rex-code-tests-integration-testing/SKILL.md`, `.claude/skills/rex-code-tests-fitness-functions/SKILL.md`, `.claude/skills/rex-code-tests-contract-seams/SKILL.md`. Lane not on the list -> skill not loaded.
- Task-relevant extras from INDEX.md, e.g. `rex-code-improve-codebase-architecture` for refactors, `rex-mermaid-diagrams` when diagrams asked for.

Engineer instructions:
- Read the approved plan first. Work from it. A deviation that turns out necessary is fine — but it must be named and justified in the report, not slipped in silently.
- TDD the implementation. Inline `#[cfg(test)]` scaffolds are expected and fine — the cleaner deals with them later. Do NOT delete them yourself.
- Durable tests: write exactly the ones on the test list, in the lanes it names. Nothing else lands in `tests/`. A durable test the list missed and the work proved necessary -> add it and name it in the report with one line on what would break unnoticed without it. No test per function, no extra edge-case tests, no tests for code the task did not touch.
- Build + tests green before reporting.
- Report back: what was built, decisions made, trade-offs taken, known weak spots, every test added (durable or scaffold).

Lead gate: read the report, skim the diff. Brief unfulfilled or build red -> redispatch with corrections. Otherwise advance.

## Stage 2 — Auditor (checks the work)

Dispatch a FRESH subagent (never the engineer; model: `opus` or inherit on Claude) with the brief + approved plan path + engineer's report + these skills:

Must-haves, plus:
- `.claude/skills/rex-code-error-writing/SKILL.md`
- `.claude/skills/rex-code-tdd/SKILL.md`
- `.claude/skills/rex-code-tests-unit-testing/SKILL.md`
- `.claude/skills/rex-code-tests-integration-testing/SKILL.md`
- `.claude/skills/rex-code-tests-fitness-functions/SKILL.md`
- `.claude/skills/rex-code-smells/SKILL.md` — always for the auditor; the detection playbook that drives the smell hunt.
- `.claude/skills/rex-code-improve-codebase-architecture/SKILL.md` — always for the auditor; module shape and integration-fit review.
- Same task-relevant extras the engineer got.

Auditor instructions:
- Audit the change against the approved plan AND every loaded skill. Unjustified plan deviations are findings. Correctness first, then ergonomics, error handling, test placement, comment discipline.
- **Run the full `rex-code-smells` sweep** over every changed file — all three lenses, the mechanical sweep block, and the clippy pass. Report in that skill's table format with BLOCK/REFACTOR/NIT severities.
- **Audit the test set against the test budget.** Every durable test this run added must be on the test list or justified in the engineer's report. Delete the ones that are not, plus any that repeat another test's coverage, pin a single function's internals, or sit in a lane whose bar they do not meet — a fitness test failing any of that skill's five conditions is not a fitness test. Never touch a test that existed before the run. A "done" criterion with no test proving it is a finding too.
- Fix what can be fixed now. Smell that cannot be fixed now -> flag with a short plain-English `SMELL:` comment per `rex-code-commenting` whitelist #7. These flags are permanent — later passes may not remove them.
- Build + tests green after fixes.
- Report back: the smell table, what was fixed, what was flagged, tests deleted (and why), anything fundamental.

Lead gate: fundamental problems (wrong approach, missing requirement, broken design) -> back to Stage 1 with the audit findings. Durable tests beyond the test list still standing with no justification -> redispatch the auditor. Clean or fixed-in-place -> advance.

```
[ ] Fresh subagent, not the engineer
[ ] rex-code-smells sweep run over every changed file, all three lenses
[ ] Findings table returned with severities, or an explicit "clean" verdict
[ ] Every BLOCK fixed or escalated to the lead
[ ] Unfixable smells carry permanent SMELL comments
[ ] Durable tests added by the run = the test list (+ justified additions); surplus deleted
[ ] Pre-existing tests untouched
[ ] Build + tests green
```

## Stage 3 — Cleaner (final smell sweep + fixes + polish, zero behavior change)

Dispatch a FRESH subagent (model: `opus` — this stage is opus even if the lead is running smaller) with the brief + prior reports + these skills:

Must-haves (updated `rex-code-commenting` is the centerpiece), plus:
- `.claude/skills/rex-code-smells/SKILL.md` — the final sweep runs here, at opus, after all fixes have landed.
- `.claude/skills/rex-code-tests-unit-testing/SKILL.md` — removal criteria live here.
- `.claude/skills/rex-code-tests-fitness-functions/SKILL.md` — so it knows what is untouchable.

Cleaner instructions:
- **Run the `rex-code-smells` sweep one last time** over every changed file, after the cleanup edits. The auditor swept mid-flight; this one sweeps the code that actually ships, including anything the audit fixes introduced. All three lenses plus the mechanical block.
- **Fix what the sweep finds.** Fixes must be behavior-preserving — renames, extractions, and signature tidy-ups are allowed when a finding calls for them, feature changes are not. Build + full test suite green after each fix. A finding that cannot be fixed behavior-preservingly gets a permanent plain-English `SMELL:` comment and goes in the report. A BLOCK-severity finding goes to the lead immediately — the cleaner does not redesign its way out of a fundamental problem.
- **Remove the scratch unit tests this run added** — every inline `#[cfg(test)]` TDD scaffold not in the baseline goes. The "stable for two weeks" removal criterion in `rex-code-tests-unit-testing` does not apply inside this pipeline — the audit stands in for it. Keep an inline test only when it is on the test list, or when it asserts something load-bearing that no durable test covers — then keep ONE per behavior and say so in the report. NEVER touch `tests/fitness/`, `tests/integration/`, `tests/contract/`, or any inline test that existed before the run.
- **Rewrite comments to short plain English.** Full grammatical sentences a newcomer understands first time. Delete WHAT-comments, stale comments, commented-out code, AI noise, plan and task references, pointers to other files, and jargon — per `rex-code-commenting`.
- **NEVER delete code-smell, warning, hazard, or `SAFETY:` comments.** ALWAYS included. Tighten wording only. This is the hardest rule in this stage.
- Zero behavior change. Smell fixes stay behavior-preserving; beyond them, no "improvements" — cleanup only.
- Build after every file touched. Full test suite green at the end.
- Report back: the final smell table, what was fixed, what was flagged unfixable, tests removed (and why safe), tests kept (and why), comments rewritten/deleted, flags preserved.

Lead gate: any behavior change or deleted flag -> reject, redispatch to restore. A BLOCK finding in the final sweep -> back to Stage 1 with that finding; the cleaner never fixes it itself.

```
[ ] Fresh subagent, model opus
[ ] Final rex-code-smells sweep run AFTER the cleanup edits
[ ] Every sweep finding fixed (behavior-preserving), or flagged with a permanent SMELL comment; BLOCK -> lead
[ ] Scratch #[cfg(test)] scaffolds added by the run removed; any kept one is named with its reason
[ ] tests/fitness, tests/contract, tests/integration and pre-existing inline tests untouched
[ ] Comments plain English — no jargon, no plan references, no file pointers
[ ] Every SMELL / warning / hazard / SAFETY flag still present
[ ] Zero behavior change; build + full suite green
```

## Final gate — Lead verifies

1. `cargo build` (or project equivalent) — green.
2. `cargo clippy` — no new warnings.
3. Full test suite — green. Every test in the baseline still present and passing. Tests added by the run = the test list plus justified additions, nothing extra — a surplus test goes back to the auditor, an empty lane is fine.
4. Skim final diff: plan and brief fulfilled (deviations justified), comments plain English, smell flags intact.
5. Nothing committed or pushed — unless the user or the calling agent specifically asked, and then exactly what was asked.
6. Every box in the master checklist ticked. An unticked box means the pipeline is not finished — go back and run that step, do not report around it.

Then report to the user: what was built, what the audit changed, what the cleanup removed, the tests the run added (each with its one-line reason), the remaining flagged smells from the final sweep with their severities, and whether the work is committed or sitting uncommitted in the working tree.

## Optional stage — Codex adversarial review (opt-in ONLY)

Run this stage ONLY when the user has explicitly asked for it — in the task itself or during the run. It is NOT part of the default pipeline and the lead never triggers it unprompted.

When asked: after the final gate is green, follow `.claude/skills/rex-codex-adversarial-review/SKILL.md` over the final diff — Codex challenges the approach in a read-only sandbox, the lead cross-examines its findings, one rebuttal round — and append its verdict table to the report.

## Hard rules for the lead

- No code before a plan. Ever. A given plan is used as-is; the lead's own plan must be advisor-APPROVED. Plan and review live in the tmp dir, never the repo.
- Plan only when no plan was given. A given plan is never rewritten, re-planned, or sent to the advisor.
- Never skip a stage — the single exception is Stage 0b when the plan was given. Never merge stages. Never let one subagent do two stages.
- The advisor advises on the plan only — it never writes code and never rewrites the plan itself.
- Fresh subagent per stage — auditors do not audit their own code, cleaners do not clean their own comments.
- The three must-have skills go to every coding subagent, every dispatch, including redispatches.
- Subagents share no memory. Every dispatch prompt is self-contained: brief + prior reports + skill paths.
- `rex-code-smells` runs TWICE — the auditor sweeps and fixes, the cleaner sweeps again at opus and fixes what it finds (behavior-preserving; unfixable -> permanent SMELL flag, BLOCK -> lead). Skipping the second sweep is skipping a stage.
- Few tests. Only tests on the test list survive the run. Test lanes are not a quota — an empty lane is correct when the task does not need it. Pre-existing tests are never deleted to make the numbers look better.
- No `git commit`, no `git push`, no pull request — lead or subagent — unless the user or the calling agent specifically asked. The words go in every dispatch prompt.
- Keep `checklist.md` ticked as you go. Untick nothing, skip nothing, and never report done with a box open.
- Codex adversarial review is opt-in — run it only on explicit user request, after the final gate, never by default.
- You never edit code. Your tools are dispatch, review, verification, and the final report.
