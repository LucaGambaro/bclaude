# betterClaude

Rust crate `bclaude` (lib `bc` + thin `main.rs`, binary `bclaude`). See README.md for behaviour and config format, `docs/profiles.md` for the profiles guide.

## Layout
- `config/` schema (serde), load/save, membership edits · `library/` scan, frontmatter, add/import, installed plugins (read from `~/.claude/plugins`, never copied) · `mcp.rs` MCP server defs + session `--mcp-config`
- `resolve.rs` pure cwd → profiles/names · `compose.rs` per-session plugin + CLAUDE.md + settings.json + mcp.json
- `session_settings.rs` per-session `--settings` (enabledPlugins) · `launch.rs` flags + spawn · `runner.rs` lifecycle (prepare, session record, cleanup)
- `sessions.rs` live session records · `report.rs` text rendering · `cli.rs` routing/clap · `commands.rs` I/O glue
- `tui/` `state.rs` (pure), `ui.rs` (ratatui), `mod.rs` (key handling + loop)

## Rules
- Unit-test everything (`cargo test`); inject paths/cwd/writers instead of reading the environment in logic.
- `cli`/`commands` only do I/O and delegate; no business logic there.
- Errors propagate as `BcError`; print once in `main`. No log + return in the same place.
- No `unsafe`, no env mutation in tests (use `BC_HOME`-style injected `Paths`).
- Keep README.md, `docs/`, CHANGELOG.md (`Unreleased`) and this file in sync with any change.
- Before a PR: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (same as CI in `.github/workflows/ci.yml`).
- Community files (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE-MIT/APACHE) live at the repo root; license is `MIT OR Apache-2.0`.
