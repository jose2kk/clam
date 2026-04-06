# Architecture Research

**Domain:** CLI profile/config management tool (Rust)
**Researched:** 2026-04-02
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Entry (main.rs)                      │
│                  clap derive parse + dispatch                │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │  add     │ │  list    │ │  switch  │ │  remove  │  ...   │
│  │  cmd     │ │  cmd     │ │  cmd     │ │  cmd     │       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘       │
│       │            │            │            │              │
├───────┴────────────┴────────────┴────────────┴──────────────┤
│                     Profile Manager                          │
│          (profile CRUD, validation, active tracking)         │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Config      │  │   State      │  │  Filesystem  │       │
│  │  (profiles    │  │  (active     │  │  (profile    │       │
│  │   .toml)      │  │   state.toml)│  │   dirs)      │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    ~/.clmux/                                  │
│  ├── config.toml            (profile registry)               │
│  ├── state.toml             (active profile pointer)         │
│  └── profiles/                                               │
│      ├── personal/          (-> CLAUDE_CONFIG_DIR)           │
│      │   ├── .credentials.json                               │
│      │   ├── .claude.json                                    │
│      │   ├── projects/                                       │
│      │   ├── statsig/                                        │
│      │   └── ...                                             │
│      └── work/                                               │
│          └── ...                                             │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **CLI Layer** | Parse args, validate input, dispatch to commands, format output | `clap` derive macros, one function per subcommand |
| **Command Handlers** | Orchestrate a single user action (add profile, switch, list, etc.) | Thin functions that call into Profile Manager and format results |
| **Profile Manager** | Core business logic: CRUD profiles, resolve active profile, validate state consistency | Pure Rust module, no I/O awareness -- receives paths, returns results |
| **Config Store** | Read/write config.toml (profile registry: name, metadata) | `serde` + `toml` crate, single struct `Config { profiles: Vec<Profile> }` |
| **State Store** | Read/write state.toml (which profile is active) | `serde` + `toml` crate, single struct `State { active: Option<String> }` |
| **Filesystem Ops** | Create/remove profile directories, verify directory integrity | `std::fs`, thin wrapper for testability |
| **Error Handling** | Typed errors, user-friendly messages, exit codes | `thiserror` for library errors, mapped to user messages at CLI boundary |

## Recommended Project Structure

```
src/
├── main.rs              # Entry point: parse CLI, dispatch, handle top-level errors
├── cli.rs               # clap structs: Cli, Commands enum, per-command arg structs
├── commands/            # One file per subcommand
│   ├── mod.rs           # Re-exports
│   ├── add.rs           # `clmux add <name>`
│   ├── list.rs          # `clmux list`
│   ├── switch.rs        # `clmux switch <name>`
│   ├── remove.rs        # `clmux remove <name>`
│   ├── show.rs          # `clmux show` (current profile info)
│   ├── env.rs           # `clmux env` (print env vars for eval)
│   └── run.rs           # `clmux run [-- cmd]` (exec with profile env)
├── profile.rs           # Profile manager: CRUD logic, validation
├── config.rs            # Config file I/O (config.toml read/write)
├── state.rs             # State file I/O (state.toml read/write)
├── paths.rs             # Path resolution: ~/.clmux, profile dirs, XDG awareness
├── shell.rs             # Shell integration: env var generation, process spawning
└── error.rs             # Error types, exit code mapping
```

### Structure Rationale

- **Flat module layout:** Matches the project constraint of "minimal abstraction, no trait-heavy design." Each file is a clear unit of responsibility. No nested module trees.
- **commands/ directory:** The only directory because there are 7+ subcommands. One file per command keeps each handler focused and independently readable. This is the pattern used by `cargo`, `gh`, and `kubectl` internally.
- **Separate config.rs and state.rs:** Config (profile registry) and state (active pointer) are separate files on disk and separate concerns. Config changes rarely (add/remove profile). State changes often (switch). Separating them avoids write contention and makes the mental model clearer.
- **paths.rs as a dedicated module:** Path resolution is the single most important thing to get right in a tool like this. Centralizing all path logic (home dir, profile dirs, config file paths) prevents path construction from leaking into every module. This is the pattern AWS CLI and kubectl use internally.

