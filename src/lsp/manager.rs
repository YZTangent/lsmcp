//! LSP manager for lifecycle management
//!
//! Manages a pool of LSP clients, one per language, with lazy initialization

use crate::config::{ConfigLoader, LspPackage};
use crate::lsp::LspClient;
use crate::types::LspError;
use lsp_types::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// LSP Manager handles lifecycle of all LSP clients
pub struct LspManager {
    /// Workspace root directory
    workspace_root: PathBuf,

    /// Configuration loader
    config: Arc<ConfigLoader>,

    /// Active LSP clients (language -> client)
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>,
}

impl LspManager {
    /// Create a new LSP manager
    pub fn new(workspace_root: PathBuf, config: Arc<ConfigLoader>) -> Result<Self, LspError> {
        info!("Creating LSP manager for workspace: {}", workspace_root.display());

        Ok(Self {
            workspace_root,
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get or create an LSP client for a language
    async fn get_or_create_client(&self, language: &str) -> Result<Arc<LspClient>, LspError> {
        let mut clients = self.clients.lock().await;

        // Check if client already exists
        if let Some(client) = clients.get(language) {
            debug!("Reusing existing LSP client for {}", language);
            return Ok(Arc::clone(client));
        }

        // Get LSP configuration for this language
        let lsp_config = self.config.get_lsp_for_language(language)?;

        info!("Initializing new LSP client for {}: {}", language, lsp_config.name);

        // Spawn new LSP client
        let client = LspClient::spawn(
            language.to_string(),
            lsp_config,
            self.workspace_root.clone(),
        ).await?;

        let client = Arc::new(client);
        clients.insert(language.to_string(), Arc::clone(&client));

        Ok(client)
    }

    /// Get LSP client for a file (by extension)
    async fn get_client_for_file(&self, file_path: &Path) -> Result<Arc<LspClient>, LspError> {
        // Detect language from file extension
        let lsp_config = self.config.get_lsp_for_file(file_path)?;
        let language = &lsp_config.languages[0];

        self.get_or_create_client(language).await
    }

    /// Go to definition
    pub async fn goto_definition(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<GotoDefinitionResponse>, LspError> {
        let client = self.get_client_for_file(file_path).await?;
        client.goto_definition(file_path, line, character).await
    }

    /// Find references
    pub async fn find_references(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Vec<Location>>, LspError> {
        let client = self.get_client_for_file(file_path).await?;
        client.find_references(file_path, line, character, include_declaration).await
    }

    /// Get hover information
    pub async fn hover(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<Hover>, LspError> {
        let client = self.get_client_for_file(file_path).await?;
        client.hover(file_path, line, character).await
    }

    /// Get document symbols
    pub async fn document_symbols(
        &self,
        file_path: &Path,
    ) -> Result<Option<DocumentSymbolResponse>, LspError> {
        let client = self.get_client_for_file(file_path).await?;
        client.document_symbols(file_path).await
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(
        &self,
        file_path: &Path,
    ) -> Result<Vec<Diagnostic>, LspError> {
        let client = self.get_client_for_file(file_path).await?;
        client.get_diagnostics(file_path).await
    }

    /// Search for symbols across the workspace
    pub async fn workspace_symbols(
        &self,
        query: String,
        language: &str,
    ) -> Result<Option<Vec<SymbolInformation>>, LspError> {
        let client = self.get_or_create_client(language).await?;
        client.workspace_symbols(query).await
    }

    /// Get status of all active LSP clients
    pub async fn status(&self) -> Vec<(String, bool)> {
        let clients = self.clients.lock().await;
        clients.iter()
            .map(|(lang, _client)| (lang.clone(), true))
            .collect()
    }

    /// Shutdown all LSP clients gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down all LSP clients");
        let mut clients = self.clients.lock().await;

        for (language, client) in clients.drain() {
            info!("Shutting down LSP client for {}", language);
            // Clients will be dropped here, triggering process cleanup via kill_on_drop
            drop(client);
        }

        info!("All LSP clients shut down");
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        // Ensure graceful shutdown on drop
        // Note: We can't await in Drop, so we just log
        debug!("LspManager dropped");
    }
}
