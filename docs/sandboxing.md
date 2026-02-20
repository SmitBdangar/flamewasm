# Capability-Based Sandboxing in FlameWasm

## Motivation

WebAssembly modules are untrusted code. Even though Wasm itself is memory-safe
and sandboxed at the instruction level, **WASI** gives modules access to the
host filesystem, environment, clock, random, and process control. Without
additional gating these become attack surfaces.

FlameWasm's sandbox layer wraps every WASI host function with a **capability
check** before delegating to the actual OS call.

## The `Capability` Token

```rust
pub enum Capability {
    ReadDir(PathBuf),   // recursive read access to a directory
    WriteDir(PathBuf),  // recursive write access to a directory
    ReadFile(PathBuf),  // exact file read
    WriteFile(PathBuf), // exact file write
    Clock,
    Random,
    Network,            // reserved
    Env(String),        // single env var key
    ProcessExit,
}
```

## Policy modes

| Mode | Description |
|---|---|
| `SandboxPolicy::allow_all()` | All operations permitted (default when no `--deny-all`) |
| `SandboxPolicy::deny_all()` | No operations permitted unless explicitly granted |

## CLI Example

```bash
# Only allow reading /data, the system clock, and random bytes
flamewasm run app.wasm \
  --deny-all \
  --allow read:/data \
  --allow clock   \
  --allow random

# Any other WASI call (fd_write to files, path_open outside /data, etc.)
# will return EACCES to the module.
```

## Programmatic API

```rust
use flame_sandbox::{SandboxCtx, SandboxPolicy, Capability};
use flame_wasi::WasiCtx;
use std::path::PathBuf;

let policy = SandboxPolicy::deny_all()
    .grant(Capability::ReadDir("/var/data".into()))
    .grant(Capability::WriteDir("/tmp/out".into()))
    .grant(Capability::Clock)
    .grant(Capability::Random)
    .grant(Capability::ProcessExit);

let sandbox = SandboxCtx::new(WasiCtx::builder().build(), policy);
```

## Future Work

- Per-fd capability tracking (not just directory-level)
- Network capability with allowlisted hosts/ports
- Resource quotas (CPU time, memory pages, fd count)
- seccomp-BPF integration on Linux for kernel-level enforcement
