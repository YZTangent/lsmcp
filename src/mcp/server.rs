//! MCP server implementation
//!
//! Implements the Model Context Protocol server that exposes LSP
//! functionality as MCP tools via stdio.

use crate::lsp::LspManager;
use crate::mcp::protocol::*;
use crate::mcp::tools;
use anyhow::Result;
use serde_json::Value;
use std::io::{BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct McpServer {
    lsp_manager: Arc<LspManager>,
    initialized: Arc<Mutex<bool>>,
}

impl McpServer {
    pub fn new(lsp_manager: Arc<LspManager>) -> Self {
        Self {
            lsp_manager,
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Run the MCP server (blocking)
    pub async fn run(&self) -> Result<()> {
        info!("MCP server starting on stdio");

        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let mut stdout = std::io::stdout();

        loop {
            // Read newline-delimited JSON
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => {
                    info!("Client closed connection");
                    return Ok(());
                }
                Ok(_) => {
                    let line = line.trim();

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    debug!("Received request: {}", line);

                    // Handle request
                    let response = self.handle_request(line).await;

                    // Write response as newline-delimited JSON
                    let response_json = serde_json::to_string(&response)?;
                    stdout.write_all(response_json.as_bytes())?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;

                    debug!("Sent response");
                }
                Err(e) => {
                    error!("Failed to read line: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    async fn handle_request(&self, content: &str) -> JsonRpcResponse {
        // Parse request
        let request: JsonRpcRequest = match serde_json::from_str(content) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let id = request.id.clone().unwrap_or(Value::Null);

        // Handle method
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(request.params).await,
            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(error),
            },
        }
    }

    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let _params: InitializeParams = serde_json::from_value(params.unwrap_or(Value::Null))
            .map_err(|e| JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Invalid initialize params: {}", e),
                data: None,
            })?;

        *self.initialized.lock().await = true;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                experimental: None,
                logging: None,
                prompts: None,
                resources: None,
                tools: Some(serde_json::json!({})),
            },
            server_info: ServerInfo {
                name: "lsmcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError {
            code: INTERNAL_ERROR,
            message: format!("Failed to serialize result: {}", e),
            data: None,
        })
    }

    async fn handle_list_tools(&self) -> Result<Value, JsonRpcError> {
        let tools = tools::get_tool_definitions();

        let result = ListToolsResult { tools };

        serde_json::to_value(result).map_err(|e| JsonRpcError {
            code: INTERNAL_ERROR,
            message: format!("Failed to serialize tools: {}", e),
            data: None,
        })
    }

    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        if !*self.initialized.lock().await {
            return Err(JsonRpcError {
                code: INTERNAL_ERROR,
                message: "Server not initialized".to_string(),
                data: None,
            });
        }

        let params: CallToolParams = serde_json::from_value(params.unwrap_or(Value::Null))
            .map_err(|e| JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Invalid tool call params: {}", e),
                data: None,
            })?;

        let result = tools::call_tool(
            &params.name,
            params.arguments,
            Arc::clone(&self.lsp_manager),
        )
        .await;

        serde_json::to_value(result).map_err(|e| JsonRpcError {
            code: INTERNAL_ERROR,
            message: format!("Failed to serialize tool result: {}", e),
            data: None,
        })
    }
}
