//! Sandbox policy: a set of granted capabilities.

use std::{collections::HashSet, path::Path};

use crate::capability::Capability;

/// A security policy expressed as a set of granted [`Capability`]s.
#[derive(Debug, Clone, Default)]
pub struct SandboxPolicy {
    grants: HashSet<Capability>,
    deny_all: bool,
}

impl SandboxPolicy {
    /// Allow every capability (default, permissive mode).
    pub fn allow_all() -> Self {
        Self { grants: HashSet::new(), deny_all: false }
    }

    /// Deny all capabilities by default (whitelist mode).
    pub fn deny_all() -> Self {
        Self { grants: HashSet::new(), deny_all: true }
    }

    /// Grant an additional capability.
    pub fn grant(mut self, cap: Capability) -> Self {
        self.grants.insert(cap);
        self
    }

    /// Check whether reading `path` is permitted.
    pub fn check_read(&self, path: &Path) -> bool {
        if !self.deny_all { return true; }
        self.grants.iter().any(|c| c.grants_read(path))
    }

    /// Check whether writing `path` is permitted.
    pub fn check_write(&self, path: &Path) -> bool {
        if !self.deny_all { return true; }
        self.grants.iter().any(|c| c.grants_write(path))
    }

    /// Check whether clock access is permitted.
    pub fn check_clock(&self) -> bool {
        if !self.deny_all { return true; }
        self.grants.contains(&Capability::Clock)
    }

    /// Check whether random access is permitted.
    pub fn check_random(&self) -> bool {
        if !self.deny_all { return true; }
        self.grants.contains(&Capability::Random)
    }

    /// Check whether process exit is permitted.
    pub fn check_proc_exit(&self) -> bool {
        if !self.deny_all { return true; }
        self.grants.contains(&Capability::ProcessExit)
    }

    /// Check whether envvar `key` is readable.
    pub fn check_env(&self, key: &str) -> bool {
        if !self.deny_all { return true; }
        self.grants.contains(&Capability::Env(key.to_owned()))
    }
}
