# Contributing to FlameWasm

Thank you for your interest in contributing! FlameWasm welcomes bug reports, feature requests, documentation improvements, and pull requests.

---

## Dev Setup

**Prerequisites:** Rust 1.78+ (stable), `git`

```bash
git clone https://github.com/SmitBdangar/flamewasm
cd flamewasm

# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Run spec tests
cargo run -p flame-spec-tests -- --report

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

### Optional Tools

```bash
# For fuzzing
cargo install cargo-fuzz   # nightly required: rustup default nightly

# For code coverage
cargo install cargo-llvm-cov
```

---

## Coding Standards

- **Safety**: avoid `unsafe` blocks unless absolutely necessary; always document safety invariants.
- **Errors**: use `anyhow` for application-level errors; `thiserror` for library error types.
- **Clippy**: the CI runs `clippy -D warnings`; resolve all warnings before submitting.
- **Fmt**: run `cargo fmt --all` before committing.
- **Tests**: every public API path should have a unit test. New features need integration tests.
- **Docs**: all `pub` items must have `///` doc comments.

---

## Running the Spec Test Suite

```bash
# Download the official testsuite (requires internet)
./scripts/fetch-spec-tests.sh      # Linux/macOS
.\scripts\fetch-spec-tests.ps1     # Windows

# Run the suite
cargo run -p flame-spec-tests -- --report
```

The target pass rate is **≥ 98.7%**. If your change causes a regression, explain why in the PR.

---

## Pull Request Process

1. Fork the repository and create a feature branch.
2. Make your changes, add tests.
3. Run `cargo test --workspace` and `cargo clippy -- -D warnings`.
4. Open a PR using the provided template.
5. A maintainer will review within 48 hours.

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(sandbox): add network capability grant
fix(parser): handle empty data sections correctly
docs(readme): update spec test results table
```
