# betterClaude

Rust crate (lib + thin `main.rs`). See README.md for behaviour and config format.

## Layout
- `config/` schema (serde), load/save, membership edits · `library/` scan, frontmatter, add/import
- `resolve.rs` pure cwd → profiles/names · `compose.rs` per-session plugin + CLAUDE.md
- `launch.rs` flags + spawn · `runner.rs` lifecycle (prepare, session record, cleanup)
- `sessions.rs` live session records · `report.rs` text rendering · `cli.rs` routing/clap · `commands.rs` I/O glue
- `tui/` `state.rs` (pure), `ui.rs` (ratatui), `mod.rs` (key handling + loop)

## Rules
- Unit-test everything (`cargo test`); inject paths/cwd/writers instead of reading the environment in logic.
- `cli`/`commands` only do I/O and delegate; no business logic there.
- Errors propagate as `BcError`; print once in `main`. No log + return in the same place.
- No `unsafe`, no env mutation in tests (use `BC_HOME`-style injected `Paths`).
- Keep README.md and this file in sync with any change.
