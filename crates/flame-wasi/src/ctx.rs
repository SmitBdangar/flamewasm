//! WASI context: holds all state for a single WASI instance.

use std::{
    collections::HashMap,
    path::PathBuf,
};

use crate::fd::FdTable;

/// Builder for [`WasiCtx`].
#[derive(Default)]
pub struct WasiCtxBuilder {
    args: Vec<String>,
    env: Vec<(String, String)>,
    preopened_dirs: Vec<(String, PathBuf)>,
    inherit_stdio: bool,
}

impl WasiCtxBuilder {
    pub fn new() -> Self { Self { inherit_stdio: true, ..Default::default() } }

    pub fn args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.args = args.into_iter().collect();
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.push((key.into(), val.into()));
        self
    }

    pub fn preopen_dir(mut self, guest_path: impl Into<String>, host_path: PathBuf) -> Self {
        self.preopened_dirs.push((guest_path.into(), host_path));
        self
    }

    pub fn build(self) -> WasiCtx {
        let fd_table = FdTable::new(self.preopened_dirs);
        WasiCtx {
            args: self.args,
            env: self.env,
            fd_table,
            exit_code: None,
        }
    }
}

/// All state needed to serve WASI host calls for one module instance.
pub struct WasiCtx {
    /// Command-line arguments (argv).
    pub args: Vec<String>,
    /// Environment variables as (key, value) pairs.
    pub env: Vec<(String, String)>,
    /// File descriptor table.
    pub fd_table: FdTable,
    /// Set to `Some(code)` when `proc_exit` is called.
    pub exit_code: Option<i32>,
}

impl WasiCtx {
    pub fn builder() -> WasiCtxBuilder { WasiCtxBuilder::new() }
}
