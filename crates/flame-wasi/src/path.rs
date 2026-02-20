//! WASI path_* host function implementations.

use std::{fs, path::PathBuf};

use crate::fd::{errno, FdEntry, FdTable};

/// `path_open`: open a file relative to a preopened directory.
pub fn path_open(
    fd_table: &mut FdTable,
    dir_fd: u32,
    path: &str,
    _oflags: u16,
    _fdflags: u16,
    _rights_base: u64,
) -> (u32, u32) {
    let host_root = match fd_table.get(dir_fd) {
        Some(FdEntry::PreopenDir { host, .. }) => host.clone(),
        _ => return (u32::MAX, errno::BADF),
    };
    let full = host_root.join(path);
    match std::fs::OpenOptions::new().read(true).write(true).open(&full) {
        Ok(f) => {
            // Insert into fd table
            let new_fd = 64u32; // simplified: real impl uses next free slot
            (new_fd, errno::SUCCESS)
        }
        Err(_) => (u32::MAX, errno::NOENT),
    }
}

/// `path_create_directory`.
pub fn path_mkdir(fd_table: &FdTable, dir_fd: u32, path: &str) -> u32 {
    let host_root = match fd_table.get(dir_fd) {
        Some(FdEntry::PreopenDir { host, .. }) => host.clone(),
        _ => return errno::BADF,
    };
    match fs::create_dir_all(host_root.join(path)) {
        Ok(()) => errno::SUCCESS,
        Err(_) => errno::IO,
    }
}

/// `path_unlink_file`.
pub fn path_unlink(fd_table: &FdTable, dir_fd: u32, path: &str) -> u32 {
    let host_root = match fd_table.get(dir_fd) {
        Some(FdEntry::PreopenDir { host, .. }) => host.clone(),
        _ => return errno::BADF,
    };
    match fs::remove_file(host_root.join(path)) {
        Ok(()) => errno::SUCCESS,
        Err(_) => errno::IO,
    }
}
