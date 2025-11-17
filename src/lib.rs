//! LSMCP - Language Server Manager for Model Context Protocol
//!
//! This crate provides a bridge between MCP clients (like Claude Code) and
//! Language Server Protocol (LSP) servers, enabling rich code intelligence
//! for CLI-based LLM tools.

pub mod config;
pub mod installer;
pub mod lsp;
pub mod mcp;
pub mod tools;
pub mod types;
pub mod utils;

pub use config::ConfigLoader;
pub use installer::ServerInstaller;
pub use lsp::{LspClient, LspManager};
pub use mcp::McpServer;
pub use types::LspError;
