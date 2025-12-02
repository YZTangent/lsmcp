//! Configuration loader with 3-tier precedence
//!
//! Priority order (highest to lowest):
//! 1. User config (~/.config/lsmcp/config.toml or .lsmcp.toml)
//! 2. Mason registry (embedded TOML files)
//! 3. Built-in defaults (hardcoded for TS/Python/Rust/Go)

use crate::config::{get_default_configs, LspPackage, UserConfig};
use crate::types::LspError;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

// Embed the registry directory at compile time
static REGISTRY_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/registry");

pub struct ConfigLoader {
    defaults: HashMap<String, LspPackage>,
    registry: HashMap<String, LspPackage>,
    user_config: Option<UserConfig>,
}

impl ConfigLoader {
    pub fn new() -> Result<Self, LspError> {
        let defaults = get_default_configs();
        info!("Loaded {} default LSP configurations", defaults.len());

        let registry = Self::load_registry()?;
        info!("Loaded {} LSP configurations from registry", registry.len());

        let user_config = Self::load_user_config()?;
        if user_config.is_some() {
            info!("Loaded user configuration");
        }

        Ok(Self {
            defaults,
            registry,
            user_config,
        })
    }

    fn load_registry() -> Result<HashMap<String, LspPackage>, LspError> {
        let mut registry = HashMap::new();

        // Iterate through all embedded .toml files
        for file in REGISTRY_DIR.files() {
            if let Some(file_name) = file.path().file_name() {
                let file_name_str = file_name.to_string_lossy();

                if file_name_str.ends_with(".toml") {
                    let content = file.contents_utf8().ok_or_else(|| {
                        LspError::ConfigError(format!("Invalid UTF-8 in {}", file_name_str))
                    })?;

                    match toml::from_str::<LspPackage>(content) {
                        Ok(package) => {
                            let lang_key = if !package.languages.is_empty() {
                                package.languages[0].clone()
                            } else {
                                package.name.clone()
                            };

                            debug!(
                                "Loaded registry entry: {} for language: {}",
                                package.name, lang_key
                            );
                            registry.insert(lang_key, package);
                        }
                        Err(e) => {
                            warn!("Failed to parse registry file {}: {}", file_name_str, e);
                        }
                    }
                }
            }
        }

        Ok(registry)
    }

