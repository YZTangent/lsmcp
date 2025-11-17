//! User configuration file parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub settings: Option<Settings>,
    pub lsp: HashMap<String, LspOverride>,
    pub language_overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub workspace_root: Option<String>,
    pub log_level: Option<String>,
    pub auto_install: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspOverride {
    pub enabled: Option<bool>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub initialization_options: Option<serde_json::Value>,
}
