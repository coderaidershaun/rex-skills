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
claude /rex-code-lead please build me a house made out of rust.
```

Installs the `rex` binary (replaces any existing `rex` from rex-cli).

## Commands

```bash
# Install the skill bundle into ./.claude
rex skills
rex init                     # same command, alias

# Install for another vendor
rex skills --vendor codex    # -> ./.codex
rex skills --vendor agents   # -> ./.agents

# Write a CODEBASE.md tree outline of the current repo
rex codebase

# List every available command
rex commands
rex help
```