## Architectural Patterns

### Pattern 1: Layered Config Resolution (from AWS CLI / kubectl)

**What:** Configuration values resolve through a priority stack: defaults -> config file -> env vars -> CLI args. Higher priority wins.
**When to use:** Any time clmux needs to determine where its own data lives or which profile is active.
**Trade-offs:** More predictable for users (matches every major CLI tool), but adds a resolution layer. Worth it even for v1.

For clmux specifically, the layers are:
1. Default: `~/.clmux/` as home directory
2. Config file: `config.toml` for profile definitions
3. Env var: `CLMUX_HOME` to relocate the entire clmux directory
4. CLI arg: `--profile <name>` to override active profile for one invocation

```rust
/// Resolve the clmux home directory
pub fn clmux_home() -> PathBuf {
    if let Ok(home) = std::env::var("CLMUX_HOME") {
        PathBuf::from(home)
    } else {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".clmux")
    }
}
```

### Pattern 2: Pointer-Based Profile Switching (from kubectl contexts)

**What:** The active profile is a string pointer in a state file, not a symlink or directory rename. Switching profiles writes one field in `state.toml`. No files are moved or copied.
**When to use:** Always. This is how `kubectl config use-context` and `aws configure` work.
**Trade-offs:** Fast, atomic, no risk of data corruption from interrupted moves. The trade-off is that the "current profile" is only meaningful when read by clmux -- the filesystem itself doesn't reflect which profile is active without consulting state.toml.

```toml
# ~/.clmux/state.toml
active = "work"
```

```rust
#[derive(Serialize, Deserialize)]
pub struct State {
    pub active: Option<String>,
}
```

### Pattern 3: Environment Injection (from nvm, direnv, aws-cli)

**What:** clmux makes Claude Code use a specific profile by setting `CLAUDE_CONFIG_DIR` to point at that profile's directory. Two mechanisms: `eval $(clmux env)` for shell integration, or `clmux run -- claude` for subprocess wrapping.
**When to use:** Every time the user wants Claude Code to use a specific profile.
**Trade-offs:** `eval` approach mutates the current shell (user must opt in, matches nvm/pyenv convention). `run` approach is self-contained but spawns a subprocess. Both are needed; `run` is simpler for v1.

```rust
// clmux env: prints export statements
pub fn print_env(profile_dir: &Path) {
    println!("export CLAUDE_CONFIG_DIR=\"{}\"", profile_dir.display());
}

// clmux run: exec with env set
pub fn run_with_profile(profile_dir: &Path, args: &[String]) -> Result<ExitStatus> {
    Command::new(&args[0])
        .args(&args[1..])
        .env("CLAUDE_CONFIG_DIR", profile_dir)
        .status()
        .map_err(Into::into)
}
```

### Pattern 4: Registry + Isolated Directories (from AWS credentials + config)

**What:** A central registry file (`config.toml`) maps profile names to metadata. Each profile's actual data lives in an isolated directory (`profiles/<name>/`). The registry is an index; the directories are the data.
**When to use:** This is the core isolation mechanism for clmux.
**Trade-offs:** Slightly more complex than a single file, but the isolation guarantee is the entire product value. AWS CLI splits config from credentials for security; clmux splits registry from profile data for isolation.

```toml
# ~/.clmux/config.toml
[[profiles]]
name = "personal"
created = "2026-04-02T10:00:00Z"

[[profiles]]
name = "work"
created = "2026-04-02T10:05:00Z"
```

## Data Flow

### Profile Add Flow

