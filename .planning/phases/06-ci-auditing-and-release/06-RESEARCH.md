# Phase 6: CI, Auditing, and Release - Research

**Researched:** 2026-04-07
**Domain:** GitHub Actions CI/CD, cargo-deny dependency auditing, cargo-dist release automation
**Confidence:** HIGH

## Summary

Phase 6 adds infrastructure-only files to the repository: GitHub Actions workflows (`ci.yml`, `release.yml`), a `deny.toml` for dependency auditing, a `dependabot.yml` for automated updates, and cargo-dist configuration for static binary releases. No source code changes are required -- the codebase already passes all linting and tests locally.

The key technical decisions are locked: single `ci.yml` with parallel jobs (fmt, clippy, deny, test, msrv, coverage), `EmbarkStudios/cargo-deny-action@v2` for auditing, `cargo-dist` for release automation targeting three platforms (x86_64-linux-musl, x86_64-macos, aarch64-macos), and `cargo-llvm-cov` for coverage reports uploaded as artifacts.

**Primary recommendation:** Write and verify all CI commands locally before committing workflow files. The CI workflow should be a transcription of known-working local commands, not a debugging environment.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Single `ci.yml` workflow with parallel jobs -- fmt, clippy, deny, test, msrv, coverage as separate jobs in one file
- **D-02:** Separate MSRV job -- installs rust-version from Cargo.toml (1.81) and runs `cargo check`. Does not run the full test matrix on MSRV.
- **D-03:** fmt and clippy run as independent parallel jobs. No fail-fast -- all jobs run to completion so all issues are visible at once.
- **D-04:** Test job uses OS matrix: ubuntu-latest + macos-latest (CI-04)
- **D-05:** Use `Swatinem/rust-cache` for dependency caching (CI-05)
- **D-06:** Claude's discretion on cargo-nextest vs cargo test
- **D-07:** deny.toml license allowlist -- Claude's discretion on the specific set
- **D-08:** Security advisories: deny on known vulnerabilities (fail CI), warn on unmaintained crates (don't fail)
- **D-09:** Duplicate crate detection: warn but don't fail CI
- **D-10:** Use cargo-dist for release workflow generation -- run `cargo dist init` to bootstrap
- **D-11:** Target platforms: x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-musl (per REL-03)
- **D-12:** Release artifacts: static binaries + SHA256 checksums attached to GitHub Release. No shell installer scripts.
- **D-13:** Release trigger: version tag push (v*) per REL-02
- **D-14:** Use cargo-llvm-cov to generate HTML coverage report, uploaded as GitHub Actions artifact (CI-08)
- **D-15:** No minimum coverage threshold -- coverage is for visibility only, not a gate
- **D-16:** Configure `.github/dependabot.yml` for Cargo dependency update PRs (AUDIT-03)

### Claude's Discretion
- Exact cargo-nextest vs cargo test decision (D-06)
- License allowlist details (D-07)
- Dependabot update frequency (weekly vs monthly)
- cargo-dist configuration details (installer types, archive formats)
- CI job naming and ordering
- Whether to add `make ci` target to Makefile or keep CI and Makefile separate

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CI-01 | GitHub Actions CI runs `cargo fmt --check` on every push and PR | ci.yml fmt job pattern documented |
| CI-02 | GitHub Actions CI runs `cargo clippy -- -D warnings` on every push and PR | ci.yml clippy job pattern documented |
| CI-03 | GitHub Actions CI runs `cargo test` on every push and PR | ci.yml test job pattern documented; nextest recommended |
| CI-04 | CI tests on macOS and Linux matrix (ubuntu-latest + macos-latest) | Matrix strategy on test job only |
| CI-05 | CI uses `Swatinem/rust-cache` for dependency caching | Swatinem/rust-cache@v2 documented; apply to clippy + test + coverage |
| CI-06 | CI verifies minimum supported Rust version (MSRV) compiles | dtolnay/rust-toolchain@1.81 with cargo check |
| CI-07 | CI runs tests via `cargo-nextest` for process-isolated test execution | taiki-e/install-action@nextest documented |
| CI-08 | CI generates code coverage via `cargo-llvm-cov` and reports it | cargo-llvm-cov --html + upload-artifact pattern |
| AUDIT-01 | `deny.toml` configured for security advisories, license compliance, and duplicate detection | Full deny.toml template with verified license list |
| AUDIT-02 | `cargo deny check` runs as a CI job | EmbarkStudios/cargo-deny-action@v2 documented |
| AUDIT-03 | Dependabot configuration for automated Cargo dependency update PRs | dependabot.yml template documented |
| REL-01 | `cargo-dist` initialized with `dist init` for static binary distribution | dist init procedure documented; generates dist-workspace.toml + release.yml |
| REL-02 | Release workflow triggers on version tag push (v*) | cargo-dist generates correct trigger |
| REL-03 | Builds target macOS (x86_64 + aarch64) and Linux (x86_64-musl) | Targets specified during dist init |

</phase_requirements>

## Standard Stack

### Core Tools (CI)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `dtolnay/rust-toolchain` | @stable / @1.81 | Rust toolchain installation in CI | Actively maintained; reads rust-toolchain.toml; replaces deprecated actions-rs/toolchain |
| `Swatinem/rust-cache` | @v2 (v2.8.1) | Cargo dependency caching | Community standard; auto-hashes rust-toolchain.toml and Cargo.lock |
| `cargo-nextest` | 0.9.130 | Test runner with per-test process isolation | Better output, JUnit XML, parallel execution; taiki-e/install-action@nextest for CI |
| `cargo-llvm-cov` | 0.8.5 | LLVM source-based code coverage | Works on macOS + Linux (unlike tarpaulin); supports nextest; HTML/LCOV output |
| `taiki-e/install-action` | @v2 | Install Rust dev tools from pre-built binaries | Fast binary downloads; supports nextest, cargo-llvm-cov, cargo-deny |

### Core Tools (Auditing)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `cargo-deny` | 0.19.0 | Dependency auditing (licenses, advisories, bans, sources) | Superset of cargo-audit; single deny.toml config |
| `EmbarkStudios/cargo-deny-action` | @v2 | CI integration for cargo-deny | No Rust toolchain needed; reads Cargo.lock directly; fast (<30s) |

### Core Tools (Release)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `cargo-dist` | 0.31.0 | Release binary distribution automation | Generates release.yml; handles cross-compilation; SHA256 checksums |

### Discretion Decisions

**cargo-nextest vs cargo test (D-06):** Use cargo-nextest. Rationale:
- CI-07 explicitly requires nextest for process-isolated execution
- `taiki-e/install-action@nextest` installs pre-built binaries (fast, no compilation)
- `cargo llvm-cov nextest` integrates coverage collection with nextest directly
- Better CI output (per-test pass/fail with timing)
- For 43+ tests, the per-process isolation prevents test pollution

**Dependabot frequency:** Weekly for Cargo, weekly for GitHub Actions. Rationale:
- Monthly is too slow for security patches
- Weekly is manageable for a small dependency tree (9 direct deps)

**Installation (not required locally):**
```bash
# Only needed if running deny/dist locally -- CI handles these
cargo install --locked cargo-deny
cargo install --locked cargo-dist
cargo install --locked cargo-nextest
cargo install --locked cargo-llvm-cov
```

## Architecture Patterns

### New File Layout

```
clmux/
├── .github/
│   ├── dependabot.yml              # Automated dependency update PRs
│   └── workflows/
│       ├── ci.yml                  # fmt, clippy, deny, test, msrv, coverage
│       └── release.yml             # Generated by cargo-dist
├── deny.toml                       # cargo-deny configuration
└── dist-workspace.toml             # cargo-dist configuration (generated)
```

### Pattern 1: ci.yml Job Structure

**What:** Six parallel jobs in one workflow file, no fail-fast
**When to use:** Every push and PR

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets -- -D warnings

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check
          arguments: --all-features

  test:
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --all-features

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.81
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-targets

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov
      - uses: taiki-e/install-action@nextest
      - run: cargo llvm-cov nextest --html --output-dir coverage-report
      - uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage-report/
```

**Key design points:**
- `fmt` needs no cache (no compilation)
- `deny` needs no Rust toolchain (reads Cargo.lock)
- `test` is the only job with OS matrix (platform-specific behavior)
- `clippy` runs on ubuntu-latest only (deterministic across platforms for this codebase)
- `msrv` uses `dtolnay/rust-toolchain@1.81` (exact version from Cargo.toml rust-version)
- `coverage` installs `llvm-tools-preview` component (required by cargo-llvm-cov)
- `coverage` uses `cargo llvm-cov nextest` to combine coverage with nextest
- `CARGO_TERM_COLOR: always` enables colored output in CI logs

### Pattern 2: deny.toml Configuration

**What:** Dependency auditing policy
**Verified license list:** Extracted from actual `cargo metadata` on current dependency tree.

```toml
[advisories]
version = 2
ignore = []

[licenses]
version = 2
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "ISC",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Unicode-3.0",
    "MPL-2.0",
    "Zlib",
    "Unlicense",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

**License list rationale (verified against actual dep tree):**
- `MIT`, `Apache-2.0`, `ISC`, `BSD-*`: Standard permissive (clap, serde, anyhow, etc.)
- `Unicode-3.0`: unicode-ident (syn dependency)
- `MPL-2.0`: option-ext (dirs dependency)
- `Zlib`: foldhash (hashbrown dependency)
- `Unlicense`: aho-corasick, memchr, walkdir, globset, ignore, same-file, winapi-util (BurntSushi crates, all dual-licensed Unlicense/MIT)
- `LGPL-2.1-or-later`: r-efi (transitive, dual-licensed MIT OR Apache-2.0 OR LGPL-2.1-or-later -- the allowlist need not include LGPL because cargo-deny evaluates OR expressions and will match MIT or Apache-2.0 first)

**Advisory policy (D-08):**
- Known vulnerabilities: `deny` (default -- fail CI)
- Unmaintained crates: use default `all` but add `ignore` entries for false positives if needed
- Yanked crates: `warn` (default)

### Pattern 3: dependabot.yml

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    commit-message:
      prefix: "deps"
    labels:
      - dependencies

  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
    commit-message:
      prefix: "ci"
    labels:
      - dependencies
```

### Pattern 4: cargo-dist Setup

**What:** Run `dist init` interactively to generate config + release workflow
**Generated files:** `dist-workspace.toml` and `.github/workflows/release.yml`

```bash
# Install cargo-dist first
cargo install --locked cargo-dist

# Run init (interactive -- select targets and CI)
dist init
# Select: GitHub CI = yes
# Select targets: x86_64-unknown-linux-musl, x86_64-apple-darwin, aarch64-apple-darwin
# Installers: none (D-12 says no shell installer scripts)
```

**Expected dist-workspace.toml:**
```toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.31.0"
ci = ["github"]
targets = [
    "x86_64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
]
```

**Expected Cargo.toml addition (auto-generated by dist init):**
```toml
[profile.dist]
inherits = "release"
lto = "thin"
```

**Release flow:**
1. Update `version` in Cargo.toml
2. Commit: `git commit -am "release: v0.1.1"`
3. Tag: `git tag v0.1.1`
4. Push: `git push && git push --tags`
5. cargo-dist's release.yml triggers, builds 3 targets, uploads to GitHub Release

### Anti-Patterns to Avoid

- **Hand-editing release.yml:** cargo-dist generates and manages this file. Run `dist generate` to regenerate after config changes. Manual edits will be overwritten.
- **Using `actions-rs/toolchain`:** Deprecated since October 2023. Use `dtolnay/rust-toolchain` instead.
- **Running fmt/clippy on OS matrix:** Waste of CI minutes. These are platform-agnostic for this codebase. Run on ubuntu-latest only.
- **Putting `-D warnings` in Cargo.toml lints:** Use `warn` in Cargo.toml, `-- -D warnings` in CI only. This keeps local dev non-blocking while CI enforces clean.
- **Debugging CI by pushing commits:** Verify all commands pass locally first. CI should be a transcription, not an experiment.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Release binary builds | Custom cross-compilation scripts | cargo-dist | Handles macOS universal builds, musl linking, GitHub Release creation, checksums |
| Dependency auditing | Manual license/advisory checking | cargo-deny | Maintains RustSec advisory DB, SPDX license parsing, transitive dep analysis |
| CI caching | Manual cache key generation | Swatinem/rust-cache | Auto-hashes toolchain + Cargo.lock + config; handles cache eviction |
| Tool installation in CI | `cargo install` (slow compilation) | taiki-e/install-action | Downloads pre-built binaries; seconds instead of minutes |
| Release workflow YAML | Hand-written release pipeline | `dist init` + `dist generate` | Multi-stage plan/build/host/publish/announce pipeline |

**Key insight:** The CI/CD toolchain for Rust is mature. Every piece of this phase has a battle-tested solution. The value is in correct configuration, not custom code.

## Common Pitfalls

### Pitfall 1: Tag-Version Mismatch Breaks Release

**What goes wrong:** Tag `v0.1.1` pushed but Cargo.toml still says `version = "0.1.0"`. Release workflow triggers but cargo-dist fails to match the tag to a package version.
**Why it happens:** Manual tagging without updating Cargo.toml version.
**How to avoid:** Always update Cargo.toml version BEFORE tagging. The tag format must be `v{version}` exactly.
**Warning signs:** Release workflow runs but produces zero artifacts; "no releases were found" in CI logs.

### Pitfall 2: Missing llvm-tools-preview Component for Coverage

**What goes wrong:** `cargo llvm-cov` fails with "Failed to find `llvm-profdata`" in CI.
**Why it happens:** The `llvm-tools-preview` rustup component is not installed by default.
**How to avoid:** Add `components: llvm-tools-preview` to the `dtolnay/rust-toolchain` step in the coverage job.
**Warning signs:** Coverage job fails immediately with a missing tool error.

### Pitfall 3: Cache Order Matters -- Toolchain Before Cache

**What goes wrong:** Cache key doesn't include toolchain version, causing stale cache hits after Rust updates.
**Why it happens:** `Swatinem/rust-cache` must run AFTER `dtolnay/rust-toolchain` to hash the resolved toolchain version.
**How to avoid:** Always order steps: checkout -> toolchain -> cache -> build/test.
**Warning signs:** Intermittent CI failures after Rust stable updates; full rebuilds despite cache hits.

### Pitfall 4: deny.toml License List Missing Actual Dependencies

**What goes wrong:** `cargo deny check licenses` fails immediately on a fresh setup because the allowlist doesn't include all licenses in the dependency tree.
**Why it happens:** Using a generic allowlist without checking actual dependencies. The prior ARCHITECTURE.md research had an incomplete list.
**How to avoid:** Run `cargo metadata` to extract actual licenses. This research has done that -- the verified list includes MPL-2.0 (option-ext), Zlib (foldhash), and Unlicense (BurntSushi crates).
**Warning signs:** `cargo deny check` fails on first run with "license not in allow list" errors.

### Pitfall 5: cargo-dist Generates Non-Workspace-Aware Config

**What goes wrong:** For a single-crate project (no `[workspace]` in Cargo.toml), `dist init` generates `dist-workspace.toml` with `members = ["cargo:."]`. If Cargo.toml later adds a `[workspace]` section, the dist config may need regeneration.
**How to avoid:** Run `dist plan` locally after init to verify the build plan looks correct. The output should list exactly 3 targets and 1 binary.

### Pitfall 6: MSRV Job Uses Wrong Rust Version

**What goes wrong:** MSRV job hardcodes a version that doesn't match `rust-version` in Cargo.toml. Over time, MSRV is updated in Cargo.toml but the CI job is forgotten.
**Why it happens:** The MSRV version appears in two places: Cargo.toml and ci.yml.
**How to avoid:** Add a comment in ci.yml referencing Cargo.toml's rust-version. When updating MSRV, grep for the version number across all files.

## Code Examples

### Complete ci.yml (verified pattern)

See Pattern 1 in Architecture Patterns above. This is the canonical reference -- all commands mirror the existing Makefile targets with CI-appropriate flags.

### Coverage Job with Nextest Integration

```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview
    - uses: Swatinem/rust-cache@v2
    - uses: taiki-e/install-action@cargo-llvm-cov
    - uses: taiki-e/install-action@nextest
    - run: cargo llvm-cov nextest --html --output-dir coverage-report
    - uses: actions/upload-artifact@v4
      with:
        name: coverage-report
        path: coverage-report/
        retention-days: 14
```

### Local Verification Commands (run before committing ci.yml)

```bash
# These must ALL pass before writing ci.yml
cargo fmt --all -- --check           # CI-01
cargo clippy --all-targets -- -D warnings  # CI-02
cargo test                           # CI-03 (or cargo nextest run)
cargo check --all-targets            # CI-06 (MSRV check equivalent)

# Requires local install of cargo-deny
cargo deny check                     # AUDIT-01, AUDIT-02

# Requires local install of cargo-dist
dist plan                            # REL-01 (dry run)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `actions-rs/toolchain@v1` | `dtolnay/rust-toolchain@stable` | Oct 2023 (deprecated) | Must not use deprecated action |
| `cargo-audit` standalone | `cargo-deny` (superset) | 2022+ | Single tool covers advisories + licenses + bans + sources |
| `Cargo.toml [workspace.metadata.dist]` | `dist-workspace.toml` | cargo-dist v0.23.0 (Oct 2024) | New default for new projects; old format still supported |
| `tarpaulin` for coverage | `cargo-llvm-cov` | 2023+ | tarpaulin is Linux-only; llvm-cov works on macOS + Linux |
| `cargo test` in CI | `cargo nextest run` | 2023+ | Per-test process isolation, better output, JUnit XML |
| `actions/upload-artifact@v3` | `actions/upload-artifact@v4` | Nov 2024 | v3 deprecated; v4 is current |

## Open Questions

1. **cargo-dist interactive init in CI/automation context**
   - What we know: `dist init` is interactive (prompts for targets, CI, installers)
   - What's unclear: Whether `dist init --yes` accepts all defaults for the target list
   - Recommendation: Run `dist init` locally in the developer's terminal, commit generated files. Do NOT run dist init in CI.

2. **MSRV 1.81 compatibility with cargo-nextest**
   - What we know: MSRV job runs `cargo check` only (not tests), so nextest is irrelevant for MSRV
   - What's unclear: Whether cargo-nextest itself has a minimum Rust version requirement that conflicts
   - Recommendation: Not a concern -- MSRV job does not install or use nextest

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | All CI jobs | Yes | 1.94.1 | -- |
| rustc | All CI jobs | Yes | 1.94.1 | -- |
| cargo-dist | REL-01 (local init) | No | -- | Install: `cargo install --locked cargo-dist` |
| cargo-deny | AUDIT-01 (local verify) | No | -- | Install: `cargo install --locked cargo-deny` |
| cargo-nextest | CI-07 (local verify) | No | -- | Install: `cargo install --locked cargo-nextest` |
| cargo-llvm-cov | CI-08 (local verify) | No | -- | Install: `cargo install --locked cargo-llvm-cov` |
| GitHub Actions | CI-01 through CI-08 | Yes (remote) | -- | -- |

**Missing dependencies with no fallback:**
- `cargo-dist` must be installed locally to run `dist init` (one-time setup)

**Missing dependencies with fallback:**
- `cargo-deny`, `cargo-nextest`, `cargo-llvm-cov` are optional locally -- CI handles them via actions. Local install is nice-to-have for pre-push verification.

## Project Constraints (from CLAUDE.md)

- **Language:** Rust -- all tooling must be Rust ecosystem (cargo-based)
- **CLI framework:** clap 4.6 with derive -- relevant for test compilation in CI
- **OS support:** macOS + Linux; Windows deferred -- CI matrix must NOT include windows
- **Architecture:** Minimal abstraction -- CI config should be straightforward YAML, no complex reusable workflows
- **Distribution:** Single statically-linked binary -- cargo-dist with musl target is the correct approach
- **MSRV:** 1.81 (owo-colors bottleneck) -- MSRV job must use exactly this version
- **Atomic writes:** tempfile crate used for file operations -- relevant for test isolation in nextest
- **Output formatting:** owo-colors with TTY detection -- CI should set `CARGO_TERM_COLOR: always`
- **Error handling:** anyhow::Result -- no impact on CI config
- **Existing tests:** 43 integration tests in `tests/cli_tests.rs` using `assert_cmd` -- these need the binary built before running
- **Existing Makefile:** Already wraps `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`, `cargo test` -- CI should mirror these exact commands

## Sources

### Primary (HIGH confidence)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) - v2.8.1, cache key hashing strategy
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) - Toolchain installation action
- [EmbarkStudios/cargo-deny](https://embarkstudios.github.io/cargo-deny/) - deny.toml v2 config format for advisories, licenses, bans, sources
- [EmbarkStudios/cargo-deny-action](https://github.com/EmbarkStudios/cargo-deny-action) - v2 CI integration
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - v0.8.5, HTML report generation, nextest integration
- [cargo-nextest](https://nexte.st/) - v0.9.130, CI installation via taiki-e/install-action
- [taiki-e/install-action](https://github.com/taiki-e/install-action) - Pre-built binary installation for CI

### Secondary (MEDIUM confidence)
- [cargo-dist](https://axodotdev.github.io/cargo-dist/) - v0.31.0, dist-workspace.toml format (new default since v0.23.0)
- [cargo-dist v0.23.0 release](https://github.com/axodotdev/cargo-dist/releases/tag/v0.23.0) - Config format migration details
- [GitHub Dependabot docs](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuring-dependabot-version-updates) - dependabot.yml configuration

### Tertiary (LOW confidence)
- cargo-dist interactive init behavior for single-crate projects -- needs hands-on verification during implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all tools verified against crates.io and official docs
- Architecture: HIGH - CI patterns verified against prior ARCHITECTURE.md research + official action docs
- Pitfalls: HIGH - license list verified against actual `cargo metadata` output; known pitfalls from PITFALLS.md confirmed
- cargo-dist config format: MEDIUM - dist-workspace.toml is documented as new default, but interactive init behavior for single-crate varies

**Research date:** 2026-04-07
**Valid until:** 2026-05-07 (30 days -- stable tools, no fast-moving changes expected)
