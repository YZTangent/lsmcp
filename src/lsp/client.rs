//! LSP client implementation
//!
//! Handles communication with a single LSP server via JSON-RPC over stdin/stdout

use crate::config::LspPackage;
use crate::types::LspError;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, Mutex, oneshot};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use url::Url;

/// JSON-RPC message types
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

/// LSP client for a single language server
pub struct LspClient {
    /// Language ID (e.g., "rust", "typescript")
    language: String,

    /// LSP package configuration
    config: LspPackage,

    /// Workspace root
    workspace_root: PathBuf,

    /// Next request ID
    next_id: Arc<AtomicU64>,

    /// Pending requests
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, LspError>>>>>,

    /// Channel to send requests to the LSP server
    request_tx: mpsc::UnboundedSender<String>,

    /// Server capabilities after initialization
    capabilities: Arc<Mutex<Option<ServerCapabilities>>>,

    /// Opened documents
    opened_documents: Arc<Mutex<HashMap<PathBuf, String>>>,

    /// Diagnostics per file
    diagnostics: Arc<Mutex<HashMap<PathBuf, Vec<Diagnostic>>>>,
}

impl LspClient {
    /// Spawn a new LSP server and create a client
    pub async fn spawn(
        language: String,
        config: LspPackage,
        workspace_root: PathBuf,
    ) -> Result<Self, LspError> {
        info!("Spawning LSP server for {}: {}", language, config.name);

        // Spawn the LSP server process
        let mut command = config.bin.primary.as_str();
        let mut args = config.bin.lsp_args.clone();

        let mut child = Command::new(command)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // TODO: Consider logging stderr
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                LspError::ServerNotFound(
                    config.name.clone(),
                    format!("Failed to spawn {}: {}. Install it first.", command, e),
                )
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            LspError::ProtocolError("Failed to get stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            LspError::ProtocolError("Failed to get stdout".to_string())
        })?;

        // Create channels for communication
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let pending = Arc::new(Mutex::new(HashMap::new()));

        // Spawn background tasks
        let pending_clone = Arc::clone(&pending);
        let diagnostics = Arc::new(Mutex::new(HashMap::new()));
        let diagnostics_clone = Arc::clone(&diagnostics);

        tokio::spawn(Self::write_loop(stdin, request_rx));
        tokio::spawn(Self::read_loop(stdout, pending_clone, diagnostics_clone));

        let client = Self {
            language: language.clone(),
            config,
            workspace_root,
            next_id: Arc::new(AtomicU64::new(1)),
            pending,
            request_tx,
            capabilities: Arc::new(Mutex::new(None)),
            opened_documents: Arc::new(Mutex::new(HashMap::new())),
            diagnostics,
        };

        // Initialize the LSP server
        client.initialize().await?;

        info!("LSP server for {} initialized successfully", language);

