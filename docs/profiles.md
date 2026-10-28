# Profiles

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
