# Stack Research

**Domain:** Rust CLI profile/config manager tool
**Researched:** 2026-04-02
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | 1.85+ | Language | Single static binary, no runtime deps, strong type system catches config bugs at compile time. Already decided in PROJECT.md. |
| clap | 4.6.0 | CLI argument parsing + subcommands | Industry standard for Rust CLIs. Derive macro (`#[derive(Parser)]`) generates arg parsing from struct definitions -- ideal for subcommands like `clmux add`, `clmux switch`, `clmux list`. Actively maintained by the clap-rs team. |
| serde | 1.0.228 | Serialization framework | Required by toml crate. Derive macros (`#[derive(Serialize, Deserialize)]`) make config struct round-tripping trivial. Universal in the Rust ecosystem. |
| toml | 1.1.2 | TOML config file read/write | First-class serde integration. Rust ecosystem convention (Cargo.toml uses it). Human-readable config for `state.toml` and profile metadata. Already decided in PROJECT.md. |
| anyhow | 1.0.102 | Application-level error handling | Provides `anyhow::Result<T>` with automatic error context chaining via `.context()`. Perfect for CLI apps where errors become user-facing messages. No need for typed error enums in a tool this focused. |
| dirs | 6.0.0 | Platform-standard home directory resolution | Returns `~` / `$HOME` correctly on macOS and Linux. Needed to resolve `~/.clmux/` base path. Tiny, no transitive deps. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| owo-colors | 4.2.3 | Colored terminal output | Status display (`clmux list`, `clmux status`). Zero-allocation, no_std compatible, respects `NO_COLOR` env var. Use for highlighting active profile, warnings, errors. |
| thiserror | 2.0.18 | Typed error derive macro | Only if the project grows to need distinct error types (e.g., `ProfileNotFound` vs `ConfigCorrupted` for programmatic handling). Not needed in v1 MVP where anyhow suffices, but pin the version now for when it becomes useful. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| assert_cmd | 2.2.0 | CLI integration testing | Run the compiled binary in tests, assert on stdout/stderr/exit code. The standard way to test Rust CLI apps end-to-end. |
| assert_fs | 1.1.3 | Filesystem fixture testing | Create temp dirs that simulate `~/.clmux/` in tests. Essential for testing profile creation, switching, removal without touching the real filesystem. |
| predicates | 3.1.4 | Test assertions | Companion to assert_cmd/assert_fs. Provides `predicate::str::contains()`, regex matching, file existence checks. Required by the other two. |
| cargo-dist | latest | Binary distribution | Generates GitHub Actions CI for building + releasing static binaries for macOS (x86_64 + aarch64) and Linux (x86_64-musl). Handles the entire release flow from `cargo dist init`. |

## Cargo.toml Skeleton