        Ok(client)
    }

    /// Background task to write messages to LSP server
    async fn write_loop(
        mut stdin: ChildStdin,
        mut request_rx: mpsc::UnboundedReceiver<String>,
    ) {
        while let Some(message) = request_rx.recv().await {
            let content_length = message.len();
            let header = format!("Content-Length: {}\r\n\r\n", content_length);

            if let Err(e) = stdin.write_all(header.as_bytes()).await {
                error!("Failed to write header: {}", e);
                break;
            }

            if let Err(e) = stdin.write_all(message.as_bytes()).await {
                error!("Failed to write message: {}", e);
                break;
            }

            if let Err(e) = stdin.flush().await {
                error!("Failed to flush: {}", e);
                break;
            }
        }
    }

    /// Background task to read messages from LSP server
    async fn read_loop(
        stdout: ChildStdout,
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, LspError>>>>>,
        diagnostics: Arc<Mutex<HashMap<PathBuf, Vec<Diagnostic>>>>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut headers = HashMap::new();

        loop {
            headers.clear();

            // Read headers
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        warn!("LSP server closed stdout");
                        return;
                    }
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            break;
                        }

                        if let Some((key, value)) = line.split_once(": ") {
                            headers.insert(key.to_string(), value.to_string());
                        }
                    }
                    Err(e) => {
                        error!("Failed to read header: {}", e);
                        return;
                    }
                }
            }

            // Get content length
            let content_length: usize = match headers.get("Content-Length") {
                Some(len) => match len.parse() {
                    Ok(len) => len,
                    Err(e) => {
                        error!("Invalid Content-Length: {}", e);
                        continue;
                    }
                },
                None => {
                    error!("Missing Content-Length header");
                    continue;
                }
            };

            // Read content
            let mut content = vec![0u8; content_length];
            match tokio::io::AsyncReadExt::read_exact(&mut reader, &mut content).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to read content: {}", e);
                    return;
                }
            }

            let content_str = match String::from_utf8(content) {
                Ok(s) => s,
                Err(e) => {
                    error!("Invalid UTF-8 in message: {}", e);
                    continue;
                }
            };

            debug!("Received message: {}", content_str);

            // Parse and dispatch message
            Self::handle_message(&content_str, &pending, &diagnostics).await;
        }
    }

    async fn handle_message(
        content: &str,
        pending: &Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, LspError>>>>>,
        diagnostics: &Arc<Mutex<HashMap<PathBuf, Vec<Diagnostic>>>>,
    ) {
        // Try to parse as response first
        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(content) {
            let mut pending_guard = pending.lock().await;
            if let Some(sender) = pending_guard.remove(&response.id) {
                let result = if let Some(result) = response.result {
                    Ok(result)
                } else if let Some(error) = response.error {
                    Err(LspError::ProtocolError(format!(
                        "LSP error: {}",
                        error.message
                    )))
                } else {
                    Err(LspError::ProtocolError("No result or error".to_string()))
                };

                let _ = sender.send(result);
            }
            return;
        }

        // Try to parse as notification
        if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(content) {
            // Handle publishDiagnostics notification
            if notification.method == "textDocument/publishDiagnostics" {
                if let Ok(params) = serde_json::from_value::<PublishDiagnosticsParams>(notification.params) {
                    // Convert URI to PathBuf
                    if let Ok(path) = params.uri.to_file_path() {
                        let mut diagnostics_guard = diagnostics.lock().await;
                        diagnostics_guard.insert(path, params.diagnostics);
                        debug!("Updated diagnostics for file");
                    }
                }
            }
            return;
        }

        warn!("Unknown message type: {}", content);
    }

    /// Send a request and wait for response
    async fn send_request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, LspError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: serde_json::to_value(params)?,
        };

        let message = serde_json::to_string(&request)?;
        debug!("Sending request {}: {}", id, method);

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        self.request_tx.send(message).map_err(|_| {
            LspError::ProtocolError("Failed to send request".to_string())
        })?;

        // Wait for response with timeout
        let result = timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| LspError::Timeout(30))?
            .map_err(|_| LspError::ProtocolError("Response channel closed".to_string()))??;

        serde_json::from_value(result).map_err(|e| {
            LspError::ProtocolError(format!("Failed to parse response: {}", e))
        })
    }

    /// Send a notification (no response expected)
    async fn send_notification<P: Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<(), LspError> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: serde_json::to_value(params)?,
        };

        let message = serde_json::to_string(&notification)?;
        debug!("Sending notification: {}", method);

        self.request_tx.send(message).map_err(|_| {
            LspError::ProtocolError("Failed to send notification".to_string())
        })?;

        Ok(())
    }

    /// Initialize the LSP server
    async fn initialize(&self) -> Result<(), LspError> {
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: Some(Url::from_file_path(&self.workspace_root).unwrap()),
            capabilities: ClientCapabilities::default(),
            initialization_options: self.config.initialization_options.clone(),
            ..Default::default()
        };

        let result: InitializeResult = self.send_request("initialize", params).await?;

        // Store capabilities
        *self.capabilities.lock().await = Some(result.capabilities);

        // Send initialized notification
        self.send_notification("initialized", InitializedParams {})
            .await?;

        Ok(())
    }

    /// Open a document
    pub async fn did_open(&self, file_path: &Path) -> Result<(), LspError> {
        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        // Read file content
        let text = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            LspError::Io(e)
        })?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: self.language.clone(),
                version: 1,
                text: text.clone(),
            },
        };

        self.send_notification("textDocument/didOpen", params).await?;

        // Track opened document
        self.opened_documents
            .lock()
            .await
            .insert(file_path.to_path_buf(), text);

        Ok(())
    }

    /// Close a document
    pub async fn did_close(&self, file_path: &Path) -> Result<(), LspError> {
        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        };

        self.send_notification("textDocument/didClose", params).await?;

        // Remove from tracking
        self.opened_documents.lock().await.remove(file_path);

        Ok(())
    }

    /// Get server capabilities
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.lock().await.clone()
    }

    /// Go to definition
    pub async fn goto_definition(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<GotoDefinitionResponse>, LspError> {
        // Ensure document is opened
        if !self.opened_documents.lock().await.contains_key(file_path) {
            self.did_open(file_path).await?;
        }

        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.send_request("textDocument/definition", params).await
    }

    /// Find references
    pub async fn find_references(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Vec<Location>>, LspError> {
        // Ensure document is opened
        if !self.opened_documents.lock().await.contains_key(file_path) {
            self.did_open(file_path).await?;
        }

        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            context: ReferenceContext {
                include_declaration,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.send_request("textDocument/references", params).await
    }

    /// Hover information
    pub async fn hover(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<Hover>, LspError> {
        // Ensure document is opened
        if !self.opened_documents.lock().await.contains_key(file_path) {
            self.did_open(file_path).await?;
        }

        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.send_request("textDocument/hover", params).await
    }

    /// Document symbols
    pub async fn document_symbols(
        &self,
        file_path: &Path,
    ) -> Result<Option<DocumentSymbolResponse>, LspError> {
        // Ensure document is opened
        if !self.opened_documents.lock().await.contains_key(file_path) {
            self.did_open(file_path).await?;
        }

        let uri = Url::from_file_path(file_path).map_err(|_| {
            LspError::InvalidPath(file_path.to_path_buf())
        })?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.send_request("textDocument/documentSymbol", params).await
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(
        &self,
        file_path: &Path,
    ) -> Result<Vec<Diagnostic>, LspError> {
        // Ensure document is opened to receive diagnostics
        if !self.opened_documents.lock().await.contains_key(file_path) {
            self.did_open(file_path).await?;

            // Wait a bit for diagnostics to be published
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let diagnostics_guard = self.diagnostics.lock().await;
        Ok(diagnostics_guard.get(file_path).cloned().unwrap_or_default())
    }
}
