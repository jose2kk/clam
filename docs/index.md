# clam

**Manage multiple Claude Code accounts on a single machine.**

Each account gets its own isolated directory for auth, config, and session data — so two Claude Code accounts never interfere with each other.

Think `aws configure` + `kubectl config use-context`, but for Claude Code.

## The Problem

Claude Code stores auth tokens, session history, and config in a single directory (`~/.claude/`). If you use Claude Code with multiple accounts — say, a work account and a personal one — they collide. Logging into one overwrites the other's state.

## The Solution

clam gives each account its own isolated profile directory. Switching between accounts is a single command, and `clam run` launches Claude Code pointing at the right directory automatically.

```
~/.clam/
├── config.toml      # Profile registry
├── state.toml       # Active profile pointer
└── profiles/
    ├── work/        # Claude Code state for "work" account
    └── personal/    # Claude Code state for "personal" account
```

## Quick Example

```sh
# Create two profiles
clam add work --inherit
clam add personal

# Switch between them
clam use work

# Launch Claude Code with the active profile
clam run
```

## Features

- **Profile isolation** — each account's auth, config, and sessions live in separate directories
- **Explicit switching** — no accidental cross-account leakage
- **Config inheritance** — optionally share settings, extensions, and tools across profiles via `--inherit`
- **Environment sanitization** — `clam run` strips stale `CLAUDE_*` and `ANTHROPIC_*` env vars
- **Shell integration** — `eval "$(clam env)"` sets up your current shell session
- **JSON output** — machine-readable output for scripting with `--json` flags
- **Shell completions** — tab completion for bash, zsh, and fish

## Next Steps

- [Getting Started](getting-started.md) — install clam and create your first profile
- [Commands](commands.md) — full reference for every subcommand
- [Configuration](configuration.md) — environment variables and file layout
- [Shell Integration](shell-integration.md) — set up your shell and scripting patterns
