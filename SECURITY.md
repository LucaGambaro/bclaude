# Security Policy

## Supported versions

Only the latest released version receives security fixes.

## Reporting a vulnerability

Please do **not** open a public issue. Use GitHub's private reporting: **Security → Report a vulnerability** on https://github.com/LucaGambaro/bclaude/security/advisories/new.

Include a description, steps to reproduce and the affected version. You can expect an acknowledgement within a few days. Once a fix is released the advisory is published and you are credited, unless you prefer otherwise.

## Scope

`bclaude` creates per-session directories and symlinks under `~/.betterclaude` and spawns the `claude` binary (overridable with `BC_CLAUDE_BIN`). Issues such as path traversal in library names, unsafe symlink handling or command injection through configuration are in scope.
