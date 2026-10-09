<p align="center">
  <img src="https://raw.githubusercontent.com/coderaidershaun/rex-skills/main/static/logo.png" alt="Rex" width="200" />
</p>

<h1 align="center">Rex Skills</h1>

<p align="center">
  Agent skills for Rust projects, installable into any repo.
</p>

## Install and Run

```bash
cargo install rex-skills

# Then in your repo
rex skills
claude /rex-code-orchestrator please build me a house made out of rust.

# One fresh code lead per feature, fix, or chore, with a commit after each
claude /rex-code-orchestrator please build every feature in FEATURES.md.
```

Installs the `rex` binary (replaces any existing `rex` from rex-cli).

`rex skills` (and `rex init`) also put a "Codebase map" section and a "Code smells" section on top of `AGENTS.md`, creating the file if it is missing. The first tells an agent to run `rex codebase` before it searches the tree by hand. The second tells it to run `rex smells` to find each `SMELL:` flag left in the code. They are added once: a file that already has the `<!-- rex:codebase -->` line is not changed. To refresh them, delete both sections and that line, then run `rex skills` again.

Claude Code reads `AGENTS.md` only when the project has no `CLAUDE.md`. If you have one, add the line `@AGENTS.md` to it, or set Project instructions to `claude-md-and-agents-md`.

## Commands

```bash
# Install the skill bundle into ./.claude
rex skills
rex init                     # same command, alias

# Install for another vendor
rex skills --vendor codex    # -> ./.codex
rex skills --vendor agents   # -> ./.agents

# Print a tree of the current repo (nothing is written to disk)
rex codebase
rex codebase --rust-only                  # only .rs files and their directories
rex codebase --with-context               # add the first `//!` sentence to each .rs line
rex codebase --rust-only --with-context   # the Rust modules and what each is for

# Write the tree to CODEBASE.md instead of printing it (works with the other flags)
rex codebase --for-human --rust-only --with-context

# List every `SMELL:` flag comment in the Rust files as path:line, then the count
rex smells

# List every available command
rex commands
rex help
```
