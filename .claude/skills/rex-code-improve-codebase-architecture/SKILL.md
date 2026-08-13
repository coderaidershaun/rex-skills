---
name: rex-code-improve-codebase-architecture
description: Find deepening opportunities in codebase, informed by domain language in CONTEXT.md + decisions in docs/adr/. Runs autonomously start to finish — never asks the user anything, picks the target itself, and hands the winning design to rex-code-lead to build. Use when user wants improve architecture, find refactor opportunities, consolidate coupled modules, make code more testable + AI-navigable, says "improve architecture", "refactor opportunities", "deepen modules".
disable-model-invocation: false
user-invocable: true
---

# Improve Codebase Architecture

Surface arch friction. Propose **deepening opportunities** — refactor shallow module deep. Aim: testability + AI-navigability + ergonomics.

## Cousin skills

- `rex-code-ergonomics`
- `rex-code-commenting`

## Glossary

Use these exact every suggestion. Consistent language = point. No drift to "component", "service", "API", "boundary". Full defs: [LANGUAGE.md](LANGUAGE.md).

- **Module** — interface + impl (fn, class, package, slice).
- **Interface** — everything caller must know: types, invariants, error modes, ordering, config. Not just signature.
- **Implementation** — code inside.
- **Depth** — leverage at interface. Lots behavior behind small interface. **Deep** = high leverage. **Shallow** = interface ≈ impl complexity.
- **Seam** — where interface live. Swap behavior w/o edit-in-place. (Use this, not "boundary".)
- **Adapter** — concrete thing satisfy interface at seam.
- **Leverage** — caller payoff from depth.
- **Locality** — maintainer payoff from depth: change/bug/knowledge one place.

Key principles ([LANGUAGE.md](LANGUAGE.md) full list):

- **Deletion test**: delete module. Complexity vanish -> pass-through. Complexity reappear across N callers -> earned keep.
- **Interface = test surface.**
- **One adapter = hypothetical seam. Two adapters = real seam.**

Skill _informed_ by domain model. Domain language name good seams. ADRs record decisions skill must not re-litigate.

## Process

**Fully autonomous. Ask the user NOTHING. Not before starting, not mid-flow, not before implementing.**

Hard rules:

- **No scoping question.** Invocation named a path or module? That's the scope. Named nothing? Scope = whole repo, minus vendored/generated code. Never ask "which area".
- **No permission question.** No "shall I proceed", no "want me to implement this", no "does this look right". The user already said go by invoking the skill.
- **No choice question.** Ranking, candidate, interface, seam placement — decide yourself. Ties are broken by you, not by the user.
- **Unknowns become assumptions, not questions.** Pick the sane reading, write it in the report under a heading that says what you assumed, and carry on.
- Findings, decisions, and risks land in the final report. Never as a prompt.

**Second opinion instead of asking.** Stuck, a load-bearing call, or a genuine tie? Dispatch a fresh subagent via the Agent tool (model `opus` or inherit) with the file paths, the shortlist, and the trade-off. It returns a recommendation and reasons; you take the call. A second opinion is always a subagent dispatch, never a question to the user.

**Lens mode.** Another skill or subagent loaded this file as a review lens (e.g. the `rex-code-lead` auditor)? Then stop after step 2 — report findings, change nothing, run no pipeline. Only a direct invocation runs step 4.

### 1. Explore

Read domain glossary + ADRs in area first.

Then Agent tool `subagent_type=Explore` walk codebase. No rigid heuristics — explore organic, note friction:

- Understanding one concept = bouncing across many small modules?
- Module **shallow** — interface ≈ impl complexity?
- Pure fns extracted for testability, but real bug hide in how called (no **locality**)?
- Coupled modules leak across seams?
- Untested or hard test through current interface?

Apply **deletion test** to suspect shallow: delete -> complexity concentrate, or move? Concentrate = signal.

### 2. Rank candidates

Numbered list, sorted by leverage × locality win. Each:

- **Files** — which files/modules
- **Problem** — why current arch friction
- **Solution** — plain English, what change
- **Benefits** — locality + leverage + test improvement
- **Ergonomics win** — does change make caller life easier? Fewer files to hunt? Better autocomplete? Say so.

**CONTEXT.md vocab for domain. [LANGUAGE.md](LANGUAGE.md) vocab for arch.** `CONTEXT.md` defines "Order" -> say "Order intake module". Not "FooBarHandler". Not "Order service".

**ADR conflicts**: candidate contradicts ADR? Surface only when friction real enough revisit ADR. Mark clear ("contradicts ADR-0007 — worth reopening because…"). No listing every theoretical refactor ADR forbids.

Pick top candidate(s) self. Tie or unsure between top 2-3 → second-opinion subagent, then decide. Never "which one do you want?"

### 3. Grill self

For chosen candidate, walk design tree alone: constraints, deps, shape of deepened module, what sit behind seam, what tests survive. Write reasoning down in report.

Side effects inline as decisions crystallize:

- **Name deepened module after concept not in `CONTEXT.md`?** Add term now. Same discipline as `/rex-plan-discovery`. See [CONTEXT-FORMAT.md](../rex-plan-discovery/CONTEXT-FORMAT.md). Lazy create file.
- **Sharpening fuzzy term mid-flow?** Update `CONTEXT.md` now.
- **Reject candidate w/ load-bearing reason?** Write ADR direct (no ask). Only when reason needed by future explorer. Skip ephemeral + self-evident. See [ADR-FORMAT.md](../rex-plan-discovery/ADR-FORMAT.md).
- **Alt interfaces for deepened module?** See [INTERFACE-DESIGN.md](INTERFACE-DESIGN.md).
- **Stuck on design tradeoff?** Second-opinion subagent, not the user.

### 4. Build it

Direct invocation only — in lens mode you already stopped at step 2.

Chosen design goes straight to `.claude/skills/rex-code-lead/SKILL.md`. No confirmation step, no "ready to start?". Hand over the design as the task brief — files in play, current friction, target interface, seam placement, dependency category, which tests must survive — and run that pipeline end to end: plan → advisor gate → engineer → auditor → cleaner → final gate.

The lead's gates are the safety net, so there is nothing left to ask about. Report at the end: what was found, what was chosen and why, what shipped, what the audit and final smell sweep flagged.

Stop before building only if the change cannot proceed at all — no compiling build, or the chosen candidate collides with a `docs/adr/` decision you judged not worth reopening. Then say so plainly in the report and leave the code untouched.

## Ergonomics check

Every candidate must answer:

- **Caller code shorter or longer after?** Shorter = win. Longer = sus.
- **How many files reader hop after?** Fewer = win.
- **Naming match domain (CONTEXT.md)?** Yes = navigable. No = friction.
- **IDE autocomplete surface easier or harder?** Easier = win.
- **Error message reach caller w/ context?** Yes = debuggable.

Fail any? Reconsider before propose.

## Size check

Under no circumstances should there be ANY code file over 500 lines of code.

<!-- Adapted from Matt Pocock [YouTube] -->
