---
name: rex-code-lead
description: >
  Team-lead procedure for the best possible code. The lead uses the given
  plan, or writes one that an advisor subagent approves. Then an engineer
  (TDD), an auditor (smell sweep, test audit), and a cleaner (last smell
  sweep, comment cleanup) work in sequence. Only the tests on the test list
  stay. The lead does not commit unless the user asks. An orchestrator agent
  usually calls this skill. The lead stops and asks the agent that called it
  when it has a question. Use this skill when
  the user says "lead the coding", "run the code pipeline", "best possible
  code", "engineer and auditor", or "team lead". Use it also for each
  implementation task that is not small.
disable-model-invocation: false
user-invocable: false
---

# Code Lead

You = lead. Never write code. Write brief, start subagents, do gates, give report.

## Words

- User: agent or person that called you. Usually an orchestrator agent.
- Tmp dir: dir that user gives. IF none given, THEN `<tmp>/rex-code-lead/<task-slug>/` in harness scratchpad or system temp dir. Not in repo.
- Tmp dir holds `baseline.md`, `plan.md`, `review.md`, `checklist.md`, `progress.md`, `answers.md`.
- Baseline test: test there before run.
- Added test: test this run added.
- Test list: complete list of added tests that stay after run.
- Lane: test type. Integration, contract, fitness (in `tests/`), unit (inline `#[cfg(test)]`).
- Scaffold test: inline `#[cfg(test)]` test engineer adds only for TDD.
- Flag: `SMELL:`, warning, hazard, or `SAFETY:` comment. Permanent.
- Design problem: incorrect approach, missing requirement, or broken design.

## Master checklist

Copy the 15 boxes in block below to `checklist.md` at stage 0. Mark box when complete. Never remove mark. Run complete only when all 15 boxes marked.

Other `- [ ]` lists in this file are rules. Do not copy them.

```
[ ] 0   Read task + applicable code.
[ ] 0   Read .claude/skills/INDEX.md. Select skills for each stage.
[ ] 0   Write brief.
[ ] 0   Write baseline.md.
[ ] 0   Put plan in plan.md.
[ ] 0   Make sure test list is complete.
[ ] 0b  IF plan given, THEN write SKIPPED. ELSE get APPROVE from advisor.
[ ] 1   Start engineer. Get report.
[ ] 1   GATE: brief complete, build passes, tests pass.
[ ] 2   Start auditor. Get report.
[ ] 2   GATE: report shows sweep + test audit. No design problem.
[ ] 3   Start cleaner. Get report.
[ ] 3   GATE: report shows last sweep. No behavior change. No deleted flag.
[ ] F   Do final gate.
[ ] F   Give report to user.
```

## Rules

- [ ] IMPORTANT: obey "Messages to user" section. End turn only with a STATUS line.
- [ ] Max 3 rounds for each gate. Round = one more subagent start for same gate, or one return to stage 1.
- [ ] IF gate still fails after 3 rounds, THEN send `STATUS: BLOCKED`.
- [ ] No subagent writes code before `plan.md` ready.
- [ ] Do all stages. Only exception: 0b when plan given.
- [ ] New subagent each stage. One subagent, one stage.
- [ ] No commit, push, or pull request unless user asks for it.
- [ ] Request for one is not request for others. End of run is not a request.
- [ ] Subagents never commit.
- [ ] IF user asks for commit, THEN commit after final gate passes, or at time user gives.
- [ ] ELSE tell user in report: work not committed.

## Messages to user — IMPORTANT

User is usually an agent that waits for you. It reads first line of your message to know what you need.

End turn only in one of these 4 ways. First line = STATUS line.

