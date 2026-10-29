# betterClaude (`bclaude`)

[![CI](https://github.com/LucaGambaro/bclaude/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaGambaro/bclaude/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**Load only the Claude Code skills, agents, plugins, MCP servers and CLAUDE.md that matter for the project you are in.**

`bclaude` is a thin Rust layer on top of `claude`. It always launches the same engine, but composes **skills, agents, plugins, MCP servers and CLAUDE.md** per project group at launch time — without touching any repository or `~/.claude`.

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
3. If the profiles select installed plugins, it also writes a `settings.json` that enables exactly those plugins and disables every other installed one.
4. If they select MCP servers, it writes an `mcp.json` with exactly those servers and passes `--mcp-config <mcp.json> --strict-mcp-config`.
5. It runs `claude --plugin-dir <plugin> --settings <settings.json> --mcp-config <mcp.json> --strict-mcp-config --append-system-prompt-file <CLAUDE.md> <your args>` and cleans up on exit.

Skills/agents of the generated plugin are namespaced: `/bc:<skill>`. Everything you pass after `bclaude` goes to `claude` unchanged (`bclaude -c`, `bclaude -p "..."`).

## Data layout (`~/.betterclaude`, override with `BC_HOME`)

```
config.toml
library/skills/<name>/SKILL.md
library/agents/<name>.md
library/claude-md/<name>.md
library/mcp/<name>.json   # one MCP server definition (the value of mcpServers.<name>)
                        # plugins are not copied: read from ~/.claude/plugins/installed_plugins.json
sessions/<pid>.json     # live sessions (used by the dashboard)
run/<pid>/              # generated per session (plugin/, CLAUDE.md, settings.json, mcp.json)
```

```toml
[profiles.global]                 # always active
skills = ["handoff"]
agents = ["plan-reviewer"]
claude_md = ["personal"]
plugins = ["code-simplifier@anthropic-plugin-directory"]   # name@marketplace
mcp = ["serena"]                  # library/mcp/serena.json

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
| `bclaude ls [--kind skill\|agent\|md\|plugin\|mcp]` | List library entries, their profiles and descriptions |
| `bclaude which` | What applies to the current dir, why, and estimated token cost |
| `bclaude add <skill\|agent\|md\|mcp> <path> [--name n]` | Copy into the library (`mcp` takes a `.json` server definition) |
| `bclaude import` | Copy skills/agents from `~/.claude` and user-scope MCP servers from `~/.claude.json` into the library |
| `bclaude dash` | TUI dashboard |

Dashboard keys: `←/→` tabs, `↑/↓` move, `space` toggle membership in the active profile (saved to config), `p` cycle active profile, `r` reload, `q` quit. Tabs: Skills, Agents, CLAUDE.md, Plugins, MCP, Profiles, Sessions (live `bclaude` sessions).

Plugins are listed from what Claude Code has already installed (`claude plugin install ...`); `bclaude add plugin` is rejected because there is nothing to copy. Plugin ids are `name@marketplace`, as shown by `bclaude ls --kind plugin`. When a session selects at least one plugin, all other installed plugins are disabled for that session; with no plugin selected your normal Claude settings apply untouched.

MCP servers are copied into the library as JSON and referenced by name. When a session selects at least one, `--strict-mcp-config` makes `claude` ignore **every other MCP source** (user-scope servers, project `.mcp.json`, plugin-provided servers, claude.ai connectors), so list all the servers you want. With none selected, your normal MCP setup applies untouched. The definition files may contain secrets (`env`, `headers`): the dashboard preview shows only a summary, and the per-session `mcp.json` is deleted on exit.

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
