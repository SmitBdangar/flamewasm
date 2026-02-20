//! WASI proc_exit.

/// Handle `proc_exit(code)` — set the exit code in the context.
///
/// In the sandbox model this does NOT call `std::process::exit` directly;
/// instead it returns an error that the CLI runner catches and exits with.
pub fn proc_exit(code: i32) -> ! {
    std::process::exit(code);
}
