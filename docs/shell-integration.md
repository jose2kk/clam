# Shell Integration

## Setting Up Your Shell

Add this to your `~/.bashrc` or `~/.zshrc`:

```sh
eval "$(clam env)"
```

This exports two environment variables into your current shell session:

- `CLAUDE_CONFIG_DIR` — points Claude Code at the active profile's directory
- `CLAM_PROFILE` — the name of the active profile

It also unsets any existing `CLAUDE_*` and `ANTHROPIC_*` variables to prevent leakage from a previous session.

## `clam run` vs `clam env`

There are two ways to use a profile:

### `clam run` — launch and forget

```sh
clam run
```

Spawns a new process (`claude` by default) with the profile environment. The environment only applies to that process — your shell isn't affected.

Best for: launching Claude Code sessions.

### `eval "$(clam env)"` — modify your shell

```sh
eval "$(clam env)"
```

Exports the profile environment into your current shell. Any subsequent commands in that shell session see the profile variables.

Best for: shell startup config, running multiple commands with profile context.

## Switching Profiles

After running `clam use <name>`, the switch takes effect for new sessions:

- **New terminals** pick up the change automatically (via `eval "$(clam env)"` in your shell rc).
- **Existing terminals** need to re-source: run `eval "$(clam env)"` again, or open a new shell.

## Scripting

### Check the active profile

`clam current` prints the active profile name to stdout and exits `0`. If no profile is active, it prints nothing and exits `1`:

```sh
if profile=$(clam current); then
  echo "Active: $profile"
else
  echo "No active profile"
fi
```

### JSON output

Commands with `--json` flags produce structured output for `jq`:

```sh
# Get the active profile's directory path
clam status --json | jq -r '.path'

# List all profile names
clam list --json | jq -r '.[].name'

# Check which profile is active
clam list --json | jq -r '.[] | select(.active) | .name'
```

### Run arbitrary commands with a profile

`clam run` accepts any command after `--`, not just `claude`:

```sh
# Check what environment a profile sets
clam run --profile work -- env | grep CLAUDE

# Run a script with a specific profile's environment
clam run --profile deploy -- ./my-script.sh
```

## Shell Completions

Generate and install tab completions for your shell:

=== "Bash"

    ```sh
    clam completions bash > ~/.local/share/bash-completion/completions/clam
    ```

=== "Zsh"

    ```sh
    clam completions zsh > ~/.zsh/completions/_clam
    ```

=== "Fish"

    ```sh
    clam completions fish > ~/.config/fish/completions/clam.fish
    ```
