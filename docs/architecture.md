# FlameWasm Architecture

## Overview

FlameWasm is structured as a **layered pipeline** of Cargo crates. Each layer depends only on the layers below it.

```
flame-cli / flame-spec-tests
       │
flame-sandbox ──► flame-wasi
       │
flame-runtime ──► flame-cranelift ──► flame-core
```

## Data Flow

```
.wasm bytes
    │
    ▼
flame-core::parse()        ← wasmparser decodes binary → Module IR
    │
flame-core::validate()     ← spec-compliant type checker
    │
flame-cranelift::compile() ← Cranelift JIT: FuncTranslator → native code
    │
flame-runtime::Instance    ← owns memory, tables, globals; calls exports
    │
flame-wasi / flame-sandbox ← host function gates with capability policy
    │
flame-cli                  ← CLI wiring + WASI I/O dispatch
```

## Crate Responsibilities

| Crate | Key Types | Key APIs |
|---|---|---|
| `flame-core` | `Module`, `Instruction`, `FuncType` | `parse()`, `validate()` |
| `flame-cranelift` | `CompiledModule` | `compile()` |
| `flame-runtime` | `Instance`, `LinearMemory`, `Trap` | `Instance::new()`, `Instance::call()` |
| `flame-wasi` | `WasiCtx`, `FdTable` | `WasiCtxBuilder` |
| `flame-sandbox` | `SandboxPolicy`, `SandboxCtx`, `Capability` | `SandboxPolicy::deny_all().grant()` |
| `flame-cli` | — | `flamewasm run`, `flamewasm compile` |
| `flame-spec-tests` | — | `run_wast_file()` |

## AOT Compilation Pipeline

1. `wasmparser` decodes binary → `Module` IR
2. `build_signature()` constructs Cranelift function signatures per type section
3. `JITModule::declare_function()` registers each function with Cranelift
4. `FuncTranslator::translate()` walks each `FuncBody::instructions` and emits Cranelift IR ops via `FunctionBuilder`
5. `JITModule::define_function()` + `finalize_definitions()` links and writes machine code
6. Export symbol addresses extracted via `get_finalized_function()`

## Sandbox Model

FlameWasm's sandbox is **capability-based** and **deny-by-default** (when `--deny-all` is used):

- Each WASI host function is guarded by `SandboxCtx::{check_read_path, check_write_path, check_clock, check_random, check_proc_exit}`
- Denied calls return `EACCES` (errno 2) rather than executing
- Capability grants are stored as a `HashSet<Capability>` — O(1) lookup

## Spec Compliance

The validator implements the WebAssembly core specification validation algorithm:
- Control-flow stack (`Frame` with `kind`, `result_types`, `stack_height`, `unreachable` flag)
- Operand stack type checking
- Index-space bounds (`types`, `funcs`, `tables`, `memories`, `globals`)
- Polymorphic unreachable handling ("bottom type")
