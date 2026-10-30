//! Per-platform adapters: storage, input mapping and the capability probe (ADR 0002).
//!
//! Adapters implement ports declared in the core; gameplay reads `Capabilities`, never the OS.

mod capabilities;
mod file_save_store;
mod save_directory;

pub use capabilities::{Capabilities, GpuApi};
pub use file_save_store::FileSaveStore;
pub use save_directory::save_directory;
