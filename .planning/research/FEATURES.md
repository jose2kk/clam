# Feature Landscape

**Domain:** CLI profile/account management for Claude Code
**Researched:** 2026-04-02
**Reference tools:** aws-cli, kubectl, gh, gcloud, fnm, volta, direnv

## Table Stakes

Features users expect from any CLI profile manager. Missing any of these and the tool feels broken or incomplete. These are patterns established by aws, kubectl, gh, and gcloud that developers already have muscle memory for.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `clmux add <name>` | Every profile tool has a create/add command. aws: `configure --profile`, gcloud: `config configurations create`, gh: `auth login`. Users expect to name and create profiles in one step. | Low | Creates `~/.clmux/profiles/<name>/` directory. Should fail if name already exists (no silent overwrite). |
| `clmux list` | Every tool has a list command (aws: `configure list-profiles`, gcloud: `config configurations list`, kubectl: `config get-contexts`). Users need to see what's available. | Low | Show all profiles with active marker (asterisk or arrow). gcloud uses `IS_ACTIVE` column; kubectl uses `*` prefix. |
| `clmux use <name>` | Core switching verb. gcloud: `config configurations activate`, kubectl: `config use-context`, gh: `auth switch`. This is the primary interaction. | Low | Updates `~/.clmux/state.toml`. Prints confirmation like gcloud does: "Activated profile [work]." |
| `clmux current` | Users need to query the active profile for scripts and prompt customization. kubectl: `config current-context`, gcloud: `config configurations list` (shows active). | Low | Prints just the name to stdout for easy capture in `$()`. |
| `clmux remove <name>` | Every tool supports deletion. Must exist for lifecycle management. | Low | Require confirmation unless `--force` flag. Refuse to remove active profile (switch first). Delete the profile directory. |
| `clmux run [-- command]` | The core value prop: launch Claude Code scoped to a profile. Analogous to `aws --profile X command` or gcloud's `--configuration` flag. Must set `CLAUDE_CONFIG_DIR` and exec into Claude Code. | Medium | `clmux run` (bare) launches Claude Code with active profile. `clmux run --profile work` launches with specific profile. Pass-through args after `--`. |
| `clmux env` | Emit environment variables for shell integration. Pattern used by direnv, fnm, volta, mise. Users do `eval "$(clmux env)"` in their shell RC. | Medium | Output `export CLAUDE_CONFIG_DIR=~/.clmux/profiles/<active>/`. Also useful for scripting and CI. |
| `clmux status` | Show current state: active profile, its path, whether the profile directory exists and has auth state. gh: `auth status` shows logged-in accounts and active tokens. | Low | More verbose than `current`. Show profile name, path, and basic health (directory exists, has expected files). |
| Error on unknown profile | If user runs `clmux use nonexistent`, fail clearly. Every serious CLI does this. | Low | Clear error message with suggestion: "Profile 'foo' not found. Run `clmux list` to see available profiles." |
| Human-readable output by default | aws, gh, gcloud all output human-friendly tables/text by default. Machine-parseable output is secondary. | Low | Colored output to tty, plain to pipes. Follow standard fd detection. |

## Differentiators

Features that go beyond what manual `CLAUDE_CONFIG_DIR` aliases provide. These make clmux worth installing over a few shell aliases.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `clmux show <name>` | Inspect a profile's paths and state in detail. No equivalent exists for manual aliases -- users have to remember where things live and `ls` around. Shows directory path, config files present, last-used timestamp, disk usage. | Low | Distinct from `status` (which shows active profile). `show` inspects any profile by name. |
| Shell prompt integration | Export the active profile name for PS1/PROMPT. `clmux prompt` outputs just the profile name (or empty if none), designed for embedding in shell prompts. Similar to kubectl prompt plugins (kube-ps1). | Low | Tiny feature, high daily-use value. `clmux prompt --format "[%s]"` with optional format string. |
| Profile validation on switch | When switching profiles, verify the target directory is healthy (exists, has expected structure). Warn if auth state appears missing. Manual aliases just blindly set an env var. | Low | Non-blocking warning, not an error. "Warning: Profile 'work' has no auth state. You may need to run `claude` to authenticate." |
| `clmux which` | Show which profile would be used in the current context (respects env var overrides). Debugging tool for "why is this profile active?" Inspired by `which`, `nvm current`, `pyenv version`. | Low | Check `CLMUX_PROFILE` env var override, then fall back to state.toml. |
| Env var override (`CLMUX_PROFILE`) | Allow per-shell or per-command profile override without switching global state. Pattern from `AWS_PROFILE`, `KUBECONFIG`, `GCLOUD_CONFIG`. Essential for running two profiles simultaneously in different terminals. | Low | `CLMUX_PROFILE=work clmux run` uses "work" without changing global active state. `clmux env` respects this too. |
| `clmux clone <source> <target>` | Copy an existing profile to a new name. Useful when setting up a second org account that shares some config. No equivalent in manual alias workflow. | Medium | Deep copy of profile directory. Clear that auth tokens won't transfer (they're tied to the original session). |
| Completions generation | Shell completions for bash, zsh, fish. Expected by power users, clap provides this nearly for free. | Low | `clmux completions bash/zsh/fish`. Clap's `clap_complete` crate handles this. Near-zero effort in Rust/clap. |
| `--json` output flag | Machine-parseable output for scripting. gh uses `--json`, aws uses `--output json`. | Low | Apply to `list`, `status`, `show`. Output JSON to stdout. |

## Anti-Features

