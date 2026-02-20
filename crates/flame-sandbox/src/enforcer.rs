//! The `SandboxCtx` wraps `WasiCtx` with policy enforcement.

use flame_wasi::WasiCtx;

use crate::policy::SandboxPolicy;

/// A secured WASI context with capability-based access control.
pub struct SandboxCtx {
    pub wasi: WasiCtx,
    pub policy: SandboxPolicy,
}

impl SandboxCtx {
    /// Create a new sandbox context.
    pub fn new(wasi: WasiCtx, policy: SandboxPolicy) -> Self {
        Self { wasi, policy }
    }

    /// Gate a `random_get` call through the policy.
    pub fn random_get(&self, buf: &mut [u8]) -> u32 {
        if !self.policy.check_random() {
            tracing::warn!("sandbox: random_get denied");
            return flame_wasi::fd::errno::ACCES;
        }
        flame_wasi::random::random_get(buf)
    }

    /// Gate `clock_time_get` through the policy.
    pub fn clock_time_get(&self, clock_id: u32) -> (u64, u32) {
        if !self.policy.check_clock() {
            tracing::warn!("sandbox: clock_time_get denied");
            return (0, flame_wasi::fd::errno::ACCES);
        }
        flame_wasi::clock::clock_time_get(clock_id)
    }

    /// Gate `proc_exit` through the policy.
    pub fn proc_exit(&self, code: i32) -> ! {
        if !self.policy.check_proc_exit() {
            tracing::warn!("sandbox: proc_exit({code}) denied");
            std::process::exit(1);
        }
        std::process::exit(code);
    }

    /// Gate a path-read operation through the policy.
    pub fn check_read_path(&self, path: &std::path::Path) -> bool {
        let ok = self.policy.check_read(path);
        if !ok { tracing::warn!("sandbox: read denied for {}", path.display()); }
        ok
    }

    /// Gate a path-write operation through the policy.
    pub fn check_write_path(&self, path: &std::path::Path) -> bool {
        let ok = self.policy.check_write(path);
        if !ok { tracing::warn!("sandbox: write denied for {}", path.display()); }
        ok
    }
}
