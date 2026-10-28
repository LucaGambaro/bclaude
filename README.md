# betterClaude (`bclaude`)

A thin Rust layer on top of `claude`. It always launches the same engine, but composes **skills, agents and CLAUDE.md** per project group at launch time — without touching any repository or `~/.claude`.

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

## Profiles: how to set them up

A **profile** is a named bundle of skills, agents and CLAUDE.md fragments. Which profiles are active in a directory is decided by `~/.betterclaude/config.toml`. Nothing is ever written inside your repositories.

### Rules for what gets activated

1. `[profiles.global]` is active everywhere (it must exist in the file to apply).
2. Any other profile is active when the current directory — **or one of its parents** — matches one of its `match` globs. So `match = ["~/Desktop/acme/*"]` also covers `~/Desktop/acme/repo/src/deep`.
3. `[projects."<path>"]` is an override for one directory (and everything below it). It can switch on extra profiles (`profiles = [...]`) and add extra `skills` / `agents` / `claude_md`.
4. The final set is the union of all active profiles, without duplicates. A name that is in the config but not in the library is reported as `MISSING` by `bclaude which` and skipped.

Glob notes: `~` is expanded, `*` matches one path segment, `**` matches any depth. Keys that contain `~` or `/` must be quoted (`[projects."~/Desktop/PRChecker"]`).

### Create or edit a profile

1. Put the building blocks in the library (once): `bclaude import`, or `bclaude add skill|agent|md <path> [--name n]`.
2. Open `~/.betterclaude/config.toml` and add or change a profile:

```toml
[profiles.global]                      # always on
description = "Personal tools"
skills = ["handoff", "find-skills"]
claude_md = ["personal"]

[profiles.acme]                      # a group of repos
description = "Acme repos"
skills = ["acme-pr-review", "pr-description"]
agents = ["pr-agent", "plan-reviewer"]
claude_md = ["acme"]
match = ["~/Desktop/acme/*", "~/work/acme-legacy/*"]   # several globs allowed

[profiles.creative]                    # another group
skills = ["layer-3d", "archify"]
match = ["~/Desktop/creative/*"]

[projects."~/Desktop/PRChecker"]       # one specific repo
profiles = ["acme"]                  # also activate this profile here
skills = ["graphify"]                  # plus this extra skill, only here
```

3. Check the result from inside a repo — no need to restart anything:

```
cd ~/Desktop/PRChecker
bclaude which          # active profiles (and why), entries, estimated tokens, missing names
bclaude --dry-run      # the exact claude command that would run
```

### Using the dashboard

`bclaude dash` edits **membership** of entries in profiles that already exist: pick the active profile with `p`, move on a skill/agent/CLAUDE.md row and press `space` to add or remove it (saved immediately to `config.toml`). The dashboard does **not** create profiles or edit `match` / `[projects]`: do that in the file, then press `r` in the dashboard to reload.

### Typical workflow

1. New group of repos → add `[profiles.<name>]` with `match` globs.
2. Assign entries (file or dashboard) → verify with `bclaude which` in one repo of the group and in one outside it.
3. Rename or delete a profile → edit the file; library files are never deleted by profile changes.

### Why you may still see "all" skills

`bclaude` only **adds** the selected skills (as `/bc:<name>`). `claude` still loads everything in `~/.claude/skills`, `~/.claude/agents` and installed plugins. To really limit them, move the originals out of `~/.claude` (after `bclaude import`) and assign them to profiles.

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
cargo install --path .
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
cargo run -- ls
```
