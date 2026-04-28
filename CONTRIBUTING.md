# Contributing to clam

Thanks for your interest in contributing! Here's how to get started.

## Opening Issues

- **Bug reports**: Please include your `clam --version` output, OS, and steps to reproduce.
- **Feature requests**: Open an issue to discuss before sending a PR. This helps avoid wasted effort.

## Development Setup

```sh
git clone https://github.com/jose2kk/clam.git
cd clam
cargo build
cargo nextest run
```

### Prerequisites

- Rust 1.81+ (MSRV)
- [cargo-nextest](https://nexte.st/) for running tests

## Before Submitting a PR

1. **Run formatting and lints**:
   ```sh
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   ```

2. **Run tests**:
   ```sh
   cargo nextest run --all-features
   ```

3. **Add a changelog entry** to the `[Unreleased]` section in `CHANGELOG.md`.

4. **Keep PRs focused** — one logical change per PR.

## Releasing (maintainers)

Releases are automated via [cargo-dist](https://opensource.axo.dev/cargo-dist/). When you push a version tag, CI builds binaries for all platforms, creates a GitHub Release, and publishes the Homebrew formula.

### Steps

1. **Update the version** in `Cargo.toml`:
   ```sh
   # Edit Cargo.toml: version = "0.2.0"
   ```

2. **Move the Unreleased changelog entries** to a new version section in `CHANGELOG.md`:
   ```markdown
   ## [0.2.0] - 2026-04-27

   ### Added
   - ...
   ```

3. **Commit the version bump**:
   ```sh
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "release: v0.2.0"
   ```

4. **Tag and push**:
   ```sh
   git tag v0.2.0
   git push origin main --tags
   ```

CI will automatically:

- Build binaries for macOS (x86 + ARM), Linux (glibc + musl), and Windows
- Create a GitHub Release with all artifacts and checksums
- Generate the shell and PowerShell installer scripts
- Publish the Homebrew formula to [jose2kk/homebrew-tap](https://github.com/jose2kk/homebrew-tap)
- Attach supply chain attestations to the release
- Publish to [crates.io](https://crates.io/crates/clam-cli)

### Prerequisites (one-time setup)

Before the first release, you need:

1. Create the [`jose2kk/homebrew-tap`](https://github.com/jose2kk/homebrew-tap) repo on GitHub (public, with a README)
2. Create a GitHub fine-grained token with **Contents: Read and write** scoped to `homebrew-tap` only
3. Add it as the `HOMEBREW_TAP_TOKEN` secret in the clam repo settings (Settings > Secrets and variables > Actions)
4. Create a [crates.io](https://crates.io) API token scoped to **publish-update** for `clam-cli`
5. Add it as the `CARGO_REGISTRY_TOKEN` secret in the clam repo settings

## Code Style

- Follow existing patterns in the codebase.
- `cargo fmt` and `cargo clippy` must pass with zero warnings.
- Unsafe code is forbidden (`#[forbid(unsafe_code)]`).

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.