```
User: `clmux add work`
    |
    v
CLI Layer (cli.rs)
    | parse args, extract name="work"
    v
add command (commands/add.rs)
    | validate name not taken
    v
Profile Manager (profile.rs)
    | check config.toml for duplicates
    | create ~/.clmux/profiles/work/
    | append to config.toml
    v
Config Store (config.rs) + Filesystem (std::fs)
    | write TOML, create directory
    v
Output: "Profile 'work' created at ~/.clmux/profiles/work/"
```

### Profile Switch Flow

```
User: `clmux switch work`
    |
    v
CLI Layer
    | parse args, extract name="work"
    v
switch command (commands/switch.rs)
    | validate profile exists in config.toml
    v
State Store (state.rs)
    | write active = "work" to state.toml
    v
Output: "Switched to profile 'work'"
```

### Claude Code Launch Flow

```
User: `clmux run -- claude`
    |
    v
CLI Layer
    | parse args, extract command=["claude"]
    v
run command (commands/run.rs)
    | read state.toml -> active = "work"
    | resolve profile dir: ~/.clmux/profiles/work/
    v
Shell module (shell.rs)
    | spawn `claude` with CLAUDE_CONFIG_DIR=~/.clmux/profiles/work/
    v
Claude Code starts, reads config from ~/.clmux/profiles/work/
    | auth, sessions, history all scoped to this directory
    v
Process exits, clmux returns exit code
```

### Key Data Flows

1. **Config read path:** Almost every command reads `config.toml` to verify profile existence. This is a hot path -- keep it simple (read file, deserialize, return).
2. **State write path:** Only `switch` writes `state.toml`. This is the simplest write in the system -- one field change.
3. **Directory creation:** Only `add` creates directories. Only `remove` deletes them. These are the riskiest operations (filesystem mutation) and need the most validation.
4. **Environment propagation:** `env` and `run` both resolve the active profile directory and either print or inject `CLAUDE_CONFIG_DIR`. This is the bridge between clmux and Claude Code.

## Integration Points

### External: Claude Code

| Integration | Mechanism | Notes |
|-------------|-----------|-------|
| Profile isolation | `CLAUDE_CONFIG_DIR` env var | This is the **only** integration point. clmux sets this var, Claude Code reads it. No API, no IPC, no shared state. |
| Auth state | Filesystem (`.credentials.json` inside profile dir) | Claude Code manages its own auth. clmux just points it at the right directory. |
| Project-local config | NOT managed by clmux | Claude Code always creates `.claude/` in the workspace regardless of `CLAUDE_CONFIG_DIR`. This is expected and fine -- project config is per-project, not per-account. |

### External: User's Shell

| Integration | Mechanism | Notes |
|-------------|-----------|-------|
| `eval $(clmux env)` | stdout env var exports | User opts in. Mutates current shell session. |
| `clmux run -- cmd` | subprocess with env set | Self-contained. No shell mutation. Preferred for v1. |
| Shell completions | `clap`'s built-in completion generation | Free from clap. Generate for bash/zsh/fish. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI -> Commands | Direct function call | cli.rs matches the subcommand, calls the handler function |
| Commands -> Profile Manager | Function call with paths | Commands resolve paths via `paths.rs`, pass to profile logic |
| Profile Manager -> Config/State | Read/write structs | Profile manager works with deserialized structs, never raw TOML |
| Commands -> Shell | Function call | Only `env` and `run` commands touch shell.rs |

## Anti-Patterns

### Anti-Pattern 1: Symlink-Based Profile Switching

**What people do:** Make `~/.claude` a symlink that points to the active profile directory. Switch by re-pointing the symlink.
**Why it's wrong:** Race conditions if Claude Code is running during switch. Symlinks interact poorly with some tools. Not atomic on all filesystems. If the tool crashes mid-switch, state is corrupt.
**Do this instead:** Pointer in state.toml + CLAUDE_CONFIG_DIR env var injection. The filesystem layout never changes; only the env var changes per-session.

