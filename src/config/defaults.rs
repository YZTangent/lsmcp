//! Built-in default LSP configurations
//!
//! These provide zero-config support for the most popular languages

use crate::config::registry::{BinaryConfig, InstallSource, LspPackage};
use std::collections::HashMap;

pub fn get_default_configs() -> HashMap<String, LspPackage> {
    let mut configs = HashMap::new();

    // TypeScript/JavaScript
    configs.insert("typescript".to_string(), typescript_config());
    configs.insert("javascript".to_string(), typescript_config());

    // Python
    configs.insert("python".to_string(), python_config());

    // Rust
    configs.insert("rust".to_string(), rust_config());

    // Go
    configs.insert("go".to_string(), go_config());

    configs
}

fn typescript_config() -> LspPackage {
    LspPackage {
        name: "typescript-language-server".to_string(),
        description: "TypeScript & JavaScript Language Server".to_string(),
        homepage: Some("https://github.com/typescript-language-server/typescript-language-server".to_string()),
        licenses: vec!["MIT".to_string()],
        languages: vec!["typescript".to_string(), "javascript".to_string()],
        file_extensions: vec![
            "ts".to_string(),
            "tsx".to_string(),
            "js".to_string(),
            "jsx".to_string(),
            "mjs".to_string(),
            "cjs".to_string(),
        ],
        source: InstallSource::Npm {
            package: "typescript-language-server".to_string(),
            version: None,
        },
        bin: BinaryConfig {
            primary: "typescript-language-server".to_string(),
            additional: vec![],
            lsp_args: vec!["--stdio".to_string()],
        },
        initialization_options: None,
    }
}

fn python_config() -> LspPackage {
    LspPackage {
        name: "pyright".to_string(),
        description: "Static type checker and language server for Python".to_string(),
        homepage: Some("https://github.com/microsoft/pyright".to_string()),
        licenses: vec!["MIT".to_string()],
        languages: vec!["python".to_string()],
        file_extensions: vec!["py".to_string(), "pyi".to_string()],
        source: InstallSource::Npm {
            package: "pyright".to_string(),
            version: None,
        },
        bin: BinaryConfig {
            primary: "pyright-langserver".to_string(),
            additional: vec!["pyright".to_string()],
            lsp_args: vec!["--stdio".to_string()],
        },
        initialization_options: None,
    }
}

fn rust_config() -> LspPackage {
    LspPackage {
        name: "rust-analyzer".to_string(),
        description: "Implementation of Language Server Protocol for Rust".to_string(),
        homepage: Some("https://rust-analyzer.github.io/".to_string()),
        licenses: vec!["MIT".to_string(), "Apache-2.0".to_string()],
        languages: vec!["rust".to_string()],
        file_extensions: vec!["rs".to_string()],
        source: InstallSource::External {
            command: "rust-analyzer".to_string(),
        },
        bin: BinaryConfig {
            primary: "rust-analyzer".to_string(),
            additional: vec![],
            lsp_args: vec![],
        },
        initialization_options: None,
    }
}

fn go_config() -> LspPackage {
    LspPackage {
        name: "gopls".to_string(),
        description: "Official Go language server".to_string(),
        homepage: Some("https://github.com/golang/tools/tree/master/gopls".to_string()),
        licenses: vec!["BSD-3-Clause".to_string()],
        languages: vec!["go".to_string()],
        file_extensions: vec!["go".to_string()],
        source: InstallSource::External {
            command: "gopls".to_string(),
        },
        bin: BinaryConfig {
            primary: "gopls".to_string(),
            additional: vec![],
            lsp_args: vec![],
        },
        initialization_options: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_configs() {
        let configs = get_default_configs();
        assert!(configs.contains_key("typescript"));
        assert!(configs.contains_key("python"));
        assert!(configs.contains_key("rust"));
        assert!(configs.contains_key("go"));
    }

    #[test]
    fn test_typescript_config() {
        let config = typescript_config();
        assert_eq!(config.name, "typescript-language-server");
        assert!(config.file_extensions.contains(&"ts".to_string()));
        assert!(config.file_extensions.contains(&"js".to_string()));
    }
}
