//! Capability tokens representing individual access rights.

use std::path::PathBuf;

/// A single capability token granting one access right.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Read access to a directory tree.
    ReadDir(PathBuf),
    /// Write access to a directory tree.
    WriteDir(PathBuf),
    /// Read access to a specific file.
    ReadFile(PathBuf),
    /// Write access to a specific file.
    WriteFile(PathBuf),
    /// Access to the system clock (`clock_time_get`).
    Clock,
    /// Access to random bytes (`random_get`).
    Random,
    /// Outbound network access (reserved; not yet enforced).
    Network,
    /// Read a specific environment variable by name.
    Env(String),
    /// Call `proc_exit`.
    ProcessExit,
}

impl Capability {
    /// Returns `true` if this capability grants read access to `path`.
    pub fn grants_read(&self, path: &std::path::Path) -> bool {
        match self {
            Self::ReadDir(base) | Self::WriteDir(base) => path.starts_with(base),
            Self::ReadFile(f) | Self::WriteFile(f) => f == path,
            _ => false,
        }
    }

    /// Returns `true` if this capability grants write access to `path`.
    pub fn grants_write(&self, path: &std::path::Path) -> bool {
        match self {
            Self::WriteDir(base) => path.starts_with(base),
            Self::WriteFile(f) => f == path,
            _ => false,
        }
    }
}
