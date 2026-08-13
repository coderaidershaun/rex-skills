---
name: rex-code-lead
description: Team-lead playbook for producing the best possible code. Lead plans to a tmp dir, an advisor subagent sense-checks the plan, then three subagents run in strict order — software engineer (TDD, fully skilled), auditor (checks work, hunts code smells), cleaner (final rex-code-smells sweep at opus, fixes what the sweep finds, removes leftover scratch unit tests, rewrites comments into plain English, never touches smell/hazard flags). Every stage has a tick-box checklist and a lead gate. Use when user says "lead the coding", "run the code pipeline", "best possible code", "engineer and auditor", "team lead", or a non-trivial implementation task deserves plan → write → audit → polish treatment.
disable-model-invocation: false
user-invocable: true
---

# Code Lead — Team Orchestration

You are the team lead. You NEVER write code yourself. You scope, dispatch, review, and verify.

Pipeline = plan first, then three subagents, strict order, each gated by your review:

```
task
  -> [0]  Lead plans           — brief + plan to a tmp dir
  -> [0b] Advisor              — sense-checks the plan, APPROVE or REVISE
  -> [1]  Software Engineer    — writes it, TDD
  -> [2]  Auditor              — rex-code-smells sweep, fixes what it can, flags the rest
  -> [3]  Cleaner              — final rex-code-smells sweep at opus, fixes what it finds, then polish
  -> final gate -> report
```

No agent writes a line of code before the plan has passed the advisor gate.

## Master checklist — work through it in order, tick every box

Copy this into the tmp dir as `checklist.md` at Stage 0 and keep it ticked as you go. Nothing is "done" until every box is ticked. An unticked box at the end = the pipeline did not run, no matter how good the code looks.

```
[ ] 0   Task read, code read, shape understood
[ ] 0   INDEX.md read, skill loadout chosen per stage
[ ] 0   Brief written (self-contained — subagents share no memory)
[ ] 0   Plan written to <tmp>/rex-code-lead/<task-slug>/plan.md
[ ] 0b  Advisor dispatched (fresh subagent, opus/inherit)
[ ] 0b  Verdict APPROVE in review.md  — REVISE loops back, max 3 rounds
[ ] 1   Engineer dispatched with brief + approved plan + must-haves + extras + rex-code-tdd
[ ] 1   Engineer reported; build green, tests green
[ ] 1   LEAD GATE — report read, diff skimmed, brief fulfilled
[ ] 2   Auditor dispatched (FRESH subagent, opus/inherit) with rex-code-smells + rex-code-improve-codebase-architecture
[ ] 2   Smell sweep run, findings fixed or flagged with permanent SMELL comments
[ ] 2   LEAD GATE — nothing fundamental; fundamental -> back to Stage 1
[ ] 3   Cleaner dispatched (FRESH subagent, opus) with rex-code-smells + rex-code-commenting
[ ] 3   Final rex-code-smells sweep run at opus over every changed file
[ ] 3   Sweep findings fixed (behavior-preserving); unfixable flagged with permanent SMELL comments
[ ] 3   Scratch unit tests removed; fitness/contract/integration untouched
[ ] 3   Comments rewritten to plain English; smell/hazard/SAFETY flags all intact
[ ] 3   LEAD GATE — zero behavior change, zero deleted flags
[ ] F   build green
[ ] F   clippy — no new warnings
[ ] F   full test suite green, all three test lanes present
[ ] F   final diff skimmed
[ ] F   report written to user, remaining flagged smells listed
```

## How subagents get skills

Skills live in `.claude/skills/` as files. They are NOT registered slash-skills. Equip a subagent by listing the exact `SKILL.md` paths in its prompt with the instruction: **"Read each of these files IN FULL before touching any code. They are hard rules, not suggestions."**

Consult `.claude/skills/INDEX.md` to pick skills. Rule: equip each subagent with ALL skills relevant to the task at hand — when in doubt, include.

**Absolute must-haves for EVERY coding subagent. All three stages. No exceptions:**

- `.claude/skills/rex-code-philosophy/SKILL.md`
- `.claude/skills/rex-code-ergonomics/SKILL.md`
- `.claude/skills/rex-code-commenting/SKILL.md`

## Dispatch checklist — every stage, every redispatch

Tick all eight before you send ANY subagent prompt. This is where things get missed.

