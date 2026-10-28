# betterClaude (`bclaude`)

[![CI](https://github.com/LucaGambaro/bclaude/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaGambaro/bclaude/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**Load only the Claude Code skills, agents and CLAUDE.md that matter for the project you are in.**

`bclaude` is a thin Rust layer on top of `claude`. It always launches the same engine, but composes **skills, agents and CLAUDE.md** per project group at launch time — without touching any repository or `~/.claude`.

## Why

Claude Code loads everything in `~/.claude`, in every directory. The more skills and agents you collect, the more context you burn on things unrelated to the repo you are working on. `bclaude` lets you keep one library and activate only the right bundle (a *profile*) depending on where you launch it.

## Quick start

```
cargo install bclaude                            # or: cargo install --path .
bclaude import                                   # copy skills and agents from ~/.claude into the library
bclaude dash                                     # assign entries to profiles in the TUI
cd ~/work/some-repo && bclaude which             # see what applies here
bclaude                                          # launch claude with the composed config
```

## How it works

1. `bclaude` looks at the current directory and resolves which **profiles** apply (`~/.betterclaude/config.toml`).
2. It generates a per-session dir `~/.betterclaude/run/<pid>/` with a plugin (symlinks to the selected skills/agents) and a concatenated `CLAUDE.md`.
3. It runs `claude --plugin-dir <plugin> --append-system-prompt-file <CLAUDE.md> <your args>` and cleans up on exit.

Skills/agents of the generated plugin are namespaced: `/bc:<skill>`. Everything you pass after `bclaude` goes to `claude` unchanged (`bclaude -c`, `bclaude -p "..."`).

## Data layout (`~/.betterclaude`, override with `BC_HOME`)

```
config.toml
library/skills/<name>/SKILL.md
library/agents/<name>.md
library/claude-md/<name>.md
sessions/<pid>.json     # live sessions (used by the dashboard)
run/<pid>/              # generated per session
```

```toml
[profiles.global]                 # always active
skills = ["handoff"]
agents = ["plan-reviewer"]
claude_md = ["personal"]

[profiles.acme]                 # active when cwd (or a parent) matches a glob
description = "Acme repos"
skills = ["acme-pr-review"]
claude_md = ["acme"]
match = ["~/work/acme/*"]

[projects."~/work/solo-repo"]     # per-project override
profiles = ["acme"]
skills = ["only-here"]
```

The full guide to profiles, glob rules and the dashboard workflow is in [docs/profiles.md](docs/profiles.md).

## Commands

| Command | What it does |
|---|---|
| `bclaude [claude args]` | Launch claude with the composed config |
| `bclaude --dry-run [args]` | Print the claude command that would run |
| `bclaude ls [--kind skill\|agent\|md]` | List library entries, their profiles and descriptions |
| `bclaude which` | What applies to the current dir, why, and estimated token cost |
| `bclaude add <skill\|agent\|md> <path> [--name n]` | Copy into the library |
| `bclaude import` | Copy skills/agents from `~/.claude` into the library |
| `bclaude dash` | TUI dashboard |

Dashboard keys: `←/→` tabs, `↑/↓` move, `space` toggle membership in the active profile (saved to config), `p` cycle active profile, `r` reload, `q` quit. Tabs: Skills, Agents, CLAUDE.md, Profiles, Sessions (live `bclaude` sessions).

`BC_CLAUDE_BIN` overrides the `claude` binary.

## Installation

Requires Rust (`brew install rustup && rustup default stable`).

```
cargo install bclaude          # from crates.io, once published
cargo install --path .         # from a local clone
```

This installs the `bclaude` binary into `~/.cargo/bin`. If that directory is not in your `PATH` (check your `~/.zshrc`), add it:

```
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## First-time setup

```
bclaude import                                   # copy skills and agents from ~/.claude into the library
bclaude add md ~/.claude/CLAUDE.md --name personal
bclaude ls                                       # list entries with their descriptions
bclaude dash                                     # dashboard to assign entries to profiles
```

## Development

```
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- ls
```

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first, and see the [changelog](CHANGELOG.md) for what changed. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md); report vulnerabilities as described in [SECURITY.md](SECURITY.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