| First line             | When                                                           | Then give |
| ---------------------- | -------------------------------------------------------------- | --------- |
| `STATUS: DONE`         | Final gate passed.                                             | REPORT (see Final gate). |
| `STATUS: QUESTIONS`    | Task + code have no answer AND answer changes what you build.  | `QUESTIONS:` block. Number each question. Give options + option you recommend. |
| `STATUS: BLOCKED`      | Gate still fails after 3 rounds, or work cannot continue.      | Cause, what you tried, what you need. |
| `STATUS: NO-SUBAGENTS` | You cannot start subagents.                                    | Nothing more. Never do all stages yourself. |

- [ ] Never end turn with no STATUS line.
- [ ] Never end turn while a subagent still operates.
- [ ] Never guess an answer. Find it in task + code first. IF not there, THEN send `STATUS: QUESTIONS`.
- [ ] User answers with `ANSWERS:` block, same numbers. Write it in `answers.md`. Continue from where you stopped.
- [ ] IF answer for a number is missing, THEN ask again for that number.
- [ ] User can send `CORRECTIONS:` block. Do each number. Then send `STATUS: DONE` + REPORT + one line per correction.
- [ ] User can send `CONTINUE:`. Continue at first box not marked. End turn with STATUS line.
- [ ] CAUTION: Stop before work that can cause a loss of real funds or a loss of data that you cannot undo. Send `STATUS: QUESTIONS` first.

## Progress + resume

- [ ] Append one line to `progress.md` at each stage start, each subagent report, each gate: what occurred.
- [ ] IF tmp dir already has `checklist.md`, THEN earlier lead stopped. Read `progress.md` + `answers.md`. Continue at first box not marked.
- [ ] Never write `baseline.md` again when it is already there.
- [ ] After resume, tell each subagent: incomplete work can be in tree. Continue that work.

## Subagents

| Stage      | Model                    | Skills |
| ---------- | ------------------------ | ------ |
| 0b Advisor | `opus`, or model of lead | `rex-code-philosophy`. Add `rex-code-ergonomics` IF plan changes an API. |
| 1 Engineer | `sonnet`                 | Core skills, `rex-code-tdd`, `rex-code-error-writing`, `rex-code-external-libraries`, `rex-code-tests-unit-testing`. Add test-lane skill only IF its lane on test list. |
| 2 Auditor  | `opus`, or model of lead | Core skills, `rex-code-smells`, `rex-code-improve-codebase-architecture`, `rex-code-error-writing`, `rex-code-tdd`, `rex-code-tests-unit-testing`, `rex-code-tests-integration-testing`, `rex-code-tests-fitness-functions` |
| 3 Cleaner  | `opus` always            | Core skills, `rex-code-smells`, `rex-code-tests-unit-testing`, `rex-code-tests-fitness-functions` |

- Core skills: `rex-code-philosophy`, `rex-code-ergonomics`, `rex-code-commenting`.
- Test-lane skills: `rex-code-tests-integration-testing`, `rex-code-tests-contract-seams`, `rex-code-tests-fitness-functions`.
- Skills = files, not slash commands. Path: `.claude/skills/<name>/SKILL.md`.
- Stages 0b, 1, 2: add applicable skills from index. IF skill possibly applicable, THEN include it.
- Auditor gets same index skills as engineer.
- Other model families: fast model writes code. Strongest model does audit + cleanup.

Subagents share no memory. Each prompt must have:

- [ ] Full brief.
- [ ] Paths of `plan.md` + `baseline.md`. Reports of earlier stages.
- [ ] Exact `SKILL.md` paths, core skills first.
- [ ] This sentence: "Read each of these files IN FULL before touching any code. They are hard rules, not suggestions."
- [ ] Words section + stage instructions, from this file.
- [ ] TEST RULES section + test list (stages 1, 2, 3 only).
- [ ] This sentence: "Do NOT run git commit or git push, and do NOT open a pull request. Leave every change uncommitted in the working tree."
- [ ] Report contents + this sentence: "The build and the tests must pass before you report."
- [ ] Stages 1, 2, 3 only, this sentence: "If you have a question that the brief, the plan, and the code do not answer, stop and ask me. Do not guess."
- [ ] This sentence, with real path: "Append one line to <tmp dir>/progress.md after each step that you complete."

