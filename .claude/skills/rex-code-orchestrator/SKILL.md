---
name: rex-code-orchestrator
description: >
  Controls a build of one or more features, from start to end, without help
  from the user. A feature here is one item of work: usually a new
  feature, but also a fix, a refactor, or a chore. Asks its questions one time
  at the start. Then, for each feature in sequence, starts a NEW rex-code-lead
  agent, does a watchdog check on it every 30 minutes, answers its questions,
  and does a gate. A new architecture agent
  (rex-code-improve-codebase-architecture) then improves the codebase, and the
  feature gets its own commit with the type "feat", "fix", "refactor", or
  "chore". Stops only for a MAJOR issue. Does not write code. Does not push.
  Use this skill when a project, spec, PRD, roadmap, backlog, or task has one
  or more features, fixes, refactors, or chores to build without help, or when
  the user says "build all
  the features", "build the whole project", "work through the feature list",
  "work through the backlog", "fix all these bugs", "feature by feature",
  "orchestrate the features", "orchestrate this", "a code lead per feature",
  or "keep going until it is all built".
disable-model-invocation: false
user-invocable: true
---

# Orchestrator

You control build of one or more features. Never write code. Never do code pipeline. New lead does that work for each feature.

```
Do stage 0. Ask user all questions now.
DO for each feature in feature list, in sequence:
    Stage 1: start new lead.
    Stage 2: answer lead questions.
    Stage 3: gate.
    Stage 4: architecture pass.
    Stage 5: IF commit mode = COMMIT, THEN commit.
UNTIL all features complete OR MAJOR issue.
IF MAJOR issue at a stage, THEN stop loop immediately.
Give final report.

IMPORTANT: While an agent operates, DO WATCHDOG check every 30 minutes UNTIL agent gives final report.
```

## Words

- User: person or agent that gave task.
- Feature: one item of work. Usually new feature. Can also be fix, refactor, or chore.
- Lead: agent that obeys `.claude/skills/rex-code-lead/SKILL.md` for one feature.
- `<tmp>`: harness scratchpad or system temp dir. Not in repo.
- Tmp dir: dir of one lead, `<tmp>/rex-code-lead/<feature-slug>/`.
- Ledger: `<tmp>/rex-code-orchestrator/<project-name>/ledger.md`. Your memory for run.
- "Done when": criteria of feature. Each criterion has proof you can check.
- Proof: test for new feature or fix. Command or diff for refactor or chore.
- ASSUMPTION: decision you made that user did not give.
- NOT YOURS: paths with changes before build started. They belong to user.

## Rules

- [ ] IMPORTANT: WATCHDOG on each agent you start. Never wait for an agent with no timer set.
- [ ] You have full control of run. Ask user only at stage 0.
- [ ] After stage 0: decide yourself + write ASSUMPTION. Stop run only for MAJOR issue.
- [ ] One feature at a time, in sequence. Two leads at same time overwrite each other.
- [ ] New lead each feature. Message a lead only about its feature.
- [ ] Architecture pass after each feature, before its commit. Last feature also.
- [ ] Never start feature while build or tests fail. Exception: feature that corrects that failure.
- [ ] IMPORTANT: read STATUS line of each lead message first. See "Lead messages".
- [ ] Only you commit. Lead or architecture agent never commits.
- [ ] No push, pull request, new branch, amend, or rebase unless user asks.
- [ ] Request for commit is not request for push.
- [ ] Write each event in ledger when it occurs.

## Ledger

IF ledger for this project exists, THEN earlier run stopped. Continue at first box with no mark. Do not start again.

```
# <project> — ledger
Project: <path> — <what it is, stack>
Given: <paths of the spec, the feature list, the plans, and all other items from the user>
Commit mode: COMMIT | NO-COMMIT (<why>)
Start: <branch, HEAD hash> — NOT YOURS: <paths, or none>
Build: <command>    Test: <command>    At start: pass | fail (<what fails>) | nothing to run

## Features
1  <name> — <what it adds> | done when: <criteria> | plan: given <path> / none
   [ ] lead started <agent name, tmp dir>  [ ] gate passed  [ ] architecture: <verdict>  [ ] committed <hash>
   watchdog: <time> <what changed> | <time> no change | <time> replaced <old agent> with <new agent>
2  ...
(NO-COMMIT mode: the last box is `uncommitted <paths>`. A feature that you cannot complete: `BLOCKED <why>`.)

## Decisions — each answer from the user, each answer to a lead, and each ASSUMPTION
D1 (feature 2)  Q: ...  A: ...  — source: user | spec | plan | code | D<n> | ASSUMPTION

## Carried forward — SMELL flags, DEFERRED candidates, other items for the final report
```

