---
description: Install clam, create your first Claude Code profile, and learn how to switch between multiple accounts.
---

# Getting Started

## Installation

### Homebrew (macOS / Linux)

```sh
brew install jose2kk/tap/clam
```

### Shell installer (macOS / Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jose2kk/clam/releases/latest/download/clam-installer.sh | sh
```

### PowerShell installer (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/jose2kk/clam/releases/latest/download/clam-installer.ps1 | iex"
```

### Cargo (pre-built binary)

If you have [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall clam
```

### Cargo (from source)

```sh
cargo install clam
```

### From source (manual build)

```sh
git clone https://github.com/jose2kk/clam.git
cd clam
cargo build --release
# Binary at target/release/clam
```

The result is a single statically-linked binary with no runtime dependencies.

## Create Your First Profile

```sh
clam add work
```

This creates a profile directory at `~/.clam/profiles/work/` and auto-activates it (since it's the first profile).

### Inherit your global config

If you already have Claude Code configured with custom settings, extensions, or tools, pass `--inherit` to symlink them into the new profile:

```sh
clam add work --inherit
```

This shares items like `settings.json`, custom agents, skills, and MCP tools. Session-specific data (auth tokens, history, cache) stays isolated.

!!! tip
    Use `--inherit` if you have custom Claude Code extensions you want across all profiles. Skip it for a clean slate.

## Create Additional Profiles

```sh
clam add personal
```

Additional profiles are created but not auto-activated. The first profile you create remains active until you explicitly switch.

## Switch Profiles

```sh
clam use personal
```

## See Your Profiles

```sh
clam list
```

Output:

```
  work
* personal
```

The `*` marks the active profile.

## Launch Claude Code

```sh
clam run
```

This launches `claude` with `CLAUDE_CONFIG_DIR` pointing at the active profile's directory, and strips any stale `CLAUDE_*` / `ANTHROPIC_*` env vars from your shell.

To launch with a specific profile without switching:

```sh
clam run --profile work
```

## Set Up Shell Integration

Add this to your `~/.bashrc` or `~/.zshrc`:

```sh
eval "$(clam env)"
```

This exports `CLAUDE_CONFIG_DIR` and `CLAM_PROFILE` for the active profile into your current shell session. See [Shell Integration](shell-integration.md) for more details.
