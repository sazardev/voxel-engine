//! Per-platform adapters: storage, input mapping and the capability probe (ADR 0002).
//!
//! Adapters implement ports declared in the core; gameplay reads `Capabilities`, never the OS.

mod capabilities;

pub use capabilities::{Capabilities, GpuApi};
