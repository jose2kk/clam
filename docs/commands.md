# Commands

## `clam add`

Create a new profile.

```sh
clam add <name> [--inherit]
```

| Argument | Description |
|----------|-------------|
| `<name>` | Profile name (letters, numbers, hyphens, underscores only) |
| `--inherit` | Symlink global Claude Code config (`~/.claude/`) into the profile |

Creates a profile directory at `~/.clam/profiles/<name>/` with `0700` permissions. If this is the first profile, it is auto-activated.

With `--inherit`, shared config files from `~/.claude/` are symlinked into the profile. Session-specific items (auth, history, cache, telemetry) are skipped to maintain isolation.

```sh
# Basic profile
clam add personal

# Profile with shared config
clam add work --inherit
```

---

## `clam list`

List all profiles.

```sh
clam list [--json]
```

| Argument | Description |
|----------|-------------|
| `--json` | Output as JSON array |

Text output marks the active profile with `*`:

```
* work
  personal
```

JSON output:

```json
[
  { "name": "work", "active": true },
  { "name": "personal", "active": false }
]
```

---

## `clam use`

Switch the active profile.

```sh
clam use <name>
```

| Argument | Description |
|----------|-------------|
| `<name>` | Profile name to activate |

Updates `~/.clam/state.toml` to point at the given profile. Does not affect running Claude Code sessions — only new sessions pick up the change.

---

## `clam current`

Print the active profile name.

```sh
clam current
```

Prints the name to stdout and exits `0`. If no profile is active, prints nothing and exits `1`.

Useful for scripting:

```sh
if profile=$(clam current); then
  echo "Active: $profile"
else
  echo "No active profile"
fi
```

---

## `clam remove`

Remove a profile.

```sh
clam remove <name> [--force]
```

| Argument | Description |
|----------|-------------|
| `<name>` | Profile name to remove |
| `--force` | Skip confirmation prompt |

Deletes the profile directory and removes it from the registry. Cannot remove the currently active profile — switch to another profile first.

Without `--force`, prompts for confirmation. In non-interactive contexts (pipes, scripts), `--force` is required.

```sh
# Interactive removal
clam remove old-profile

# Scripted removal
clam remove old-profile --force
```

---

## `clam status`

Show details about the active profile.

```sh
clam status [--json]
```

| Argument | Description |
|----------|-------------|
| `--json` | Output as JSON object |

Text output:

```
Profile: work
Path:    /Users/you/.clam/profiles/work
```

If no profile is active, prints an error to stderr and exits `1`.

---

## `clam run`

Launch a command with the profile's environment.

```sh
clam run [--profile <name>] [-- <command> [args...]]
```

| Argument | Description |
|----------|-------------|
| `--profile <name>` | Use a specific profile without switching the active one |
| `-- <command>` | Command to run (defaults to `claude`) |

Sets `CLAUDE_CONFIG_DIR` to the profile directory, sets `CLAM_PROFILE` to the profile name, and strips all `CLAUDE_*` and `ANTHROPIC_*` env vars from the parent shell. Then `exec()`s into the target command (replacing the clam process).

```sh
# Launch Claude Code with active profile
clam run

# Launch with a specific profile
clam run --profile work

# Pass arguments to claude
clam run -- --help

# Run any command with profile environment
clam run -- env | grep CLAUDE
```

---

## `clam env`

Print shell environment exports.

```sh
clam env [--json]
```

| Argument | Description |
|----------|-------------|
| `--json` | Output as JSON object instead of shell exports |

Outputs `export` and `unset` statements suitable for `eval`:

```sh
eval "$(clam env)"
```

This unsets any existing `CLAUDE_*` and `ANTHROPIC_*` variables, then exports `CLAUDE_CONFIG_DIR` and `CLAM_PROFILE` for the active profile.

---

## `clam completions`

Generate shell completion scripts.

```sh
clam completions <shell>
```

| Argument | Description |
|----------|-------------|
| `<shell>` | One of: `bash`, `zsh`, `fish` |

```sh
# Bash
clam completions bash > ~/.local/share/bash-completion/completions/clam

# Zsh
clam completions zsh > ~/.zsh/completions/_clam

# Fish
clam completions fish > ~/.config/fish/completions/clam.fish
```
