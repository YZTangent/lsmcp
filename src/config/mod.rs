//! Configuration system for LSMCP
//!
//! Provides a 3-tier configuration hierarchy:
//! 1. User config (highest priority)
//! 2. Mason registry (medium priority)
//! 3. Built-in defaults (lowest priority)

mod defaults;
mod loader;
mod registry;
mod user_config;

pub use defaults::get_default_configs;
pub use loader::ConfigLoader;
pub use registry::{LspPackage, InstallSource, BinaryConfig};
pub use user_config::UserConfig;
