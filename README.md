# clam

[![CI](https://github.com/jose2kk/clam/actions/workflows/ci.yml/badge.svg)](https://github.com/jose2kk/clam/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/clam.svg)](https://crates.io/crates/clam)
[![License](https://img.shields.io/crates/l/clam.svg)](LICENSE-MIT)

Manage multiple Claude Code accounts on a single machine.

**[Documentation](https://clam.jose2kk.com)**

Each account gets its own isolated directory for auth, config, and session data — so two Claude Code accounts never interfere with each other.

Think `aws configure` + `kubectl config use-context`, but for Claude Code.

## Install

### Homebrew (macOS / Linux)

```sh
brew install jose2kk/tap/clam
```

### Shell installer (macOS / Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jose2kk/clam/releases/latest/download/clam-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/jose2kk/clam/releases/latest/download/clam-installer.ps1 | iex"
```

### Cargo

```sh
cargo binstall clam   # pre-built binary (requires cargo-binstall)
cargo install clam    # compile from source
```

## Quick Start

```sh
# Create your first profile (auto-activates)
clam add work

# Or inherit your global Claude Code config (settings, extensions, tools)
clam add work --inherit

# Create a second profile
clam add personal

# See all profiles (* = active)
clam list
# * work
#   personal

# Switch profiles
clam use personal

# Launch Claude Code with the active profile
clam run

# Launch with a specific profile (without switching)
clam run --profile work
```

## How It Works

Each profile maps to an isolated directory at `~/.clam/profiles/<name>/`. When you run `clam run`, it sets `CLAUDE_CONFIG_DIR` to point Claude Code at that directory and strips all `CLAUDE_*` / `ANTHROPIC_*` env vars from the parent shell to prevent leakage.

```
~/.clam/
├── config.toml      # Profile registry
├── state.toml       # Active profile pointer
└── profiles/
    ├── work/        # Claude Code state for "work" account
    └── personal/    # Claude Code state for "personal" account
```

## Commands

### Profile Management

```sh
clam add <name>              # Create a new profile
clam add <name> --inherit    # Create and inherit global Claude Code config
clam list                    # List all profiles (* marks active)
clam list --names            # Print only profile names, one per line
clam use <name>              # Switch active profile
clam current                 # Print active profile name (for scripts)
clam remove <name>           # Remove a profile (prompts for confirmation)
clam remove <name> --force   # Remove without confirmation
clam status                  # Show active profile details
clam repair                  # Fix legacy profiles with shared session data
```

### Launch & Shell Integration

```sh
# Launch Claude Code with active profile
clam run

# Launch with a specific profile (doesn't change active)
clam run --profile work

# Pass arguments through to claude
clam run -- --help

# Run any command with profile environment
clam run -- env | grep CLAUDE

# Set up your current shell session
eval "$(clam env)"
```

### Developer Experience

```sh
# Machine-parseable output
clam list --json
clam status --json
clam env --json

# Shell completions
clam completions bash > ~/.local/share/bash-completion/completions/clam
clam completions zsh > ~/.zsh/completions/_clam
clam completions fish > ~/.config/fish/completions/clam.fish
```

## Shell Integration

Add to your `~/.bashrc` or `~/.zshrc`:

```sh
eval "$(clam env)"
```

This exports `CLAUDE_CONFIG_DIR` and `CLAM_PROFILE` for the active profile. Run `clam use <name>` in another terminal to switch, then source again or open a new shell.

## Scripting

`clam current` prints the active profile name to stdout and exits 0. If no profile is active, it prints nothing and exits 1:

```sh
if profile=$(clam current); then
  echo "Active: $profile"
else
  echo "No active profile"
fi
```

`--json` flags produce structured output for `jq`:

```sh
# Get the active profile's path
clam status --json | jq -r '.path'

# List all profile names
clam list --json | jq -r '.[].name'
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CLAM_HOME` | Override the default `~/.clam/` data directory |
| `CLAM_PROFILE` | Set by `clam run` and `clam env` to the active profile name |
| `CLAUDE_CONFIG_DIR` | Set by `clam run` and `clam env` to the profile's directory |
| `NO_COLOR` | Disable colored output (respected automatically) |

## Inheriting Global Config

By default, new profiles start empty. Use `--inherit` to symlink your global Claude Code config (`~/.claude/`) into the profile:

```sh
clam add work --inherit
```

This symlinks shared items like `settings.json`, custom agents, skills, and tools (e.g., GSD) so they're available in the new profile. Session-specific items (auth, history, cache) are skipped — those stay isolated per profile.

Use `--inherit` if you have custom Claude Code extensions you want across all profiles. Skip it for a clean slate.

## Profile Names

Profile names must contain only letters, numbers, hyphens, and underscores. Names like `../evil`, `.`, or names with spaces are rejected.

## License

MIT or Apache-2.0
