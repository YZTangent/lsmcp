use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("LSP server not found: {0}. Install with: {1}")]
    ServerNotFound(String, String),
    
    #[error("LSP server crashed: {0}")]
    ServerCrashed(String),
    
    #[error("Request timeout after {0}s")]
    Timeout(u64),
    
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),
    
    #[error("LSP protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, LspError>;
