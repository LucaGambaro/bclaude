# Changelog

All notable changes to this project are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
