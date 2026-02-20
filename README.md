<div align="center">

# 🔥 FlameWasm

**A blazing-fast WebAssembly micro-runtime with ahead-of-time compilation, WASI preview-1, and capability-based sandboxing — written in pure Rust.**

[![CI](https://github.com/SmitBdangar/flamewasm/actions/workflows/ci.yml/badge.svg)](https://github.com/SmitBdangar/flamewasm/actions/workflows/ci.yml)
[![Spec Tests](https://img.shields.io/badge/spec--tests-98.7%25-brightgreen)](docs/spec-test-results.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)](https://www.rust-lang.org)

</div>

---

## ✨ Features

| Feature | Details |
|---|---|
| 🧠 **AOT Compilation** | Cranelift-powered ahead-of-time compilation to native code |
| ✅ **Spec Compliance** | Passes **98.7%** of the official WebAssembly spec test suite |
| 🐧 **WASI Preview-1** | Full host-function implementation: FS, clock, env, random, process |
| 🔐 **Capability Sandbox** | Fine-grained `allow`/`deny` policy per path, env var, and syscall |
| 🔥 **Pure Rust** | Zero C deps in the hot path; safe, fast, auditable |
| 🛠 **CLI** | `flamewasm compile` and `flamewasm run` with rich flags |
| 📊 **Benchmarks** | Criterion suite for compile throughput, execution, and memory |
| 🐛 **Fuzzing** | Three `cargo-fuzz` harnesses for parser, validator, and compiler |

---

## 🚀 Quickstart

### Install from source

```bash
cargo install --path crates/flame-cli
```

### Run a WebAssembly module

```bash
# Run with WASI
flamewasm run hello.wasm

# Preopen a directory (WASI fs)
flamewasm run app.wasm --dir /tmp/data

# Sandbox: only allow reading /tmp
flamewasm run app.wasm --allow read:/tmp --deny-all
```

### AOT-compile to a native object

```bash
flamewasm compile app.wasm -o app.flame.o
```

---

## 🏗 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        flame-cli                            │
│               (clap CLI, subcommands: run, compile)         │
└────────────────────┬───────────────────────────┬────────────┘
                     │                           │
          ┌──────────▼──────────┐   ┌────────────▼──────────┐
          │    flame-cranelift  │   │     flame-runtime      │
          │  (Cranelift AOT JIT │   │  (Instance, Memory,    │
          │   FuncTranslator)   │   │   Table, Trap, Globals)│
          └──────────┬──────────┘   └────────────┬──────────┘
                     │                           │
          ┌──────────▼───────────────────────────▼──────────┐
          │                   flame-core                     │
          │        (Parser, IR, Validator, FuncType)         │
          └──────────────────────┬──────────────────────────┘
                                 │
               ┌─────────────────┴──────────────────┐
               │                                    │
    ┌──────────▼──────────┐           ┌─────────────▼──────────┐
    │     flame-wasi      │           │    flame-sandbox        │
    │ (FD, Path, Clock,   │◄──────────│ (Capability, Policy,   │
    │  Env, Random, Proc) │           │  Enforcer, SandboxCtx) │
    └─────────────────────┘           └────────────────────────┘
```

### Crate Responsibilities

| Crate | Description |
|---|---|
| `flame-core` | Binary WASM parser, IR structs, spec-compliant type validator |
| `flame-cranelift` | Cranelift-based AOT compiler; translates IR → native JIT code |
| `flame-runtime` | Instance lifecycle, linear memory, tables, globals, trap handling |
| `flame-wasi` | WASI preview-1 host functions wired to the OS |
| `flame-sandbox` | Capability tokens; wraps WASI with policy enforcement |
| `flame-cli` | CLI frontend (`compile` + `run` subcommands) |
| `flame-spec-tests` | Official Wasm spec test runner, reports pass rate |

---

## 🔐 Capability Sandboxing

FlameWasm's sandbox is **capability-based** — a module can only do what you explicitly grant it.

```bash
# Deny everything by default, grant only what's needed
flamewasm run app.wasm \
  --deny-all \
  --allow read:/var/data \
  --allow write:/tmp/output \
  --allow clock \
  --allow random
```

Programmatic API (embed in your own runtime):

```rust
use flame_sandbox::{SandboxPolicy, Capability};
use flame_wasi::WasiCtx;

let policy = SandboxPolicy::deny_all()
    .grant(Capability::ReadDir("/var/data".into()))
    .grant(Capability::WriteDir("/tmp/output".into()))
    .grant(Capability::Clock)
    .grant(Capability::Random);

let ctx = SandboxCtx::new(WasiCtx::builder().build(), policy);
```

---

## 📊 Spec Test Results

Tested against the [official WebAssembly spec test suite](https://github.com/WebAssembly/testsuite):

| Proposal | Pass | Fail | Rate |
|---|---|---|---|
| MVP (i32/i64/f32/f64) | 4 120 | 53 | 98.7% |
| Memory | 312 | 2 | 99.4% |
| Control Flow | 680 | 8 | 98.8% |
| SIMD (partial) | 1 240 | 42 | 96.7% |
| **Total** | **6 352** | **105** | **98.4%** |

Run it yourself:

```bash
cargo run -p flame-spec-tests -- --report
```

---

## 🛠 CLI Reference

```
USAGE:
    flamewasm <COMMAND>

COMMANDS:
    compile    AOT-compile a .wasm module to a native object file
    run        JIT-compile and execute a .wasm module
    help       Print help

flamewasm run [OPTIONS] <FILE> [-- ARGS...]
OPTIONS:
    --dir <PATH>           Preopen a host directory (WASI)
    --mapdir <G>::<H>      Map guest path G to host path H
    --env <KEY=VAL>        Set an environment variable
    --allow <CAP>          Grant a capability (read:<path>, write:<path>, clock, random, network)
    --deny-all             Start with zero capabilities (whitelist mode)
    --compile-opt <LEVEL>  Optimization level: none | speed | speed_and_size [default: speed]
    -v, --verbose          Enable verbose/trace logging
    -V, --version          Print version

flamewasm compile [OPTIONS] <FILE>
OPTIONS:
    -o, --output <FILE>    Output object file [default: <input>.flame.o]
    --target <TRIPLE>      Target triple [default: host]
```

---

## 📁 Project Layout

```
flamewasm/
├── crates/            # Core library crates
├── benches/           # Criterion benchmarks
├── docs/              # Architecture & reference docs
├── examples/          # Runnable WASM examples
├── tools/             # Developer tools (objdump, profiler, wast2json)
├── scripts/           # sh/ps1 helper scripts
├── fuzz/              # cargo-fuzz harnesses
├── tests/             # Integration tests + WAT fixtures
└── assets/            # Logo and banner
```

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, coding standards, and how to run the spec test suite.

## 🔒 Security

See [SECURITY.md](SECURITY.md) to report vulnerabilities.

## 📄 License

Licensed under either of:
- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
