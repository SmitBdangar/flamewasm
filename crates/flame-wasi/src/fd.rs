//! WASI file descriptor table and fd_* host functions.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

/// WASI errno values (preview-1 subset).
#[allow(dead_code)]
pub mod errno {
    pub const SUCCESS: u32 = 0;
    pub const BADF: u32 = 8;
    pub const INVAL: u32 = 28;
    pub const IO: u32 = 29;
    pub const NOENT: u32 = 44;
    pub const NOTDIR: u32 = 54;
    pub const ACCES: u32 = 2;
    pub const NOTSUP: u32 = 58;
}

/// A single open file descriptor entry.
pub enum FdEntry {
    Stdin,
    Stdout,
    Stderr,
    File { path: PathBuf, file: File },
    PreopenDir { guest: String, host: PathBuf },
}

/// The file descriptor table for a WASI instance.
pub struct FdTable {
    entries: Vec<Option<FdEntry>>,
}

impl FdTable {
    /// Create a new table with stdio (fds 0/1/2) and preopened directories.
    pub fn new(preopens: Vec<(String, PathBuf)>) -> Self {
        let mut entries: Vec<Option<FdEntry>> = vec![
            Some(FdEntry::Stdin),
            Some(FdEntry::Stdout),
            Some(FdEntry::Stderr),
        ];
        for (guest, host) in preopens {
            entries.push(Some(FdEntry::PreopenDir { guest, host }));
        }
        Self { entries }
    }

    pub fn get(&self, fd: u32) -> Option<&FdEntry> {
        self.entries.get(fd as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, fd: u32) -> Option<&mut FdEntry> {
        self.entries.get_mut(fd as usize)?.as_mut()
    }

    pub fn close(&mut self, fd: u32) -> u32 {
        if let Some(slot) = self.entries.get_mut(fd as usize) {
            *slot = None;
            errno::SUCCESS
        } else {
            errno::BADF
        }
    }

    /// `fd_write`: write iovec list to fd, return nwritten.
    pub fn fd_write(&mut self, fd: u32, data: &[u8]) -> (u32, u32) {
        match fd {
            1 => { print!("{}", String::from_utf8_lossy(data)); (data.len() as u32, errno::SUCCESS) }
            2 => { eprint!("{}", String::from_utf8_lossy(data)); (data.len() as u32, errno::SUCCESS) }
            _ => {
                if let Some(Some(FdEntry::File { file, .. })) = self.entries.get_mut(fd as usize) {
                    match file.write_all(data) {
                        Ok(()) => (data.len() as u32, errno::SUCCESS),
                        Err(_) => (0, errno::IO),
                    }
                } else {
                    (0, errno::BADF)
                }
            }
        }
    }

    /// `fd_read`: read from fd into buffer, return nread.
    pub fn fd_read(&mut self, fd: u32, buf: &mut Vec<u8>, max: usize) -> (u32, u32) {
        match fd {
            0 => {
                buf.resize(max, 0);
                match std::io::stdin().read(buf) {
                    Ok(n) => { buf.truncate(n); (n as u32, errno::SUCCESS) }
                    Err(_) => (0, errno::IO),
                }
            }
            _ => {
                if let Some(Some(FdEntry::File { file, .. })) = self.entries.get_mut(fd as usize) {
                    buf.resize(max, 0);
                    match file.read(buf) {
                        Ok(n) => { buf.truncate(n); (n as u32, errno::SUCCESS) }
                        Err(_) => (0, errno::IO),
                    }
                } else {
                    (0, errno::BADF)
                }
            }
        }
    }

    /// Number of preopened directories (fds 3..).
    pub fn preopen_count(&self) -> u32 {
        self.entries.iter().skip(3).filter(|e| {
            matches!(e, Some(FdEntry::PreopenDir { .. }))
        }).count() as u32
    }

    /// Get preopen at index (fd = 3 + index).
    pub fn preopen_at(&self, idx: u32) -> Option<(&str, &PathBuf)> {
        let fd = 3 + idx as usize;
        if let Some(Some(FdEntry::PreopenDir { guest, host })) = self.entries.get(fd) {
            Some((guest.as_str(), host))
        } else {
            None
        }
    }
}
