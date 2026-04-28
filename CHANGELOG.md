# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Homebrew, shell, and PowerShell installers via cargo-dist
- GitHub Attestations for supply chain security
- musl targets for static Linux binaries

## [0.1.0] - 2025-06-01

### Added

- Profile management: `add`, `remove`, `use`, `list`, `current`, `status`
- Config inheritance with `--inherit` flag (symlinks shared Claude Code config)
- `clam run` to launch Claude Code with isolated profile environment
- `clam env` for shell integration (`eval "$(clam env)"`)
- Environment sanitization (strips stale `CLAUDE_*` / `ANTHROPIC_*` vars)
- JSON output for scripting (`--json` flags)
- Shell completions for bash, zsh, and fish
- Per-profile isolation of `projects/` and `todos/` directories
- `clam repair` to fix broken profile state
- CI pipeline: fmt, clippy, deny, test (macOS + Linux), MSRV (1.81), docs, coverage
