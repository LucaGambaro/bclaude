# Changelog

All notable changes to this project are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- `--kind` and the `add` kind argument now accept any letter case (`MCP`, `Skill`).

### Added
- MCP server management: a new `mcp` kind (`mcp = [...]` in profiles and project overrides, `bclaude ls/add/import` support, an MCP tab in the dashboard). Selected servers are written to a per-session `mcp.json` and passed with `--mcp-config --strict-mcp-config`, so only those servers load.
- Plugin management: a new `plugin` kind (`plugins = [...]` in profiles and project overrides, `bclaude ls --kind plugin`, a Plugins tab in the dashboard). Plugins are read from Claude Code's installed plugins and enabled per session through a generated `--settings` file; when any plugin is selected, all other installed plugins are disabled for that session.

## [0.1.0] - 2026-10-28

### Added
- Dual license `MIT OR Apache-2.0`.
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue and PR templates.
- GitHub Actions CI (fmt, clippy, test) and tag-based release workflow.
- Crate metadata for publishing.

### Changed
- Package renamed from `bc` to `bclaude` (library crate name stays `bc`; the binary is still `bclaude`).
- Profiles guide moved from the README to `docs/profiles.md`.

## [0.1.0]

### Added
- Initial release: profile resolution, per-session plugin and CLAUDE.md composition, library management, TUI dashboard.