    fn load_user_config() -> Result<Option<UserConfig>, LspError> {
        // Try multiple locations in priority order:
        // 1. ./.lsmcp.toml (project-specific)
        // 2. $LSMCP_CONFIG (environment variable)
        // 3. ~/.config/lsmcp/config.toml (user-global)

        let mut candidates = Vec::new();

        // Project-specific config
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join(".lsmcp.toml"));
        }

        // Environment variable
        if let Ok(config_path) = std::env::var("LSMCP_CONFIG") {
            candidates.push(PathBuf::from(config_path));
        }

        // User-global config
        if let Some(config_dir) = dirs::config_dir() {
            candidates.push(config_dir.join("lsmcp").join("config.toml"));
        }

        for path in &candidates {
            if path.exists() {
                debug!("Loading user config from: {}", path.display());
                let content = std::fs::read_to_string(path)
                    .map_err(|e| LspError::ConfigError(format!("Failed to read config: {}", e)))?;

                let config: UserConfig = toml::from_str(&content)
                    .map_err(|e| LspError::ConfigError(format!("Failed to parse config: {}", e)))?;

                return Ok(Some(config));
            }
        }

        debug!("No user config file found");
        Ok(None)
    }

    /// Get LSP configuration for a file based on its extension
    pub fn get_lsp_for_file(&self, file: &Path) -> Result<LspPackage, LspError> {
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| LspError::InvalidPath(file.to_path_buf()))?;

        self.get_lsp_for_extension(ext)
    }

    /// Get LSP configuration for a specific file extension
    pub fn get_lsp_for_extension(&self, ext: &str) -> Result<LspPackage, LspError> {
        debug!("Looking up LSP for extension: .{}", ext);

        // Check user config first
        if let Some(user_cfg) = &self.user_config {
            // Check if user has custom LSP for this extension
            for (name, _) in &user_cfg.lsp {
                // TODO: Match against file extensions in custom configs
                debug!("Found user config for LSP: {}", name);
            }
        }

        // Search in all sources: defaults, registry
        for (source_name, source) in [("defaults", &self.defaults), ("registry", &self.registry)] {
            for (lang, pkg) in source {
                if pkg.file_extensions.iter().any(|e| e == ext) {
                    debug!("Found LSP '{}' for .{} in {}", pkg.name, ext, source_name);
                    return Ok(pkg.clone());
                }
            }
        }

        Err(LspError::UnsupportedLanguage(format!(
            "No LSP found for file extension '.{}'",
            ext
        )))
    }

    /// Get LSP configuration by language name
    pub fn get_lsp_for_language(&self, language: &str) -> Result<LspPackage, LspError> {
        debug!("Looking up LSP for language: {}", language);

        // Check user config for language overrides
        if let Some(user_cfg) = &self.user_config {
            if let Some(override_lsp) = user_cfg.language_overrides.get(language) {
                debug!("User override: {} -> {}", language, override_lsp);
                return self.get_lsp_by_name(override_lsp);
            }
        }

        // Try defaults first (highest priority for built-in langs)
        if let Some(pkg) = self.defaults.get(language) {
            debug!("Found LSP for {} in defaults", language);
            return Ok(pkg.clone());
        }

        // Try registry
        if let Some(pkg) = self.registry.get(language) {
            debug!("Found LSP for {} in registry", language);
            return Ok(pkg.clone());
        }

        Err(LspError::UnsupportedLanguage(format!(
            "No LSP found for language '{}'",
            language
        )))
    }

    /// Get LSP configuration by exact name
    pub fn get_lsp_by_name(&self, name: &str) -> Result<LspPackage, LspError> {
        // Check user config
        if let Some(user_cfg) = &self.user_config {
            if let Some(_lsp_override) = user_cfg.lsp.get(name) {
                // TODO: Merge user override with base config
                debug!("Found user override for LSP: {}", name);
            }
        }

        // Search all sources
        for source in [&self.defaults, &self.registry] {
            for pkg in source.values() {
                if pkg.name == name {
                    return Ok(pkg.clone());
                }
            }
        }

        Err(LspError::ConfigError(format!("LSP '{}' not found", name)))
    }

    /// List all available LSPs
    pub fn list_available_lsps(&self) -> Vec<&LspPackage> {
        let mut lsps: Vec<&LspPackage> = Vec::new();

        // Collect from all sources (defaults take priority for duplicates)
        let mut seen = std::collections::HashSet::new();

        for pkg in self.defaults.values() {
            if seen.insert(&pkg.name) {
                lsps.push(pkg);
            }
        }

        for pkg in self.registry.values() {
            if seen.insert(&pkg.name) {
                lsps.push(pkg);
            }
        }

        lsps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_new() {
        let loader = ConfigLoader::new().expect("Failed to create ConfigLoader");
        assert!(!loader.defaults.is_empty());
    }

    #[test]
    fn test_get_lsp_for_extension() {
        let loader = ConfigLoader::new().unwrap();

        // Test TypeScript
        let ts_lsp = loader.get_lsp_for_extension("ts");
        assert!(ts_lsp.is_ok());
        assert_eq!(ts_lsp.unwrap().name, "typescript-language-server");

        // Test Python
        let py_lsp = loader.get_lsp_for_extension("py");
        assert!(py_lsp.is_ok());
        assert_eq!(py_lsp.unwrap().name, "pyright");

        // Test Rust
        let rs_lsp = loader.get_lsp_for_extension("rs");
        assert!(rs_lsp.is_ok());
        assert_eq!(rs_lsp.unwrap().name, "rust-analyzer");

        // Test Go
        let go_lsp = loader.get_lsp_for_extension("go");
        assert!(go_lsp.is_ok());
        assert_eq!(go_lsp.unwrap().name, "gopls");
    }

    #[test]
    fn test_unsupported_extension() {
        let loader = ConfigLoader::new().unwrap();
        let result = loader.get_lsp_for_extension("xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_loaded() {
        let loader = ConfigLoader::new().unwrap();

        // Should have 4 defaults + 20 from registry
        assert!(!loader.registry.is_empty(), "Registry should not be empty");

        // Verify we can access a registry-only LSP (not in defaults)
        let lua_lsp = loader.get_lsp_for_extension("lua");
        assert!(lua_lsp.is_ok(), "Should find Lua LSP from registry");
        assert_eq!(lua_lsp.unwrap().name, "lua-language-server");
    }

    #[test]
    fn test_list_available_lsps() {
        let loader = ConfigLoader::new().unwrap();
        let lsps = loader.list_available_lsps();

        // Should have at least defaults (4) + some from registry
        assert!(lsps.len() >= 4, "Should have at least 4 LSPs");

        // Verify mix of defaults and registry
        let names: Vec<&str> = lsps.iter().map(|p| p.name.as_str()).collect();
        assert!(
            names.contains(&"rust-analyzer"),
            "Should have rust-analyzer"
        );
        assert!(
            names.contains(&"typescript-language-server"),
            "Should have TypeScript LSP"
        );
    }
}
