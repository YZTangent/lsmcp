//! LSP server installer
//!
//! Automatically downloads and manages LSP server installations

use crate::config::{InstallSource, LspPackage};
use crate::types::LspError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn};

/// Manifest tracking installed LSP servers
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstallManifest {
    pub servers: HashMap<String, InstalledServer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledServer {
    pub name: String,
    pub version: Option<String>,
    pub install_date: String,
    pub binary_path: PathBuf,
    pub install_method: String,
}

/// LSP Server installer
pub struct ServerInstaller {
    /// LSMCP data directory (~/.local/share/lsmcp)
    data_dir: PathBuf,

    /// Servers directory (~/.local/share/lsmcp/servers)
    servers_dir: PathBuf,

    /// Manifest file path
    manifest_path: PathBuf,

    /// Loaded manifest
    manifest: InstallManifest,
}

impl ServerInstaller {
    /// Create a new server installer
    pub fn new() -> Result<Self, LspError> {
        let data_dir = Self::get_data_dir()?;
        let servers_dir = data_dir.join("servers");
        let manifest_path = data_dir.join("manifest.json");

        // Ensure directories exist
        fs::create_dir_all(&servers_dir).map_err(|e| {
            LspError::Io(e)
        })?;

        // Load or create manifest
        let manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(LspError::Io)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            InstallManifest::default()
        };

        Ok(Self {
            data_dir,
            servers_dir,
            manifest_path,
            manifest,
        })
    }

    /// Get LSMCP data directory
    fn get_data_dir() -> Result<PathBuf, LspError> {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            Ok(PathBuf::from(xdg_data).join("lsmcp"))
        } else if let Ok(home) = std::env::var("HOME") {
            Ok(PathBuf::from(home).join(".local/share/lsmcp"))
        } else {
            Err(LspError::ConfigError(
                "Cannot determine data directory (no $HOME or $XDG_DATA_HOME)".to_string(),
            ))
        }
    }

    /// Find LSP binary in multiple locations
    pub fn find_lsp_binary(&self, lsp_name: &str, binary_name: &str) -> Option<PathBuf> {
        // 1. Check LSMCP managed directory
        if let Some(installed) = self.manifest.servers.get(lsp_name) {
            if installed.binary_path.exists() {
                debug!("Found {} in LSMCP directory", lsp_name);
                return Some(installed.binary_path.clone());
            }
        }

        // 2. Check Mason directory
        if let Ok(home) = std::env::var("HOME") {
            let mason_path = PathBuf::from(home)
                .join(".local/share/nvim/mason/bin")
                .join(binary_name);
            if mason_path.exists() {
                debug!("Found {} in Mason directory", lsp_name);
                return Some(mason_path);
            }
        }

        // 3. Check system PATH
        if let Ok(output) = Command::new("which").arg(binary_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    let path_buf = PathBuf::from(path);
                    if path_buf.exists() {
                        debug!("Found {} in system PATH", lsp_name);
                        return Some(path_buf);
                    }
                }
            }
        }

        None
    }

    /// Install an LSP server
    pub async fn install_lsp(&mut self, package: &LspPackage) -> Result<PathBuf, LspError> {
        info!("Installing LSP server: {}", package.name);

        let binary_path = match &package.source {
            InstallSource::Npm { package: npm_pkg, .. } => {
                self.install_npm(npm_pkg, &package.bin.primary).await?
            }
            InstallSource::Cargo { crate_name, .. } => {
                self.install_cargo(crate_name, &package.bin.primary).await?
            }
            InstallSource::Go { package: go_pkg, .. } => {
                self.install_go(go_pkg, &package.bin.primary).await?
            }
            InstallSource::External { command } => {
                return Err(LspError::ServerNotFound(
                    package.name.clone(),
                    format!("Cannot auto-install external command: {}. Please install manually.", command),
                ));
            }
            _ => {
                return Err(LspError::ServerNotFound(
                    package.name.clone(),
                    format!("Auto-installation not yet supported for this install source type."),
                ));
            }
        };

        // Record installation in manifest
        self.manifest.servers.insert(
            package.name.clone(),
            InstalledServer {
                name: package.name.clone(),
                version: None, // TODO: Extract version
                install_date: chrono::Utc::now().to_rfc3339(),
                binary_path: binary_path.clone(),
                install_method: format!("{:?}", package.source),
            },
        );

        self.save_manifest()?;

        info!("Successfully installed {}", package.name);
        Ok(binary_path)
    }

    /// Install from npm
    async fn install_npm(&self, package: &str, binary: &str) -> Result<PathBuf, LspError> {
        info!("Installing {} via npm", package);

        let server_dir = self.servers_dir.join(package);
        fs::create_dir_all(&server_dir).map_err(LspError::Io)?;

        // Install locally to server directory
        let output = AsyncCommand::new("npm")
            .args(&["install", "--prefix", server_dir.to_str().unwrap(), package])
            .output()
            .await
            .map_err(|e| {
                LspError::ServerNotFound(
                    package.to_string(),
                    format!("npm not found or failed: {}", e),
                )
            })?;

        if !output.status.success() {
            return Err(LspError::ServerNotFound(
                package.to_string(),
                format!("npm install failed: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }

        // Find the binary in node_modules/.bin/
        let binary_path = server_dir.join("node_modules/.bin").join(binary);

        if !binary_path.exists() {
            return Err(LspError::ServerNotFound(
                package.to_string(),
                format!("Binary {} not found after npm install", binary),
            ));
        }

        Ok(binary_path)
    }

    /// Install from cargo
    async fn install_cargo(&self, crate_name: &str, binary: &str) -> Result<PathBuf, LspError> {
        info!("Installing {} via cargo", crate_name);

        let output = AsyncCommand::new("cargo")
            .args(&["install", crate_name, "--root", self.servers_dir.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| {
                LspError::ServerNotFound(
                    crate_name.to_string(),
                    format!("cargo not found or failed: {}", e),
                )
            })?;

        if !output.status.success() {
            return Err(LspError::ServerNotFound(
                crate_name.to_string(),
                format!("cargo install failed: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }

        let binary_path = self.servers_dir.join("bin").join(binary);

        if !binary_path.exists() {
            return Err(LspError::ServerNotFound(
                crate_name.to_string(),
                format!("Binary {} not found after cargo install", binary),
            ));
        }

        Ok(binary_path)
    }

    /// Install from go
    async fn install_go(&self, package: &str, binary: &str) -> Result<PathBuf, LspError> {
        info!("Installing {} via go install", package);

        let gobin = self.servers_dir.join("go-bin");
        fs::create_dir_all(&gobin).map_err(LspError::Io)?;

        let output = AsyncCommand::new("go")
            .args(&["install", &format!("{}@latest", package)])
            .env("GOBIN", gobin.to_str().unwrap())
            .output()
            .await
            .map_err(|e| {
                LspError::ServerNotFound(
                    package.to_string(),
                    format!("go not found or failed: {}", e),
                )
            })?;

        if !output.status.success() {
            return Err(LspError::ServerNotFound(
                package.to_string(),
                format!("go install failed: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }

        let binary_path = gobin.join(binary);

        if !binary_path.exists() {
            return Err(LspError::ServerNotFound(
                package.to_string(),
                format!("Binary {} not found after go install", binary),
            ));
        }

        Ok(binary_path)
    }

    /// Save manifest to disk
    fn save_manifest(&self) -> Result<(), LspError> {
        let content = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| LspError::ConfigError(format!("Failed to serialize manifest: {}", e)))?;

        fs::write(&self.manifest_path, content).map_err(LspError::Io)?;

        Ok(())
    }

    /// List all installed servers
    pub fn list_installed(&self) -> Vec<&InstalledServer> {
        self.manifest.servers.values().collect()
    }
}