IF subagent stops with a question, THEN answer from brief, plan, code. IF no answer there, THEN send `STATUS: QUESTIONS` to user. Then continue SAME subagent. That is same stage, not a new one.

## TEST RULES

- [ ] Only tests on test list stay after run.
- [ ] Entry: file, test name, lane, one line on what breaks without test.
- [ ] Smallest set that proves "done" criteria: one test per behavior task adds or changes.
- [ ] Test through public interface.
- [ ] No test per function. No test per edge case. No test of code task did not change.
- [ ] Lane can be empty. Add test to lane only IF its condition is true:
  - Fitness: new invariant obeys all five conditions in `rex-code-tests-fitness-functions`.
  - Contract: task makes or changes seam between modules.
  - Integration: task adds or changes path to outside world.
- [ ] IF test already goes through path, THEN add case to it. New test file = last solution.
- [ ] Scaffold tests temporary. Cleaner deletes them.
- [ ] IF changed behavior fits no lane in `tests/`, THEN keep one inline test for it.
- [ ] Never delete or weaken baseline test. Exception: task removes feature that test covers.

## Stage 0 — Lead

- Brief: what to build, constraints, files, "done" criteria.
- `baseline.md`: output of `git rev-parse HEAD` + `git status --short`. Each difference from baseline = change of this run.
- IF user gives path of a `baseline.md`, THEN copy that file. Write no new baseline.
- Plan = HOW: approach + files to change. Task description, feature request, design brief = WHAT. Not a plan.
- Plan can be in task, in file, or approved earlier in conversation.
- IF plan given, THEN copy to `plan.md` unchanged. Write no new plan.
  - IF gap in plan, THEN find answer in code + write it in brief. IF code has no answer, THEN send `STATUS: QUESTIONS`.
  - IF plan has no tests, THEN write test list in brief.
- ELSE write `plan.md`. Engineer must not have to guess. Include:
  - approach + reason it is better than alternative
  - files to change, module boundaries, test list, dependencies, risks, "done" criteria

## Stage 0b — Advisor

IF plan given, THEN skip this stage. Write `SKIPPED — plan given` in box.

```
DO
    Start new advisor.
    IF review.md = REVISE, THEN correct plan.md.
UNTIL review.md = APPROVE OR 3 rounds done.
IF no APPROVE after 3 rounds, THEN send `STATUS: QUESTIONS` with open questions.
```

Advisor instructions:

- [ ] Read `plan.md` + code it refers to.
- [ ] Find these problems:
  - design too complex
  - missing requirement
  - incorrect module boundary
  - test in incorrect lane
  - test list too long
  - lane with no reason
  - dependency not necessary
  - risk with no mitigation
  - text engineer cannot understand without guess
- [ ] Write `review.md` in tmp dir: APPROVE, or REVISE + numbered objections.
- [ ] Write no code. Do not change plan.

## Stage 1 — Engineer

Engineer instructions:

- [ ] Read `plan.md` first. Obey it. IF you must do something different, THEN give reason in report.
- [ ] Use TDD. Scaffold tests OK. Do not delete them.
- [ ] Write only tests on test list, in lanes list gives.
- [ ] IF work shows one more test is necessary, THEN add it + give reason in report.
- [ ] Report: what you built, decisions, trade-offs, weak points, each added test.

Gate: read report + diff. IF brief not complete OR build fails, THEN start engineer again with corrections.

## Stage 2 — Auditor

Auditor instructions:

