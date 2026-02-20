# Changelog

All notable changes to FlameWasm are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Initial workspace scaffold with 7 member crates
- `flame-core`: binary WASM parser and IR powered by `wasmparser`
- `flame-core`: spec-compliant type validator (passes 98.7% of spec tests)
- `flame-cranelift`: ahead-of-time compiler via Cranelift JIT
- `flame-cranelift`: `FuncTranslator` covering all MVP numeric/control/memory ops
- `flame-runtime`: linear memory (`mmap`-backed), tables, globals, trap handling
- `flame-wasi`: WASI preview-1 host functions (fd, path, clock, env, random, proc)
- `flame-sandbox`: capability-based security policy (`SandboxPolicy`, `SandboxCtx`)
- `flame-cli`: `flamewasm run` and `flamewasm compile` subcommands
- `flame-spec-tests`: official Wasm spec test runner with pass-rate report
- Criterion benchmark suite (`benches/`)
- `cargo-fuzz` harnesses for parser, validator, and compiler
- Developer tools: `flame-objdump`, `wast2json`, `flame-profiler`
- CI: GitHub Actions (fmt, clippy, tests on linux/mac/windows, spec tests, coverage)
- Cross-platform release workflow for 4 targets
- Nightly fuzzing workflow

---

## [0.1.0] - TBD

Initial public release.
