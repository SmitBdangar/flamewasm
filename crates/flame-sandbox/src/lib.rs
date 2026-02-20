//! `flame-sandbox`: Capability-based security policy for FlameWasm.

pub mod capability;
pub mod enforcer;
pub mod policy;

pub use capability::Capability;
pub use enforcer::SandboxCtx;
pub use policy::SandboxPolicy;
