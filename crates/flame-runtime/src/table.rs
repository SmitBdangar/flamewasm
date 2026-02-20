//! Table implementation (funcref/externref).

use crate::trap::Trap;

/// A WebAssembly table of function references.
pub struct Table {
    elements: Vec<Option<u32>>,
    max: Option<u32>,
}

impl Table {
    pub fn new(min: u32, max: Option<u32>) -> Self {
        Self { elements: vec![None; min as usize], max }
    }

    pub fn size(&self) -> u32 { self.elements.len() as u32 }

    pub fn get(&self, idx: u32) -> Result<Option<u32>, Trap> {
        self.elements
            .get(idx as usize)
            .copied()
            .ok_or(Trap::TableOutOfBounds { index: idx })
    }

    pub fn set(&mut self, idx: u32, val: Option<u32>) -> Result<(), Trap> {
        let elem = self
            .elements
            .get_mut(idx as usize)
            .ok_or(Trap::TableOutOfBounds { index: idx })?;
        *elem = val;
        Ok(())
    }

    pub fn grow(&mut self, delta: u32, init: Option<u32>) -> i32 {
        let old = self.size();
        let new = match old.checked_add(delta) {
            Some(n) => n,
            None => return -1,
        };
        if let Some(max) = self.max {
            if new > max { return -1; }
        }
        self.elements.resize(new as usize, init);
        old as i32
    }
}