Features to explicitly NOT build. These represent scope creep, wrong abstraction level, or premature complexity.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Credential encryption / keychain integration | Claude Code manages its own auth. clmux just points Claude Code at a directory. Adding encryption means owning a security surface that isn't yours. The profile directory permissions (0700) are sufficient for v1. | Set directory permissions to user-only on creation. Revisit if users request it. |
| Automatic shell mutation (hook on cd) | direnv-style auto-switching is magic that confuses users. The project explicitly chose "explicit > magic." Auto-switching profiles based on directory would be surprising and hard to debug. | Provide `eval "$(clmux env)"` for opt-in shell integration. Users control when it runs. |
| TUI / interactive selection | Interactive profile picker (fzf-style) adds a runtime dependency or significant code for marginal value. The target audience types commands, not navigates menus. | Simple `clmux list` + `clmux use <name>`. Users who want fuzzy matching can pipe `clmux list --json` to fzf themselves. |
| Plugin system / extensibility | No hooks, no plugin directories, no event system. Premature abstraction for a tool this focused. | Keep the tool simple. If extensibility is needed later, add specific hooks, not a framework. |
| Cloud sync / cross-machine state | Local-only tool. Syncing profiles across machines requires solving conflict resolution, encryption-in-transit, and server infrastructure. Wrong problem to solve. | Users can manually copy `~/.clmux/` or use their own sync (git, rsync, Syncthing). |
| Profile import/export | YAGNI for v1. Can be added later if there's demand. Profile directories are just directories -- users can tar/cp them. | Document the directory structure so power users can manually import/export. |
| API request proxying | clmux is a profile manager, not a middleware or proxy. Intercepting Claude Code's API calls is a completely different tool (see claude-code-mux on GitHub for that). | Stay in the profile-management lane. |
| Multi-profile simultaneous dashboard | Tools like amux and cmux already handle running multiple Claude Code sessions in parallel with dashboards. clmux manages identity, not sessions. | `clmux run --profile X` in one terminal, `clmux run --profile Y` in another. The OS handles parallelism. |
| Config file editing (settings within profiles) | clmux should not be an editor for Claude Code's internal config files. That's Claude Code's responsibility. | `clmux show <name>` shows the path; user edits with their preferred editor. |
| Windows support in v1 | macOS + Linux first. Windows has different path conventions, shell integration patterns, and a smaller Claude Code user base. | Defer. Rust makes eventual Windows support straightforward but not free. |

## Feature Dependencies

```
clmux add ─────────── (foundation for everything)
  │
  ├── clmux list      (needs profiles to exist)
  ├── clmux use       (needs profiles to exist)
  │     │
  │     ├── clmux current  (needs active profile concept)
  │     ├── clmux env      (needs active profile concept)
  │     ├── clmux status   (needs active profile concept)
  │     ├── clmux prompt   (needs active profile concept)
  │     └── clmux which    (needs active profile + env var override)
  │
  ├── clmux show      (needs profiles to exist)
  ├── clmux remove    (needs profiles to exist)
  ├── clmux clone     (needs profiles to exist)
  └── clmux run       (needs active profile OR --profile flag)

clmux completions ─── (independent, can ship anytime)
--json flag ────────── (independent, apply to list/status/show)
CLMUX_PROFILE env ─── (independent, modifies resolution logic in use/current/env/run)
```

## MVP Recommendation

**Phase 1 -- Core lifecycle (ship first):**
1. `clmux add <name>` -- create profile with isolated directory
2. `clmux list` -- show all profiles with active marker
3. `clmux use <name>` -- switch active profile
4. `clmux current` -- print active profile name
5. `clmux remove <name>` -- delete profile (with confirmation)
6. State management via `~/.clmux/state.toml`

**Phase 2 -- Launch and integrate:**
7. `clmux run [-- args]` -- launch Claude Code with profile's `CLAUDE_CONFIG_DIR`
8. `clmux env` -- emit shell exports for eval integration
9. `clmux status` -- verbose active profile info

**Phase 3 -- Polish:**
10. `clmux show <name>` -- inspect any profile's details
11. `clmux completions` -- shell completions (bash, zsh, fish)
12. `CLMUX_PROFILE` env var override
13. `clmux which` -- debug profile resolution
14. `clmux prompt` -- shell prompt helper
15. `--json` flag on list/status/show

**Defer:** `clmux clone` (nice-to-have, not essential for v1)

**Rationale:** Phase 1 establishes the data model and core CRUD. Phase 2 adds the actual value (launching Claude Code with isolation). Phase 3 adds developer experience polish. This order lets you validate the directory isolation approach in Phase 1 before committing to the launch mechanism in Phase 2.

## Sources

- [AWS CLI Named Profiles](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html) -- profile-per-section in config files, `--profile` flag, `AWS_PROFILE` env var
- [kubectl config contexts](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_use-context/) -- context switching, current-context query
- [gh auth switch](https://cli.github.com/manual/gh_auth_switch) -- multi-account auth, `--user` flag, `auth status` display
- [gcloud configurations](https://docs.google.com/sdk/docs/configurations) -- named configurations, activate/create/list, `--configuration` per-command override
- [Claude Code environment variables](https://code.claude.com/docs/en/env-vars) -- `CLAUDE_CONFIG_DIR` as primary isolation mechanism, auth env vars
- [CLAUDE_CONFIG_DIR issues on GitHub](https://github.com/anthropics/claude-code/issues/3833) -- known bugs with config dir behavior, local `.claude/` still created
- [direnv](https://direnv.net/) -- `eval` shell integration pattern
- [fnm (Fast Node Manager)](https://blog.logrocket.com/fnm-rust-built-node-js-version-manager/) -- Rust CLI tool patterns, shell integration, version switching UX