```toml
[package]
name = "clmux"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.6", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
toml = "1.1"
anyhow = "1.0"
dirs = "6.0"
owo-colors = "4.2"

[dev-dependencies]
assert_cmd = "2.2"
assert_fs = "1.1"
predicates = "3.1"
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| clap (derive) | argh (Google) | Never for this project. argh is lighter but lacks clap's ecosystem (completions, man pages, help formatting). clap's derive API is equally ergonomic. |
| anyhow | thiserror | When building a library where callers need to match on error variants. clmux is an end-user binary, not a library -- anyhow is the right choice. |
| toml | serde_json / serde_yaml | Never for user-facing config. TOML is the Rust convention, human-editable, no footguns. JSON lacks comments. YAML has the Norway problem and implicit type coercion. |
| dirs | directories | If you need XDG cache/data/state separation. clmux only needs home dir to build `~/.clmux/`, so the simpler `dirs` crate is sufficient. |
| owo-colors | colored | If you want a more familiar API. `colored` allocates Strings; `owo-colors` is zero-cost. For a CLI tool that may colorize every line of output, prefer owo-colors. |
| cargo-dist | Manual GH Actions | If you need non-standard build steps. cargo-dist handles 95% of use cases and is less maintenance. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| structopt | Deprecated -- merged into clap 4.x derive API. Still compiles but receives no updates. | clap with `features = ["derive"]` |
| config (crate) | Overly complex for this use case. Supports 12-factor, env overlays, multiple formats. clmux just reads/writes one TOML file. | Direct serde + toml deserialization |
| confy | Opinionated about config location (XDG). clmux has its own `~/.clmux/` convention. confy would fight the design. | Manual `dirs::home_dir()` + `fs::read_to_string` + `toml::from_str` |
| dialoguer / inquire | Interactive prompts. clmux is non-interactive by design (explicit commands, no wizards). Out of scope per PROJECT.md. | Direct clap subcommands |
| tokio / async-std | Async runtimes. clmux does synchronous filesystem operations only. Adding async would add compile time and complexity for zero benefit. | `std::fs` |
| snafu | More complex error handling with context selectors. Overkill for a focused CLI tool. | anyhow for app errors |

## Stack Patterns by Variant

**If adding shell completions later:**
- Enable clap feature `features = ["derive", "env"]` and use `clap_complete` crate
- clap_complete generates bash/zsh/fish completions from the Parser definition

**If adding man page generation later:**
- Use `clap_mangen` crate to generate man pages from the Parser definition
- Both clap_complete and clap_mangen are maintained by the clap team

**If profile config grows complex (nested structures, arrays):**
- TOML handles this fine. No need to switch formats.
- Use `#[serde(default)]` on optional fields for forward compatibility

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| clap 4.6 | Rust 1.80+ | MSRV is 1.80. Use stable channel. |
| serde 1.0.228 | toml 1.1.2 | toml depends on serde; versions always compatible within serde 1.x |
| owo-colors 4.2 | Rust 1.81+ | MSRV is 1.81. No conflicts with other deps. |
| anyhow 1.0.102 | All above | No compatibility concerns. Works with any serde/clap version. |
| dirs 6.0 | Rust 1.63+ | Very low MSRV, no issues. |
| Effective MSRV | Rust 1.81+ | Bottleneck is owo-colors. All other crates work at 1.80 or lower. |

## Static Binary Distribution

For the single-binary requirement:

**macOS:** Default `cargo build --release` produces a dynamically linked binary, but macOS system libs (libSystem) are always available. No special action needed -- the binary is portable across macOS versions.

**Linux:** Use `x86_64-unknown-linux-musl` target for fully static linking:
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

**cargo-dist** automates this: `cargo dist init` scaffolds GitHub Actions that build for all targets and creates GitHub Releases with downloadable binaries.

## Sources

- [docs.rs/crate/clap/latest](https://docs.rs/crate/clap/latest) -- clap 4.6.0 version verified (HIGH confidence)
- [docs.rs/crate/toml/latest](https://docs.rs/crate/toml/latest) -- toml 1.1.2 version verified (HIGH confidence)
- [docs.rs/crate/serde/latest](https://docs.rs/crate/serde/latest) -- serde 1.0.228 version verified (HIGH confidence)
- [docs.rs/crate/anyhow/latest](https://docs.rs/crate/anyhow/latest) -- anyhow 1.0.102 version verified (HIGH confidence)
- [docs.rs/crate/thiserror/latest](https://docs.rs/crate/thiserror/latest) -- thiserror 2.0.18 version verified (HIGH confidence)
- [docs.rs/crate/dirs/latest](https://docs.rs/crate/dirs/latest) -- dirs 6.0.0 version verified (HIGH confidence)
- [docs.rs/crate/owo-colors/latest](https://docs.rs/crate/owo-colors/latest) -- owo-colors 4.2.3 version verified (MEDIUM confidence, from search results)
- [docs.rs/crate/assert_cmd/latest](https://docs.rs/crate/assert_cmd/latest) -- assert_cmd 2.2.0 version verified (HIGH confidence)
- [docs.rs/crate/assert_fs/latest](https://docs.rs/crate/assert_fs/latest) -- assert_fs 1.1.3 version verified (HIGH confidence)
- [docs.rs/crate/predicates/latest](https://docs.rs/crate/predicates/latest) -- predicates 3.1.4 version verified (HIGH confidence)
- [rust-cli-recommendations.sunshowers.io](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) -- owo-colors recommendation (MEDIUM confidence)
- [axodotdev/cargo-dist](https://github.com/axodotdev/cargo-dist/releases) -- cargo-dist release automation (MEDIUM confidence)
- [rust-cli.github.io/book/tutorial/testing.html](https://rust-cli.github.io/book/tutorial/testing.html) -- CLI testing patterns (HIGH confidence)

---
*Stack research for: Rust CLI profile/config manager (clmux)*
*Researched: 2026-04-02*
