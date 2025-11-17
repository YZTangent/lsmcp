#!/usr/bin/env rust-script
//! Mason Registry Sync Script
//!
//! This script fetches LSP package definitions from the Mason registry
//! and converts them to our TOML format.
//!
//! Usage: cargo run --bin sync-mason-registry
//!
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["blocking"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_yaml = "0.9"
//! serde_json = "1.0"
//! toml = "0.8"
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const MASON_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/mason-org/mason-registry/main/packages";

// Mason YAML package structure (simplified)
#[derive(Debug, Deserialize)]
struct MasonPackage {
    name: String,
    description: String,
    homepage: Option<String>,
    licenses: Vec<String>,
    languages: Vec<String>,
    categories: Vec<String>,
    source: MasonSource,
}

#[derive(Debug, Deserialize)]
struct MasonSource {
    id: String,
}

// Our TOML package structure
#[derive(Debug, Serialize)]
struct LsmcpPackage {
    name: String,
    description: String,
    homepage: Option<String>,
    licenses: Vec<String>,
    languages: Vec<String>,
    file_extensions: Vec<String>,
    source: LsmcpSource,
    bin: BinaryConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    initialization_options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum LsmcpSource {
    Npm { package: String },
    Cargo { crate_name: String },
    Pip { package: String },
    GithubRelease { repo: String },
    External { command: String },
}

#[derive(Debug, Serialize)]
struct BinaryConfig {
    primary: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    additional: Vec<String>,
    lsp_args: Vec<String>,
}

// Curated list of LSPs to sync
const LSP_PACKAGES: &[&str] = &[
    "rust-analyzer",
    "typescript-language-server",
    "pyright",
    "gopls",
    "lua-language-server",
    "clangd",
    "jdtls",
    "zls",
    "solargraph",
    "elixir-ls",
    "haskell-language-server",
    "metals",
    "ocaml-lsp",
    "texlab",
    "taplo",
    "yaml-language-server",
    "json-lsp",
    "css-lsp",
    "html-lsp",
    "svelte-language-server",
];

fn main() -> Result<()> {
    println!("🔄 Syncing LSP configs from Mason registry...\n");

    // Ensure registry directory exists
    let registry_dir = Path::new("registry");
    fs::create_dir_all(registry_dir)?;

    let mut synced = 0;
    let mut failed = Vec::new();

    for package_name in LSP_PACKAGES {
        match sync_package(package_name, registry_dir) {
            Ok(()) => {
                println!("  ✓ {}", package_name);
                synced += 1;
            }
            Err(e) => {
                println!("  ✗ {} - {}", package_name, e);
                failed.push(package_name);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("  Synced: {}/{}", synced, LSP_PACKAGES.len());
    if !failed.is_empty() {
        println!("  Failed: {:?}", failed);
    }

    Ok(())
}

fn sync_package(package_name: &str, registry_dir: &Path) -> Result<()> {
    // For now, manually create configs instead of fetching from Mason
    // (network access may not be available in all environments)
    let package = create_manual_config(package_name)?;

    // Write to TOML file
    let toml_content = toml::to_string_pretty(&package)?;
    let output_path = registry_dir.join(format!("{}.toml", package_name));
    fs::write(&output_path, toml_content)?;

    Ok(())
}

fn create_manual_config(name: &str) -> Result<LsmcpPackage> {
    match name {
        "rust-analyzer" => Ok(LsmcpPackage {
            name: "rust-analyzer".into(),
            description: "Implementation of LSP for Rust".into(),
            homepage: Some("https://rust-analyzer.github.io/".into()),
            licenses: vec!["MIT".into(), "Apache-2.0".into()],
            languages: vec!["rust".into()],
            file_extensions: vec!["rs".into()],
            source: LsmcpSource::External {
                command: "rust-analyzer".into(),
            },
            bin: BinaryConfig {
                primary: "rust-analyzer".into(),
                additional: vec![],
                lsp_args: vec![],
            },
            initialization_options: None,
        }),

        "lua-language-server" => Ok(LsmcpPackage {
            name: "lua-language-server".into(),
            description: "Lua language server".into(),
            homepage: Some("https://github.com/LuaLS/lua-language-server".into()),
            licenses: vec!["MIT".into()],
            languages: vec!["lua".into()],
            file_extensions: vec!["lua".into()],
            source: LsmcpSource::External {
                command: "lua-language-server".into(),
            },
            bin: BinaryConfig {
                primary: "lua-language-server".into(),
                additional: vec![],
                lsp_args: vec![],
            },
            initialization_options: None,
        }),

        "clangd" => Ok(LsmcpPackage {
            name: "clangd".into(),
            description: "C/C++/Objective-C language server".into(),
            homepage: Some("https://clangd.llvm.org/".into()),
            licenses: vec!["Apache-2.0".into()],
            languages: vec!["c".into(), "cpp".into(), "objc".into()],
            file_extensions: vec!["c".into(), "h".into(), "cpp".into(), "hpp".into(), "cc".into(), "cxx".into(), "m".into()],
            source: LsmcpSource::External {
                command: "clangd".into(),
            },
            bin: BinaryConfig {
                primary: "clangd".into(),
                additional: vec![],
                lsp_args: vec![],
            },
            initialization_options: None,
        }),

        "zls" => Ok(LsmcpPackage {
            name: "zls".into(),
            description: "Zig language server".into(),
            homepage: Some("https://github.com/zigtools/zls".into()),
            licenses: vec!["MIT".into()],
            languages: vec!["zig".into()],
            file_extensions: vec!["zig".into()],
            source: LsmcpSource::External {
                command: "zls".into(),
            },
            bin: BinaryConfig {
                primary: "zls".into(),
                additional: vec![],
                lsp_args: vec![],
            },
            initialization_options: None,
        }),

        "solargraph" => Ok(LsmcpPackage {
            name: "solargraph".into(),
            description: "Ruby language server".into(),
            homepage: Some("https://solargraph.org/".into()),
            licenses: vec!["MIT".into()],
            languages: vec!["ruby".into()],
            file_extensions: vec!["rb".into(), "rake".into()],
            source: LsmcpSource::External {
                command: "solargraph".into(),
            },
            bin: BinaryConfig {
                primary: "solargraph".into(),
                additional: vec![],
                lsp_args: vec!["stdio".into()],
            },
            initialization_options: None,
        }),

        "yaml-language-server" => Ok(LsmcpPackage {
            name: "yaml-language-server".into(),
            description: "YAML language server".into(),
            homepage: Some("https://github.com/redhat-developer/yaml-language-server".into()),
            licenses: vec!["MIT".into()],
            languages: vec!["yaml".into()],
            file_extensions: vec!["yaml".into(), "yml".into()],
            source: LsmcpSource::Npm {
                package: "yaml-language-server".into(),
            },
            bin: BinaryConfig {
                primary: "yaml-language-server".into(),
                additional: vec![],
                lsp_args: vec!["--stdio".into()],
            },
            initialization_options: None,
        }),

        "taplo" => Ok(LsmcpPackage {
            name: "taplo".into(),
            description: "TOML language server".into(),
            homepage: Some("https://taplo.tamasfe.dev/".into()),
            licenses: vec!["MIT".into()],
            languages: vec!["toml".into()],
            file_extensions: vec!["toml".into()],
            source: LsmcpSource::External {
                command: "taplo".into(),
            },
            bin: BinaryConfig {
                primary: "taplo".into(),
                additional: vec![],
                lsp_args: vec!["lsp".into(), "stdio".into()],
            },
            initialization_options: None,
        }),

        _ => anyhow::bail!("Unknown package: {}", name),
    }
}