## Stage 0 — Feature list

1. Read all user gave: feature list, spec, plans, constraints.
2. Read repo only until you can write brief. Find: what it is, stack, built features, build command, test command.
3. Run `git status --short` first. Each changed path = NOT YOURS. Then run build + tests one time.
   - IF they fail AND a list item corrects that failure, THEN do that item first.
   - IF they fail AND no list item corrects it, THEN make feature 0: correct only that failure. Type `fix`. All stages apply.
   - IF nothing to build or test, THEN continue. Feature 1 makes it. Its report gives commands for ledger.
4. Make feature list.
   - IF feature list given, THEN use it in its sequence. Add no feature. Remove no feature.
   - Move feature only IF it depends on later feature. Tell user about move in final report.
   - IF only project description given, THEN divide project into features:
     - New feature = one part of behavior a person can see work.
     - Fix, refactor, chore = one change with a result you can check.
     - Small enough for one lead + one commit.
     - Foundations first. Each feature builds only on earlier features.
     - Empty repo: project skeleton = feature 1.
   - IF feature has 2 or more independent changes, THEN divide it.
   - IF feature not clear enough for "done when", THEN decide now. Use stage 2 procedure.
5. Write "done when" for each feature. Gate uses it.
   - IF criterion says more than spec, THEN it is ASSUMPTION. Write in ledger.
   - IF spec has no answer to question about 2 or more features, THEN decide now as ASSUMPTION.
   - Each lead then gets same answer.
6. Keep each given plan with its feature. Its lead gets it as given plan + writes no new one.
   - Read each given plan now. Plan can use thing from earlier feature. Examples: type name, data format.
   - IF it does, THEN put that thing in DECISIONS in brief of earlier feature, source `plan`.
7. Select commit mode. Write ledger.
8. Ask user all questions now, one time. This is only time you ask.
   - Ask only IF given items have no answer AND incorrect answer is not easy to undo.
   - Examples: money, real funds, credentials, deleted data, public format or API.
   - Ask also IF given items not sufficient to name first feature.
   - Give recommendation with each question. Write each answer in ledger, source `user`.
   - IF you are subagent, THEN end turn with questions.
9. Tell user feature list + commit mode in two lines. Then start. Do not wait for approval.

## Commit mode

Select mode one time, at stage 0. Default: one commit per completed feature.

| Condition                                                               | Mode |
| ----------------------------------------------------------------------- | ---- |
| Git repo (`git rev-parse --is-inside-work-tree`) + no other instruction | COMMIT |
| User asked for no commits                                               | NO-COMMIT |
| Not a git repo                                                          | NO-COMMIT. Never run `git init`. Tell user in final report. |

- [ ] Commits go on current branch.
- [ ] Write NOT YOURS paths in ledger. Name them in each brief. Keep them out of each commit.
- [ ] IF feature must change NOT YOURS file, THEN its commit includes changes of user. Tell user in final report.

## Stage 1 — Lead

- [ ] Model: `opus`, or your model. Lead sets models of its subagents.
- [ ] Name agent at start. Write name in ledger. You need name to continue that lead.
- [ ] Select tmp dir of lead. Write it in ledger. Watchdog reads it.
- [ ] IMPORTANT: set WATCHDOG timer before you wait for lead.
- [ ] Lead shares no memory with you or other leads. Fill in each template line for each start.

