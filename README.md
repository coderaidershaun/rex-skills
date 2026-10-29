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

`rex skills` also adds a block to `.gitignore` so the installed skill files are not committed: `.claude/skills/` and `.claude/settings.local.json` for the default vendor, or `.codex/skills/` and `.agents/skills/` for the others. The block is appended once per vendor; a `.gitignore` that already has the `# rex:ignore claude` line is not changed.

It also writes `.pre-commit-config.yaml` for the [pre-commit](https://pre-commit.com) framework and runs `pre-commit install --hook-type pre-commit --hook-type pre-push`. At commit time the hooks run gitleaks on the staged diff, `cargo fmt --check` and `cargo clippy -D warnings`, plus the usual hygiene checks (large files, merge markers, TOML and YAML syntax, private keys, trailing whitespace). At push time they run `trivy fs` (secrets, vulnerabilities and misconfigurations at HIGH and CRITICAL), which scans the whole tree and needs its vulnerability database, and `cargo test`. The hooks need `pre-commit`, `gitleaks` and `trivy` on PATH; when `pre-commit` is missing, rex prints the install command to run later, and when `gitleaks` or `trivy` is missing it says so on stderr, because that hook would fail every time it runs. A config you wrote yourself is never replaced.

Claude Code reads `AGENTS.md` only when the project has no `CLAUDE.md`. If you have one, add the line `@AGENTS.md` to it, or set Project instructions to `claude-md-and-agents-md`.

## Commands

```bash
# Install the skill bundle into ./.claude
rex skills
rex init                     # same command, alias

# A skill file that already exists is never replaced, and the summary says how
# many of them differ from this version. To update them, overwrite those files:
rex skills --force           # only skill files are replaced; AGENTS.md, .gitignore and your pre-commit config keep their add-once rules

# Install for another vendor. The skill text is rewritten to name that
# folder, so a path such as .claude/skills/INDEX.md becomes .codex/skills/INDEX.md.
rex skills --vendor codex    # -> ./.codex
rex skills --vendor agents   # -> ./.agents

# Print a tree of the current repo (nothing is written to disk).
# A directory or Rust file that cannot be read is skipped, with a warning on stderr.
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
