# clmux

Manage multiple Claude Code accounts on a single machine.

Each account gets its own isolated directory for auth, config, and session data — so two Claude Code accounts never interfere with each other.

Think `aws configure` + `kubectl config use-context`, but for Claude Code.

## Install

Requires [Rust](https://rustup.rs/):

```sh
cargo install --path .
```

Or build from source:

```sh
git clone https://github.com/tepuii-tech/clmux.git
cd clmux
cargo build --release
# Binary at target/release/clmux
```

## Quick Start

```sh
# Create your first profile (auto-activates)
clmux add work

# Or inherit your global Claude Code config (settings, extensions, tools)
clmux add work --inherit

# Create a second profile
clmux add personal

# See all profiles (* = active)
clmux list
# * work
#   personal

# Switch profiles
clmux use personal

# Launch Claude Code with the active profile
clmux run

# Launch with a specific profile (without switching)
clmux run --profile work
```

## How It Works

Each profile maps to an isolated directory at `~/.clmux/profiles/<name>/`. When you run `clmux run`, it sets `CLAUDE_CONFIG_DIR` to point Claude Code at that directory and strips all `CLAUDE_*` / `ANTHROPIC_*` env vars from the parent shell to prevent leakage.

```
~/.clmux/
├── config.toml      # Profile registry
├── state.toml       # Active profile pointer
└── profiles/
    ├── work/        # Claude Code state for "work" account
    └── personal/    # Claude Code state for "personal" account
```

## Commands

### Profile Management

```sh
clmux add <name>              # Create a new profile
clmux add <name> --inherit    # Create and inherit global Claude Code config
clmux list                    # List all profiles (* marks active)
clmux use <name>              # Switch active profile
clmux current                 # Print active profile name (for scripts)
clmux remove <name>           # Remove a profile (prompts for confirmation)
clmux remove <name> --force   # Remove without confirmation
clmux status                  # Show active profile details
```

### Launch & Shell Integration

```sh
# Launch Claude Code with active profile
clmux run

# Launch with a specific profile (doesn't change active)
clmux run --profile work

# Pass arguments through to claude
clmux run -- --help

# Run any command with profile environment
clmux run -- env | grep CLAUDE

# Set up your current shell session
eval "$(clmux env)"
```

### Developer Experience

```sh
# Machine-parseable output
clmux list --json
clmux status --json
clmux env --json

# Shell completions
clmux completions bash > ~/.local/share/bash-completion/completions/clmux
clmux completions zsh > ~/.zsh/completions/_clmux
clmux completions fish > ~/.config/fish/completions/clmux.fish
```

## Shell Integration

Add to your `~/.bashrc` or `~/.zshrc`:

```sh
eval "$(clmux env)"
```

This exports `CLAUDE_CONFIG_DIR` and `CLMUX_PROFILE` for the active profile. Run `clmux use <name>` in another terminal to switch, then source again or open a new shell.

## Scripting

`clmux current` prints the active profile name to stdout and exits 0. If no profile is active, it prints nothing and exits 1:

```sh
if profile=$(clmux current); then
  echo "Active: $profile"
else
  echo "No active profile"
fi
```

`--json` flags produce structured output for `jq`:

```sh
# Get the active profile's path
clmux status --json | jq -r '.path'

# List all profile names
clmux list --json | jq -r '.[].name'
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CLMUX_HOME` | Override the default `~/.clmux/` data directory |
| `CLMUX_PROFILE` | Set by `clmux run` and `clmux env` to the active profile name |
| `CLAUDE_CONFIG_DIR` | Set by `clmux run` and `clmux env` to the profile's directory |
| `NO_COLOR` | Disable colored output (respected automatically) |

## Inheriting Global Config

By default, new profiles start empty. Use `--inherit` to symlink your global Claude Code config (`~/.claude/`) into the profile:

```sh
clmux add work --inherit
```

This symlinks shared items like `settings.json`, custom agents, skills, and tools (e.g., GSD) so they're available in the new profile. Session-specific items (auth, history, cache) are skipped — those stay isolated per profile.

Use `--inherit` if you have custom Claude Code extensions you want across all profiles. Skip it for a clean slate.

## Profile Names

Profile names must contain only letters, numbers, hyphens, and underscores. Names like `../evil`, `.`, or names with spaces are rejected.

## License

MIT