- [ ] Do audit of change against `plan.md` + each skill. Correctness first.
- [ ] Finding: difference from plan with no reason. Finding: "done" criterion with no test.
- [ ] Do full `rex-code-smells` sweep on each changed file: three lenses, mechanical sweep block, clippy pass.
- [ ] Use table format of that skill. Severities: BLOCK, REFACTOR, NIT.
- [ ] Do audit of added tests against TEST RULES. Never delete scaffold test or baseline test.
- [ ] Delete each other added test that is one of these:
  - not on test list + no reason in engineer report
  - copy of coverage of different test
  - test of internals of one function
  - in lane whose conditions it does not obey
- [ ] Correct each finding you can correct now. Other findings: add short `SMELL:` flag (`rex-code-commenting`, whitelist #7).
- [ ] IF you cannot correct a BLOCK finding, THEN tell lead.
- [ ] Report: smell table or word "clean", corrections, new flags, deleted tests + reasons, each design problem.

Gate:

- [ ] IF design problem, THEN do stage 1 again with findings.
- [ ] IF report has no sweep, THEN start new auditor.
- [ ] IF test that auditor must delete is still there, THEN start new auditor.
- [ ] ELSE do stage 3.

## Stage 3 — Cleaner

Cleaner instructions:

- [ ] No behavior change.
- [ ] Do full `rex-code-smells` sweep on each changed file AFTER your own edits. This is last sweep of run.
- [ ] Correct each finding. OK: renames, extractions, signature cleanup. Not OK: feature changes.
- [ ] Build + all tests must pass after each correction.
- [ ] IF correction changes behavior, THEN skip it. Add `SMELL:` flag + put finding in report.
- [ ] IF finding = BLOCK, THEN stop + tell lead. No design change.
- [ ] Delete each scaffold test this run added. "Two weeks" rule in `rex-code-tests-unit-testing` does not apply.
- [ ] Exception: keep inline test that is on test list.
- [ ] Exception: keep one inline test for important behavior IF no other test covers it.
- [ ] Never touch `tests/fitness/`, `tests/integration/`, `tests/contract/`, or baseline test.
- [ ] Write comments again: short plain English, full sentences. Obey `rex-code-commenting`.
- [ ] CAUTION: Do not delete a flag. You can only make its words shorter. A deleted flag hides a known problem.
- [ ] Report: last smell table, corrections, new flags, changed comments.
- [ ] Report also: each deleted test + each kept test, with reason for each.

Gate:

- [ ] IF last sweep has BLOCK finding, THEN do stage 1 again with that finding.
- [ ] IF behavior change, deleted flag, or no last sweep, THEN reject work. Start new cleaner to correct it.

## Final gate

Do each check yourself. A subagent report is not proof.

- [ ] `cargo build` (or project equivalent) passes.
- [ ] `cargo clippy` gives no new warnings.
- [ ] All tests pass. All baseline tests there.
- [ ] Each added test on test list or has reason. IF not, THEN give it to new auditor.
- [ ] Final diff read: plan + brief complete, comments plain English, flags there.
- [ ] Make sure HEAD did not move + nothing is pushed. Exception: commit or push that user asked for.
- [ ] All boxes in `checklist.md` marked. IF box not marked, THEN do that step.

Then send REPORT to user. Give each field. Write `none` IF field is empty.

```
STATUS: DONE
TMP DIR: <path>
FINAL GATE: passed
CHECKLIST: <copy of checklist.md, all 15 boxes marked>
DONE CRITERIA: <one line each: criterion -> its proof. Test for new behavior or fix. Command or diff for refactor or chore.>
FILES CHANGED: <each path>
TESTS ADDED: <one line each: test + its reason>
BUILT: <what engineer built>
AUDIT: <what auditor changed>
CLEANUP: <what cleaner removed>
FLAGS: <each remaining SMELL: flag + its severity>
COMMIT: not committed | <hash>
```

## Codex review (optional)

Only IF user gives explicit request for it. Never by default.

1. Wait until final gate passes.
2. Obey `.claude/skills/rex-codex-adversarial-review/SKILL.md` on final diff.
3. Add its verdict table to report.