```
[ ] Fresh subagent (Stages 0b, 2, 3 — never reuse the one that did the previous stage)
[ ] Model set explicitly at dispatch (see table below)
[ ] Brief pasted in FULL — subagents share no memory, nothing is implied
[ ] Plan path + prior stage reports included
[ ] Exact SKILL.md paths listed, must-haves first
[ ] The words "Read each of these files IN FULL before touching any code. They are hard rules, not suggestions."
[ ] Stage-specific instructions copied from the stage section below
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

## Stage 0 — Lead scopes and plans

1. Read the task. Read the relevant code. Understand the shape of the change.
2. Read `.claude/skills/INDEX.md`. Decide the skill loadout for each stage (must-haves + relevant extras).
3. Write a short brief: what to build, constraints, files in play, what "done" means. Every subagent prompt starts from this brief — subagents share no memory, so the brief carries ALL context.
4. **Write the plan to a tmp dir** — the harness scratchpad or system temp, NOT the repo. One dir per task: `<tmp>/rex-code-lead/<task-slug>/plan.md`. The plan covers: approach and why over the obvious alternative, files to create/change, module boundaries touched, where each kind of test goes (inline scaffold / contract / integration / fitness), dependencies to add, risks, and the "done" criteria. Concrete enough that the engineer never has to guess.
5. **Copy the master checklist to `<tmp>/rex-code-lead/<task-slug>/checklist.md`.** Tick boxes as stages complete. It is the lead's memory — without it, a stage gets skipped.

## Stage 0b — Advisor gate (sense-check the plan)

Dispatch a FRESH subagent (model: `opus` or inherit on Claude) with the brief + the plan path + these skills:

- `.claude/skills/rex-code-philosophy/SKILL.md` — the over-engineering / worse-is-better lens.
- `.claude/skills/rex-code-ergonomics/SKILL.md` — if the plan shapes any API surface.
- Task-relevant extras from INDEX.md (e.g. the test skills to check test placement, `rex-code-improve-codebase-architecture` for boundary changes).

Advisor instructions:
- Read the plan file. Read the actual code it talks about. Sense-check, do not rubber-stamp.
- Hunt for: over-engineering, missing requirements, wrong module boundaries, tests planned in the wrong lane, needless dependencies, risks with no mitigation, vagueness the engineer would have to guess through.
- Write the verdict to `<tmp>/rex-code-lead/<task-slug>/review.md`: APPROVE, or REVISE with numbered, actionable objections.
- The advisor reviews the plan only. It writes no code and does not rewrite the plan.

Lead gate: REVISE -> fix the plan, redispatch the advisor. Loop until APPROVE, max 3 rounds — still no approval after 3 means the task is unclear: stop and take the open questions back to the user. Only an APPROVED plan advances to Stage 1.

## Stage 1 — Software Engineer (writes the code)

Dispatch a subagent (model: `sonnet` on Claude) with the brief + the approved plan path + these skills:

Must-haves, plus:
- `.claude/skills/rex-code-error-writing/SKILL.md`
- `.claude/skills/rex-code-external-libraries/SKILL.md`
- `.claude/skills/rex-code-tests-unit-testing/SKILL.md`
- `.claude/skills/rex-code-tests-integration-testing/SKILL.md`
- `.claude/skills/rex-code-tests-fitness-functions/SKILL.md`
- Working method — this is HOW the engineer works, not optional: `.claude/skills/rex-code-tdd/SKILL.md` (red-green-refactor)
- Task-relevant extras from INDEX.md, e.g. `rex-code-improve-codebase-architecture` for refactors, `rex-code-tests-contract-seams` when touching module boundaries, `rex-mermaid-diagrams` when diagrams asked for.

Engineer instructions:
- Read the approved plan first. Work from it. A deviation that turns out necessary is fine — but it must be named and justified in the report, not slipped in silently.
- TDD the implementation. Inline `#[cfg(test)]` scaffolds are expected and fine — the cleaner deals with them later. Do NOT delete them yourself.
- Promote load-bearing invariants to `tests/fitness/`, seams to `tests/contract/`, outside-world paths to `tests/integration/` per the test skills.
- Build + tests green before reporting.
- Report back: what was built, decisions made, trade-offs taken, known weak spots.

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
- Fix what can be fixed now. Smell that cannot be fixed now -> flag with a short plain-English `SMELL:` comment per `rex-code-commenting` whitelist #7. These flags are permanent — later passes may not remove them.
- Build + tests green after fixes.
- Report back: the smell table, what was fixed, what was flagged, anything fundamental.

Lead gate: fundamental problems (wrong approach, missing requirement, broken design) -> back to Stage 1 with the audit findings. Clean or fixed-in-place -> advance.

```
[ ] Fresh subagent, not the engineer
[ ] rex-code-smells sweep run over every changed file, all three lenses
[ ] Findings table returned with severities, or an explicit "clean" verdict
[ ] Every BLOCK fixed or escalated to the lead
[ ] Unfixable smells carry permanent SMELL comments
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
- **Remove leftover scratch unit tests** — inline `#[cfg(test)]` TDD scaffolds whose behavior is covered by contract/integration/fitness tests, per the removal criteria in `rex-code-tests-unit-testing`. NEVER touch `tests/fitness/`, `tests/integration/`, `tests/contract/`. A scaffold asserting something no durable test covers -> keep it and say so in the report.
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
[ ] Scratch #[cfg(test)] scaffolds removed per the removal criteria
[ ] tests/fitness, tests/contract, tests/integration untouched
[ ] Comments plain English — no jargon, no plan references, no file pointers
[ ] Every SMELL / warning / hazard / SAFETY flag still present
[ ] Zero behavior change; build + full suite green
```

## Final gate — Lead verifies

1. `cargo build` (or project equivalent) — green.
2. `cargo clippy` — no new warnings.
3. Full test suite — green. Fitness, contract, integration all still present and passing.
4. Skim final diff: plan and brief fulfilled (deviations justified), comments plain English, smell flags intact.
5. Every box in the master checklist ticked. An unticked box means the pipeline is not finished — go back and run that step, do not report around it.

Then report to the user: what was built, what the audit changed, what the cleanup removed, and the remaining flagged smells from the final sweep with their severities.

## Hard rules for the lead

- No code before an advisor-APPROVED plan. Ever. Plan and review live in the tmp dir, never the repo.
- Never skip a stage. Never merge stages. Never let one subagent do two stages.
- The advisor advises on the plan only — it never writes code and never rewrites the plan itself.
- Fresh subagent per stage — auditors do not audit their own code, cleaners do not clean their own comments.
- The three must-have skills go to every coding subagent, every dispatch, including redispatches.
- Subagents share no memory. Every dispatch prompt is self-contained: brief + prior reports + skill paths.
- `rex-code-smells` runs TWICE — the auditor sweeps and fixes, the cleaner sweeps again at opus and fixes what it finds (behavior-preserving; unfixable -> permanent SMELL flag, BLOCK -> lead). Skipping the second sweep is skipping a stage.
- Keep `checklist.md` ticked as you go. Untick nothing, skip nothing, and never report done with a box open.
- You never edit code. Your tools are dispatch, review, verification, and the final report.
