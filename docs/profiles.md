# Profiles

A **profile** is a named bundle of skills, agents, plugins, MCP servers and CLAUDE.md fragments. Which profiles are active in a directory is decided by `~/.betterclaude/config.toml`. Nothing is ever written inside your repositories.

### Rules for what gets activated

1. `[profiles.global]` is active everywhere (it must exist in the file to apply).
2. Any other profile is active when the current directory — **or one of its parents** — matches one of its `match` globs. So `match = ["~/Desktop/acme/*"]` also covers `~/Desktop/acme/repo/src/deep`.
3. `[projects."<path>"]` is an override for one directory (and everything below it). It can switch on extra profiles (`profiles = [...]`) and add extra `skills` / `agents` / `claude_md` / `plugins` / `mcp`.
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
plugins = ["code-simplifier@anthropic-plugin-directory"]
mcp = ["serena"]

[profiles.acme]                      # a group of repos
description = "Acme repos"
skills = ["acme-pr-review", "pr-description"]
agents = ["pr-agent", "plan-reviewer"]
plugins = ["pr-review-toolkit@anthropic-plugin-directory"]
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

### Plugins

Plugins are referenced by id (`name@marketplace`), exactly as Claude Code registers them in `~/.claude/plugins/installed_plugins.json`; list them with `bclaude ls --kind plugin`. They are never copied into the library: install them with `claude plugin install`. Rules at launch:

- If the resolved set contains at least one installed plugin, `bclaude` passes `--settings <run>/settings.json` with `enabledPlugins`: the selected plugins are `true`, **every other installed plugin is `false`** for that session.
- If no plugin is selected, nothing is overridden and your normal Claude plugin settings apply.
- An id that is not installed is reported as `MISSING` by `bclaude which` and skipped.

### MCP servers

MCP servers live in the library as `library/mcp/<name>.json`, each file being one server definition (what sits under `mcpServers.<name>` in a Claude config). Get them there with `bclaude import` (user-scope servers from `~/.claude.json`) or `bclaude add mcp <file.json> [--name n]`. Rules at launch:

- If the resolved set contains at least one server, `bclaude` writes `<run>/mcp.json` and passes `--mcp-config <run>/mcp.json --strict-mcp-config`. `claude` then loads **only** those servers: user-scope servers, project `.mcp.json`, plugin-provided servers and claude.ai connectors are all ignored for that session.
- If no server is selected, nothing is overridden and your normal MCP setup applies.
- A name that is not in the library is reported as `MISSING` by `bclaude which` and skipped.
- Project-scope servers (`claude mcp add --scope project/local`) are not imported: copy their JSON with `bclaude add mcp`.

### Using the dashboard

`bclaude dash` edits **membership** of entries in profiles that already exist: pick the active profile with `p`, move on a skill/agent/CLAUDE.md/plugin/MCP row and press `space` to add or remove it (saved immediately to `config.toml`). The dashboard does **not** create profiles or edit `match` / `[projects]`: do that in the file, then press `r` in the dashboard to reload.

### Typical workflow

1. New group of repos → add `[profiles.<name>]` with `match` globs.
2. Assign entries (file or dashboard) → verify with `bclaude which` in one repo of the group and in one outside it.
3. Rename or delete a profile → edit the file; library files are never deleted by profile changes.

### Why you may still see "all" skills

`bclaude` only **adds** the selected skills and agents (as `/bc:<name>`). `claude` still loads everything in `~/.claude/skills` and `~/.claude/agents`. To really limit them, move the originals out of `~/.claude` (after `bclaude import`) and assign them to profiles. Installed plugins are the exception: once a profile selects any plugin, the others are disabled for the session.
