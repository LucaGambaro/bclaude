# Contributing to bclaude

Thanks for your interest! Bug reports, ideas and pull requests are all welcome.

## Getting started

```
git clone https://github.com/LucaGambaro/bclaude
cd bclaude
cargo test
```

Use `BC_HOME` to point `bclaude` at a throwaway data directory while trying things out (`BC_HOME=/tmp/bc-test cargo run -- ls`).

## Before opening a PR

Run all of these; CI runs the same checks:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Code rules

The layout is described in [CLAUDE.md](CLAUDE.md). In short:

- Unit-test everything. Inject paths, cwd and writers instead of reading the environment inside logic.
- `cli` and `commands` only do I/O and delegate; no business logic there.
- Errors propagate as `BcError` and are printed once, in `main`. Do not log and return in the same place.
- No `unsafe`, and no environment mutation in tests (use an injected `Paths`).
- Keep [README.md](README.md), [docs/](docs/) and CLAUDE.md in sync with any behaviour change, and add a line to [CHANGELOG.md](CHANGELOG.md) under `Unreleased`.

## Pull requests

- Keep each PR focused on one change.
- Describe the problem and how you verified the fix.
- By contributing you agree that your work is licensed under `MIT OR Apache-2.0`, as stated in the README.

## Reporting bugs and requesting features

Open an issue using the templates. For bugs, include your OS, `bclaude` version, the output of `bclaude which` and `bclaude --dry-run`, and what you expected. Security problems: see [SECURITY.md](SECURITY.md).
