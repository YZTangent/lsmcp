//! LSP package registry types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPackage {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub licenses: Vec<String>,
    pub languages: Vec<String>,
    pub file_extensions: Vec<String>,
    pub source: InstallSource,
    pub bin: BinaryConfig,
    pub initialization_options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallSource {
    Npm {
        package: String,
        version: Option<String>,
    },
    Cargo {
        crate_name: String,
        version: Option<String>,
    },
    Pip {
        package: String,
        version: Option<String>,
    },
    GithubRelease {
        repo: String,
        tag: Option<String>,
    },
    System {
        packages: HashMap<String, String>,
    },
    External {
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    pub primary: String,
    pub additional: Vec<String>,
    pub lsp_args: Vec<String>,
}