### Anti-Pattern 2: Copying Files Between Profiles

**What people do:** Copy auth tokens or config files from one profile directory to another to "clone" a profile.
**Why it's wrong:** Creates shared mutable state. Edits in one profile silently affect the clone. Session tokens may be invalidated by the source.
**Do this instead:** Each profile is fully independent. If a user wants a similar profile, they create a new one and authenticate separately.

### Anti-Pattern 3: Global State Mutation Without User Consent

**What people do:** Automatically modify `~/.bashrc` or `~/.zshrc` to source clmux env on every shell start.
**Why it's wrong:** Violates the "explicit > magic" principle. Users lose trust when tools modify their shell config. Debugging becomes harder.
**Do this instead:** Document `eval $(clmux env)` as opt-in. Provide `clmux run` as the zero-config alternative. Let users choose their integration level.

### Anti-Pattern 4: Monolithic Config File

**What people do:** Put everything in one file: profile definitions, active state, user preferences, cached data.
**Why it's wrong:** Every operation (including read-only `list`) writes the same file. Merge conflicts with concurrent access. Hard to reason about what changed.
**Do this instead:** Separate concerns: `config.toml` for profile registry (rarely changes), `state.toml` for active profile (changes on switch). This is what kubectl does with kubeconfig (contexts, clusters, users are sections, current-context is a single field).

## Suggested Build Order

Dependencies between components dictate implementation sequence:

```
Phase 1: Foundation (no commands work without these)
    paths.rs -> error.rs -> config.rs -> state.rs

Phase 2: Core Commands (depend on Foundation)
    profile.rs -> commands/add.rs -> commands/list.rs -> commands/show.rs

Phase 3: Switching (depends on Core Commands existing)
    commands/switch.rs -> commands/remove.rs

Phase 4: Shell Integration (depends on everything above)
    shell.rs -> commands/env.rs -> commands/run.rs

Phase 5: Polish
    Shell completions, --profile global flag, error message refinement
```

**Rationale:** You cannot add profiles without paths and config. You cannot switch without profiles to switch between. You cannot launch Claude Code without switching working. Each phase produces a testable, demoable increment.

## CLAUDE_CONFIG_DIR Behavior (Critical Context)

Based on investigation of Claude Code's actual behavior:

- `CLAUDE_CONFIG_DIR` redirects the **global** config directory (auth, history, projects, statsig, etc.)
- Project-local `.claude/` directories are **always** created in the workspace regardless of this env var
- Without the env var, Claude Code uses `~/.claude/` as its home
- With the env var, files like `.credentials.json` and `.claude.json` are stored under the specified directory

**Implication for clmux:** Each profile directory at `~/.clmux/profiles/<name>/` becomes a complete Claude Code home. When clmux sets `CLAUDE_CONFIG_DIR` to this path, Claude Code stores all global state there. This provides full isolation of auth, sessions, and settings per profile. Project-level `.claude/` directories are shared across profiles (they live in the workspace), which is the correct behavior -- project settings are per-project, not per-account.

## Sources

- [Rain's Rust CLI Recommendations - Configuration](https://rust-cli-recommendations.sunshowers.io/configuration.html)
- [AWS CLI Configuration and Credential Files](https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html)
- [Kubernetes kubeconfig Organization](https://kubernetes.io/docs/concepts/configuration/organize-cluster-access-kubeconfig/)
- [kubectl config use-context](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_use-context/)
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [Claude Code Settings Docs](https://code.claude.com/docs/en/settings)
- [CLAUDE_CONFIG_DIR behavior issue](https://github.com/anthropics/claude-code/issues/3833)
- [Rust CLI Structure - Kevin Knapp](https://kbknapp.dev/cli-structure-01/)
- [dirs crate for XDG/platform directories](https://lib.rs/config)

---
*Architecture research for: CLI profile/config management (Rust)*
*Researched: 2026-04-02*
