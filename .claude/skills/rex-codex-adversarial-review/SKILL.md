---
name: rex-codex-adversarial-review
description: Run OpenAI's Codex CLI as a second-model adversarial reviewer, then cross-examine its findings and force one rebuttal round. Codex challenges the implementation approach, design choices, tradeoffs, and assumptions — not just defects; Claude verifies every finding against the real code before anything reaches the user. Use when user says "adversarial review", "codex review", "second opinion on this code/design", "challenge this implementation", or invokes /rex-codex-adversarial-review.
disable-model-invocation: false
user-invocable: true
---

# Codex Adversarial Review — Second-Model Challenge

Run the local `codex` CLI as an independent adversarial reviewer, cross-examine every claim it makes, send back one rebuttal round, and report only verdicts that survived. This is a challenge review: Codex questions whether the chosen approach is right, what assumptions it depends on, and where the design fails under real-world conditions — implementation defects come second.

## Hard rules

- `--sandbox read-only` on EVERY codex invocation, no exceptions. Never `workspace-write`, never `--yolo`. A review must not be able to touch the tree.
- Never use `--full-auto` — removed from the codex CLI in 0.147.
- Exactly one rebuttal round. Concede or contest, then stop — no loops.
- No Codex finding reaches the user unverified. Verify against the real files, not the diff alone.
- Review-only end to end: no fixes, no patches, no "while I'm here" edits. Fixes happen only if the user asks afterwards.

## 1. Preflight

- `command -v codex` — if missing, stop and tell the user: install with `brew install codex` (or `npm i -g @openai/codex`), then authenticate with `codex login`.
- Tell the user the review takes a few minutes; codex streams progress to stderr and prints its final message to stdout.

## 2. Resolve scope

Default scope, unless the user names one:

- Dirty working tree → review uncommitted changes (`git diff HEAD` plus untracked files).
- Clean tree → review the branch diff against the merge-base with `main` (or `master`).

The user may override with a base ref, specific paths, or a free-text focus ("challenge the error handling"). Collect the diff and the list of touched files before invoking codex.

## 3. Round 1 — Codex challenge review

Build the prompt with these demands, then feed it to codex via stdin heredoc with the diff embedded:

- Challenge the approach first: is this the right design at all? What assumptions does it silently depend on? What tradeoff was taken and what does it cost under real-world conditions? What simpler or safer alternative was ignored?
- Hunt real defects second: correctness, edge cases, error handling, concurrency, security.
- Every finding needs: severity `BLOCK`/`MAJOR`/`MINOR`, `file:line`, a one-sentence claim, and concrete evidence from the code.
- No fixes, no patches, no praise padding. Findings only.

```bash
codex exec --sandbox read-only -C <repo-root> \
  -c model_reasoning_effort=high \
  -o <scratch>/codex-review.md - <<'EOF'
<adversarial prompt + embedded diff>
EOF
```

Use a scratch directory for `-o`, never the repo. For small diffs `-c model_reasoning_effort=medium` is faster and usually enough.

## 4. Cross-examination

Read every finding from the review. For each one, open the actual files and verify the claim — the diff alone is not evidence. Classify:

- `CONFIRMED` — the claim holds; cite the exact `file:line`.
- `REJECTED` — the claim is wrong; state the disproving evidence.
- `PARTIAL` — kernel of truth, wrong severity or wrong reasoning; state both halves.

## 5. Round 2 — rebuttal

Send the classifications back to the same codex session and demand a response:

```bash
codex exec --sandbox read-only resume --last - <<'EOF'
Here is the verdict on each of your findings, with evidence: <classifications>
For each REJECTED or PARTIAL verdict: concede, or double down with NEW evidence.
Also name anything you now see that you missed the first time.
EOF
```

One round only. Whatever stands after this is final.

## 6. Final report

Deliver a verdict table: finding, severity, Codex position, Claude verdict, resolution (`upheld` / `withdrawn` / `contested`), `file:line`. List contested items honestly with both positions — do not silently drop disagreements. Then stop: no fixes unless the user asks.

## Failure modes

- codex missing or unauthenticated → preflight message, stop.
- Non-zero exit or empty stdout → report the last ~20 lines of stderr, retry once at most, then stop and tell the user.
- Review running far too long → suggest rerunning with `-c model_reasoning_effort=medium` or narrowing the scope.
