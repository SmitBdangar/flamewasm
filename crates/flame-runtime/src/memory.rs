//! Linear memory implementation (Vec-backed, growable).

use anyhow::Result;
use tracing::debug;

use crate::trap::Trap;

/// The WASM page size in bytes.
pub const PAGE_SIZE: usize = 65_536;

/// A WebAssembly linear memory.
pub struct LinearMemory {
    data: Vec<u8>,
    max_pages: Option<u32>,
}

impl LinearMemory {
    /// Allocate a new linear memory with `min_pages` initial pages.
    pub fn new(min_pages: u32, max_pages: Option<u32>) -> Self {
        let size = min_pages as usize * PAGE_SIZE;
        Self {
            data: vec![0u8; size],
            max_pages,
        }
    }

    /// Current size in pages.
    #[must_use]
    pub fn size_pages(&self) -> u32 {
        (self.data.len() / PAGE_SIZE) as u32
    }

    /// Current size in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    /// Grow by `delta` pages. Returns old page count or -1 on failure.
    pub fn grow(&mut self, delta: u32) -> i32 {
        let old = self.size_pages();
        let new = match old.checked_add(delta) {
            Some(n) => n,
            None => return -1,
        };
        if let Some(max) = self.max_pages {
            if new > max { return -1; }
        }
        if new > 65_536 { return -1; } // absolute Wasm max
        let new_size = new as usize * PAGE_SIZE;
        debug!("memory.grow: {} -> {} pages", old, new);
        self.data.resize(new_size, 0);
        old as i32
    }

    /// Read bytes at `addr..addr+len`. Returns a trap on OOB.
    pub fn load_bytes(&self, addr: usize, len: usize) -> Result<&[u8], Trap> {
        self.data
            .get(addr..addr + len)
            .ok_or(Trap::MemoryOutOfBounds { offset: addr })
    }

    /// Write bytes at `addr`. Returns a trap on OOB.
    pub fn store_bytes(&mut self, addr: usize, data: &[u8]) -> Result<(), Trap> {
        let len = data.len();
        let dest = self
            .data
            .get_mut(addr..addr + len)
            .ok_or(Trap::MemoryOutOfBounds { offset: addr })?;
        dest.copy_from_slice(data);
        Ok(())
    }

    /// Return a raw pointer to the memory base (for JIT).
    pub fn base_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Return a raw mutable pointer to the memory base (for JIT).
    pub fn base_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Return the full memory slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Return the full memory slice mutably.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