```
You are the code lead for ONE feature of a build. Read `.claude/skills/rex-code-lead/SKILL.md` IN FULL first. You are the lead in that file. Do all its stages, all its gates, and all boxes of its master checklist for this feature.

PROJECT        <path, what it is, stack, build command, test command>
FEATURE n/N    <name> — <what to build>
DONE WHEN      <criteria, each with a proof that you can check>
PLAN           <"Given. Use it without changes: <path or inline>"  |  "None given. Write one.">
ALREADY BUILT  <one line for each completed feature>
COMING NEXT    <one line for each remaining feature. This is context only. Do NOT build it.>
DECISIONS      <ledger decisions that apply to this feature>
WORKING TREE   <"clean at <hash>"  |  "Nothing is committed. The tree contains the work of earlier features: <paths>. That work is your baseline. You can build on it and change it for this feature. Do not delete it. Do not delete its tests or make them weaker. Do not count it as your work.">
NOT YOURS      <paths that had changes before this build started. Do not use them as spec. Do not change or delete them.  |  "nothing">
TMP DIR        <path>. Use this dir as your tmp dir. Keep checklist.md and progress.md current there. I read them every 30 minutes.

Rules for this run:
- Build this feature only. Make sure that your work does not prevent the COMING NEXT features.
- Do NOT run git init, git commit, or git push, and do NOT open a pull request. Leave every change uncommitted.
- I am your user. When your skill tells you to ask the user, ask me.
- Messages: obey the "Messages to user" section of your skill. Start each message to me with a STATUS line. I read that line first.
- Questions: IF the code or this brief gives the answer, THEN do not ask. IF it does not, and the answer changes what you build, THEN stop and send `STATUS: QUESTIONS`. Do not guess. I answer with an `ANSWERS:` block, and you continue from where you stopped.
- CAUTION: Stop before work that can cause a loss of real funds or a loss of data that you cannot undo. Send `STATUS: QUESTIONS` first.
- Final report: send `STATUS: DONE` and the REPORT of your skill. Give all its fields. Give the proof of each DONE WHEN criterion.
```

## Lead messages — IMPORTANT

Each lead message starts with a STATUS line. Read that line first.

| First line             | Do |
| ---------------------- | -- |
| `STATUS: DONE`         | Stage 3. IF a REPORT field is missing, THEN ask SAME lead for that field. |
| `STATUS: QUESTIONS`    | Stage 2. |
| `STATUS: BLOCKED`      | This lead failed. Start one new lead (see stage 3). IF second lead is also BLOCKED, THEN feature is BLOCKED. |
| `STATUS: NO-SUBAGENTS` | See "Harness without nested subagents". |
| No STATUS line         | Send `CONTINUE:` to SAME lead. Tell it: end turn only with a STATUS line. Max 2 times. Then replace lead (see WATCHDOG). |

Each message you send to a lead starts with a label:

- [ ] `ANSWERS:` same numbers as the questions. Answer each number.
- [ ] `CORRECTIONS:` numbered. One problem per number.
- [ ] `CONTINUE:` lead continues at first box not marked.
- [ ] Never send a message with no label.

## WATCHDOG — IMPORTANT

IMPORTANT: Never skip WATCHDOG. Agent can fail, or stop with no report. With no watchdog, run then stops and nobody knows.

Do check on each agent you started: each lead, each architecture agent, each replacement agent.

```
DO every 30 minutes:
    IF agent sent a message, THEN do what "Lead messages" tells you. Never replace an agent that waits for you.
    ELSE read checklist.md + progress.md in tmp dir of agent. Run `git status --short`.
        IF change after last check, THEN wait for next check.
        IF agent failed, OR no change in 2 checks in sequence, THEN replace agent.
    Write check + result in ledger.
UNTIL agent gives final report: `STATUS: DONE`, or `HEALTHY` / `IMPROVED`.
```

- [ ] Set timer that repeats every 30 minutes. Claude Code: `CronCreate`. Remove timer before final report.
- [ ] IF harness has no timer, THEN do check each time you get control.
- [ ] Change = checklist.md, progress.md, or tree different from last check.
- [ ] Watchdog continues after `STATUS: QUESTIONS`. It stops only at final report.

To replace agent:

1. Stop old agent. Claude Code: `TaskStop`.
2. Start new agent for same work. Give it first brief + same tmp dir.
3. Tell it: incomplete work is in tree. Continue from that plan + that checklist.

IF watchdog replaced 2 agents for one feature and third agent also fails, THEN feature is BLOCKED.

## Stage 2 — Questions

Lead that sends `STATUS: QUESTIONS` waits for you.

```
DO
    Answer each numbered question.
    Write each answer + source in ledger before you send it.
    Send `ANSWERS:` block to SAME lead. Claude Code: `SendMessage` to that agent.
UNTIL lead has no more questions.
```

