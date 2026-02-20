//! WASI clock_time_get / clock_res_get.

use std::time::{SystemTime, UNIX_EPOCH};

/// WASI clock IDs (preview-1).
pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;
pub const CLOCK_PROCESS_CPU: u32 = 2;
pub const CLOCK_THREAD_CPU: u32 = 3;

/// Return the current time in nanoseconds for the given clock ID.
pub fn clock_time_get(id: u32) -> (u64, u32) {
    match id {
        CLOCK_REALTIME => {
            let ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            (ns, 0)
        }
        CLOCK_MONOTONIC => {
            // Use a simple monotonic approximation via SystemTime
            let ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            (ns, 0)
        }
        _ => (0, 28), // EINVAL
    }
}

/// Return the resolution (precision) of the given clock in nanoseconds.
pub fn clock_res_get(id: u32) -> (u64, u32) {
    match id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => (1_000, 0), // 1 µs precision
        _ => (0, 28),
    }
}
