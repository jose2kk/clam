---
description: Configure clam with environment variables, file layout, profile directories, and config inheritance for Claude Code.
---

# Configuration

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `CLAM_HOME` | Override the base data directory | `~/.clam/` |
| `CLAM_PROFILE` | Set by `clam run` and `clam env` — indicates the active profile name | — |
| `CLAUDE_CONFIG_DIR` | Set by `clam run` and `clam env` — tells Claude Code where to find profile state | — |
| `NO_COLOR` | Disable colored output | — |

### `CLAM_HOME`

By default, clam stores everything under `~/.clam/`. Set `CLAM_HOME` to use a different location:

```sh
export CLAM_HOME="/path/to/my/clam-data"
```

All paths (config, state, profiles) are resolved relative to this directory.

## File Layout

```
~/.clam/                     # or $CLAM_HOME
├── config.toml              # Profile registry
├── state.toml               # Active profile pointer
└── profiles/
    ├── work/                # Profile directory (0700 permissions)
    │   ├── settings.json    # (if --inherit was used, symlinked)
    │   └── ...              # Claude Code state files
    └── personal/
        └── ...
```

### `config.toml`

The profile registry. Lists all known profiles:

```toml
[[profiles]]
name = "work"

[[profiles]]
name = "personal"
```

### `state.toml`

The active profile pointer:

```toml
active = "work"
```

### Profile Directories

Each profile gets its own directory at `~/.clam/profiles/<name>/`. This directory is passed as `CLAUDE_CONFIG_DIR` when launching Claude Code, so all of Claude Code's state (auth, sessions, history) is scoped to the profile.

Profile directories are created with `0700` permissions (owner-only access).

## Config Inheritance

When you create a profile with `--inherit`, clam symlinks shared config files from `~/.claude/` into the profile directory. This lets you share settings, custom agents, skills, and tools across profiles.

### What gets inherited

Everything in `~/.claude/` **except** session-specific items:

| Skipped Item | Reason |
|-------------|--------|
| `statsig` | Telemetry state |
| `telemetry` | Telemetry data |
| `worktrees` | Git worktree state |
| `tasks` | Task tracking |
| `sessions` | Session data |
| `session-env` | Session environment |
| `shell-snapshots` | Shell state |
| `cache` | Cache data |
| `paste-cache` | Paste buffer |
| `stats-cache.json` | Statistics cache |
| `mcp-needs-auth-cache.json` | MCP auth cache |
| `history.jsonl` | Command history |
| `file-history` | File access history |
| `debug` | Debug logs |
| `plans` | Plan data |
| `backups` | Backup data |

### How symlinks work

Inherited items are symlinked, not copied. Changes to the original files in `~/.claude/` are reflected in all profiles that inherited them. If you want a profile to diverge, delete the symlink and create a regular file in its place.

## Profile Names

Profile names must match `[a-zA-Z0-9_-]+`:

- Letters (a-z, A-Z)
- Numbers (0-9)
- Hyphens (`-`)
- Underscores (`_`)

Names like `../evil`, `.`, empty strings, or names with spaces are rejected.

## Atomic Writes

All config and state files are written atomically using a temp-file-and-rename pattern. If clam crashes mid-write, you won't end up with a corrupted `config.toml` or `state.toml`.