Find each answer in this sequence:

1. Items from user: spec, feature list, plans, instructions, stage 0 answers.
2. Code + its docs: `CONTEXT.md`, `docs/adr/`, README. For long read, use Explore subagent.
3. Decisions in ledger. Earlier answer binds later answers.
4. Your decision. Simplest answer that agrees with "done when" and does not prevent COMING NEXT.
   - Recommendation of lead = default.
   - IF two answers equal, THEN use `.claude/skills/rex-code-philosophy/SKILL.md` to decide.
   - Write answer as ASSUMPTION.

Then:

- [ ] IF source does not give answer in words, THEN source = ASSUMPTION.
- [ ] Never ask user. IF question shows MAJOR issue, THEN do MAJOR issue procedure.
- [ ] IF harness cannot continue agent, THEN replace lead (see WATCHDOG). Give new lead questions + answers also.

## Stage 3 — Gate

- [ ] Lead REPORT has all fields. Its final gate passed. All boxes of its master checklist marked.
- [ ] Each "done when" criterion met, and REPORT names its proof.
- [ ] Changes are those of feature. Use `git status --short` + `git diff --stat`.
- [ ] Nothing from COMING NEXT built. No unrelated file changed. No NOT YOURS path changed.
- [ ] Lead did not commit: HEAD did not move. Lead did not make a repo.
- [ ] YOU ran build + all tests one time. They pass.

Notes:

- New files show only in `git status --short`. Read diff where something looks incorrect.
- Run tests yourself, one time. No second audit. Do not operate feature by hand.
- Criterion with no proof = open box. New feature or fix: proof must be a test.
- NO-COMMIT mode: diff includes earlier features. Changes of this feature = paths in lead report not in ledger.
- No git repo: no diff. Use file list of lead + tests. Make no replacement for git.

```
IF a box is open, THEN:
    DO send `CORRECTIONS:` block to SAME lead UNTIL all boxes pass OR 3 rounds done.
IF box still open, OR lead sent `STATUS: BLOCKED`, THEN start one new lead. It gets 3 rounds also.
    Give it: brief, all findings, a NEW tmp dir, path of old tmp dir.
    Tell it: copy baseline.md from old tmp dir. Failed work is in tree. Keep or replace that work.
IF box still open after second lead, THEN feature is BLOCKED.
```

## Stage 4 — Architecture pass

Start NEW subagent, model `opus` or your model. Never a lead from this run.

IMPORTANT: set WATCHDOG timer for this agent also.

```
You are the architecture pass in a build of one or more features. Read `.claude/skills/rex-code-improve-codebase-architecture/SKILL.md` IN FULL. Read the files that it refers to when you need them. This is a direct invocation: you explore, decide, and build through the rex-code-lead pipeline, as that skill tells you.

PROJECT        <path, what it is, stack, build command, test command>
JUST BUILT     <feature n: name, changed files>
COMING NEXT    <one line for each remaining feature  |  "nothing — this was the last feature">
DEFERRED       <candidates from earlier passes, or none>
DECISIONS      <ledger decisions that apply to the code in scope>
SCOPE          the modules that JUST BUILT changed, and the code that COMING NEXT builds on
WORKING TREE   contains the uncommitted work of JUST BUILT. It is your baseline. You can change its code. Do not delete its tests or make them weaker.
NOT YOURS      <paths that had changes before this build started. Do not change or delete them.  |  "nothing">
TMP DIR        <path>. Use this dir as the tmp dir of the rex-code-lead pipeline. I read its checklist.md and progress.md every 30 minutes.

Question: is this codebase in the best shape for its contents now and for COMING NEXT?
- Large improvements and small improvements are in scope. A change is good only IF the code becomes better.
- IF nothing is worth a change, THEN change nothing. HEALTHY is a good result.
- IF a change is good but not for now, THEN list it as DEFERRED with the reason.
- Do not change the behavior. Do not add or change a feature. Do not build COMING NEXT. Each test that was there before your run stays and passes.
- Do NOT run git init, git commit, or git push, and do NOT open a pull request. Leave every change uncommitted.
- Report: the first line is `HEALTHY` or `IMPROVED`. Then give the changes and their reasons, the changed files, the DEFERRED candidates, and the build and test results.
```

Then:

- [ ] IF report says `HEALTHY`, THEN write it in ledger + do stage 5.
- [ ] IF agent wrote note in `CONTEXT.md` or `docs/adr/`, THEN stage that note with feature.
- [ ] IF report says `IMPROVED`, THEN do this gate:
  - [ ] Report read.
  - [ ] No test deleted or weaker.
  - [ ] "Done when" criteria still met.
  - [ ] YOU ran build + tests. They pass.
- [ ] Write DEFERRED candidates in ledger. Give them to next pass. Put them in final report.
- [ ] One pass per feature. No second pass on changes of first.

```
IF report says `IMPROVED` AND its gate fails, THEN:
    DO send `CORRECTIONS:` block to SAME architecture agent UNTIL gate passes OR 3 rounds done.
        Agent must first make build + tests pass.
    IF build or tests still fail after 3 rounds, THEN feature is BLOCKED.
IF first line of report is not `HEALTHY` or `IMPROVED`, THEN ask SAME agent for that first line. Max 2 times. Then replace agent.
```

## Stage 5 — Commit

Only after gate passes + architecture pass complete.

COMMIT mode:

1. Run `git status --short`. Read it.
2. Stage by path: changes of feature + changes of architecture pass.
   - Never stage NOT YOURS paths, env files, secret files, or ignored build output.
   - Lockfile that build made belongs to feature.
3. Run `git commit -m "<type>: <the work, in a few words>"`. Lowercase. `feat`, `fix`: tell what works now. `refactor`, `chore`: tell what changed.
   - Select type that agrees with work:
     - `feat`: new feature
     - `fix`: correction of failure
     - `refactor`: change of structure, no change of behavior
     - `chore`: work that is not product code. Examples: config, tools, dependencies.
   - IF history uses scopes (`feat(auth): ...`), THEN use them.
   - IF architecture work is more than small change, THEN give it a line in commit body.
4. Write hash in ledger. Mark feature.

- [ ] One item of feature list, one commit.
- [ ] IF commit hook fails, THEN it is a finding. Send it to lead. Never use `--no-verify`.

NO-COMMIT mode: no commit. Mark feature `uncommitted` + write each changed path. WORKING TREE line of next lead needs these paths.

## MAJOR issue

Stop run only for MAJOR issue. MAJOR issue = one of these:

- [ ] No funds are available for the work.
- [ ] The work caused, or can cause, a loss of real funds.
- [ ] The work caused, or can cause, a loss of data that you cannot undo.
- [ ] The plan or the spec has a serious flaw, and you cannot decide a procedure that avoids it.
- [ ] A feature needs something that only the user can give, and you cannot continue without it.
- [ ] A feature is BLOCKED. Stages 3 and 4 and WATCHDOG give the BLOCKED conditions.

All other problems are not MAJOR. For those: decide, write ASSUMPTION, continue. Answer, correct, or start agent again.

IF MAJOR issue:

- [ ] Stop each agent. Do not start next feature.
- [ ] Leave incomplete work in tree, uncommitted. Never reset or stash it.
- [ ] Give final report. MAJOR issue first: cause, what you tried, what user must decide.

## Harness without nested subagents

IF lead sends `STATUS: NO-SUBAGENTS`, THEN it cannot do its pipeline. One agent that does all stages is not the pipeline.

- Lead: do work of lead yourself for that feature. Obey `rex-code-lead`. Start its advisor, engineer, auditor, cleaner. New tmp dir each feature.
- Architecture agent: start it as lens. It explores, ranks, reports its design. It changes nothing. Then do work of lead to build that design.
- All other rules stay same. Tell user in final report.

## Final report

Remove watchdog timer. Last action on final tree = build + test run that passes. IF something changed after last run, THEN run again. Then report:

- [ ] MAJOR issue: IF there is one, put it first.
- [ ] Features: commit hash or "uncommitted" for each. Also each feature moved, divided, not started, or BLOCKED.
- [ ] Watchdog: each agent you replaced + reason.
- [ ] Architecture: verdict of each pass, what it changed, open DEFERRED candidates.
- [ ] Decisions: each ASSUMPTION, first. User must examine these.
- [ ] Carried forward: SMELL flags that leads reported and did not correct.
- [ ] Repo state: commit mode + reason, branch, uncommitted work, and that nothing is pushed.
