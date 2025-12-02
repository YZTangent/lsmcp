//! MCP tools implementation
//!
//! Defines and implements all MCP tools that expose LSP functionality

use crate::lsp::LspManager;
use crate::mcp::protocol::{CallToolResult, Tool, ToolContent};
use lsp_types::*;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;

/// Get all tool definitions
pub fn get_tool_definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "lsp_goto_definition".to_string(),
            description: "Navigate to the definition of a symbol at a given position in a file. Returns the location(s) where the symbol is defined.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number (0-indexed)"
                    },
                    "character": {
                        "type": "integer",
                        "description": "Character offset in line (0-indexed)"
                    }
                },
                "required": ["file", "line", "character"]
            }),
        },
        Tool {
            name: "lsp_find_references".to_string(),
            description: "Find all references to a symbol at a given position. Returns all locations where the symbol is used.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number (0-indexed)"
                    },
                    "character": {
                        "type": "integer",
                        "description": "Character offset in line (0-indexed)"
                    },
                    "includeDeclaration": {
                        "type": "boolean",
                        "description": "Include the declaration in results",
                        "default": true
                    }
                },
                "required": ["file", "line", "character"]
            }),
        },
        Tool {
            name: "lsp_hover".to_string(),
            description: "Get hover information (documentation, type info, signatures) for a symbol at a given position.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number (0-indexed)"
                    },
                    "character": {
                        "type": "integer",
                        "description": "Character offset in line (0-indexed)"
                    }
                },
                "required": ["file", "line", "character"]
            }),
        },
        Tool {
            name: "lsp_document_symbols".to_string(),
            description: "Get the symbol outline for a file (classes, functions, variables, etc.). Returns a hierarchical structure of all symbols in the file.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    }
                },
                "required": ["file"]
            }),
        },
        Tool {
            name: "lsp_diagnostics".to_string(),
            description: "Get diagnostics (errors, warnings, hints) for a file from the LSP server. Shows compiler errors, linting issues, and other problems.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    }
                },
                "required": ["file"]
            }),
        },
        Tool {
            name: "lsp_workspace_symbols".to_string(),
            description: "Search for symbols across the entire workspace by name or pattern. Useful for finding functions, classes, variables, etc. across multiple files.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (symbol name or pattern)"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language to search in (e.g., 'rust', 'typescript', 'python', 'go')"
                    }
                },
                "required": ["query", "language"]
            }),
        },
    ]
}

/// Call a tool by name
pub async fn call_tool(
    name: &str,
    arguments: Option<Value>,
    lsp_manager: Arc<LspManager>,
) -> CallToolResult {
    let args = arguments.unwrap_or(Value::Null);

    match name {
        "lsp_goto_definition" => handle_goto_definition(args, lsp_manager).await,
        "lsp_find_references" => handle_find_references(args, lsp_manager).await,
        "lsp_hover" => handle_hover(args, lsp_manager).await,
        "lsp_document_symbols" => handle_document_symbols(args, lsp_manager).await,
        "lsp_diagnostics" => handle_diagnostics(args, lsp_manager).await,
        "lsp_workspace_symbols" => handle_workspace_symbols(args, lsp_manager).await,
        _ => CallToolResult {
            content: vec![ToolContent::Text {
                text: format!("Unknown tool: {}", name),
            }],
            is_error: Some(true),
        },
    }
}

#[derive(Debug, Deserialize)]
struct GotoDefinitionArgs {
    file: String,
    line: u32,
    character: u32,
}

async fn handle_goto_definition(
    args: Value,
    lsp_manager: Arc<LspManager>,
) -> CallToolResult {
    let args: GotoDefinitionArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    let file_path = PathBuf::from(&args.file);

    match lsp_manager
        .goto_definition(&file_path, args.line, args.character)
        .await
    {
        Ok(Some(response)) => {
            let text = format_definition_response(response);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Ok(None) => CallToolResult {
            content: vec![ToolContent::Text {
                text: "No definition found".to_string(),
            }],
            is_error: None,
        },
        Err(e) => {
            error!("goto_definition error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct FindReferencesArgs {
    file: String,
    line: u32,
    character: u32,
    #[serde(rename = "includeDeclaration", default = "default_true")]
    include_declaration: bool,
}

fn default_true() -> bool {
    true
}

async fn handle_find_references(args: Value, lsp_manager: Arc<LspManager>) -> CallToolResult {
    let args: FindReferencesArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    let file_path = PathBuf::from(&args.file);

    match lsp_manager
        .find_references(
            &file_path,
            args.line,
            args.character,
            args.include_declaration,
        )
        .await
    {
        Ok(Some(locations)) => {
            let text = format_locations(locations);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Ok(None) => CallToolResult {
            content: vec![ToolContent::Text {
                text: "No references found".to_string(),
            }],
            is_error: None,
        },
        Err(e) => {
            error!("find_references error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct HoverArgs {
    file: String,
    line: u32,
    character: u32,
}

async fn handle_hover(args: Value, lsp_manager: Arc<LspManager>) -> CallToolResult {
    let args: HoverArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    let file_path = PathBuf::from(&args.file);

    match lsp_manager
        .hover(&file_path, args.line, args.character)
        .await
    {
        Ok(Some(hover)) => {
            let text = format_hover(hover);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Ok(None) => CallToolResult {
            content: vec![ToolContent::Text {
                text: "No hover information available".to_string(),
            }],
            is_error: None,
        },
        Err(e) => {
            error!("hover error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct DocumentSymbolsArgs {
    file: String,
}

async fn handle_document_symbols(args: Value, lsp_manager: Arc<LspManager>) -> CallToolResult {
    let args: DocumentSymbolsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    let file_path = PathBuf::from(&args.file);

    match lsp_manager.document_symbols(&file_path).await {
        Ok(Some(response)) => {
            let text = format_document_symbols(response);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Ok(None) => CallToolResult {
            content: vec![ToolContent::Text {
                text: "No symbols found".to_string(),
            }],
            is_error: None,
        },
        Err(e) => {
            error!("document_symbols error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct DiagnosticsArgs {
    file: String,
}

async fn handle_diagnostics(args: Value, lsp_manager: Arc<LspManager>) -> CallToolResult {
    let args: DiagnosticsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    let file_path = PathBuf::from(&args.file);

    match lsp_manager.get_diagnostics(&file_path).await {
        Ok(diagnostics) => {
            let text = format_diagnostics(diagnostics);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Err(e) => {
            error!("get_diagnostics error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct WorkspaceSymbolsArgs {
    query: String,
    language: String,
}

async fn handle_workspace_symbols(args: Value, lsp_manager: Arc<LspManager>) -> CallToolResult {
    let args: WorkspaceSymbolsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => {
            return CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid arguments: {}", e),
                }],
                is_error: Some(true),
            };
        }
    };

    match lsp_manager.workspace_symbols(args.query.clone(), &args.language).await {
        Ok(Some(symbols)) => {
            let text = format_workspace_symbols(symbols, &args.query);
            CallToolResult {
                content: vec![ToolContent::Text { text }],
                is_error: None,
            }
        }
        Ok(None) => CallToolResult {
            content: vec![ToolContent::Text {
                text: format!("No symbols found for query: {}", args.query),
            }],
            is_error: None,
        },
        Err(e) => {
            error!("workspace_symbols error: {}", e);
            CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            }
        }
    }
}

// Formatting helpers

fn format_definition_response(response: GotoDefinitionResponse) -> String {
    match response {
        GotoDefinitionResponse::Scalar(location) => format_location(&location),
        GotoDefinitionResponse::Array(locations) => {
            if locations.is_empty() {
                "No definitions found".to_string()
            } else {
                locations
                    .iter()
                    .map(format_location)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        GotoDefinitionResponse::Link(links) => {
            if links.is_empty() {
                "No definitions found".to_string()
            } else {
                links
                    .iter()
                    .map(|link| {
                        format!(
                            "{}:{}:{}",
                            link.target_uri,
                            link.target_range.start.line + 1,
                            link.target_range.start.character + 1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

fn format_location(location: &Location) -> String {
    format!(
        "{}:{}:{}",
        location.uri.path(),
        location.range.start.line + 1,
        location.range.start.character + 1
    )
}

fn format_locations(locations: Vec<Location>) -> String {
    if locations.is_empty() {
        return "No references found".to_string();
    }

    let count = locations.len();
    let formatted = locations
        .iter()
        .map(format_location)
        .collect::<Vec<_>>()
        .join("\n");

    format!("Found {} reference(s):\n{}", count, formatted)
}

fn format_hover(hover: Hover) -> String {
    match hover.contents {
        HoverContents::Scalar(content) => format_markup_content(content),
        HoverContents::Array(contents) => contents
            .into_iter()
            .map(format_markup_content)
            .collect::<Vec<_>>()
            .join("\n\n"),
        HoverContents::Markup(content) => content.value,
    }
}

fn format_markup_content(content: MarkedString) -> String {
    match content {
        MarkedString::String(s) => s,
        MarkedString::LanguageString(ls) => {
            format!("```{}\n{}\n```", ls.language, ls.value)
        }
    }
}

fn format_document_symbols(response: DocumentSymbolResponse) -> String {
    match response {
        DocumentSymbolResponse::Flat(symbols) => {
            if symbols.is_empty() {
                return "No symbols found".to_string();
            }

            let mut output = format!("Found {} symbol(s):\n\n", symbols.len());
            for symbol in symbols {
                output.push_str(&format!(
                    "- {} ({:?}) at {}:{}\n",
                    symbol.name,
                    symbol.kind,
                    symbol.location.range.start.line + 1,
                    symbol.location.range.start.character + 1
                ));
            }
            output
        }
        DocumentSymbolResponse::Nested(symbols) => {
            if symbols.is_empty() {
                return "No symbols found".to_string();
            }

            let mut output = String::from("Document outline:\n\n");
            for symbol in symbols {
                format_document_symbol(&symbol, 0, &mut output);
            }
            output
        }
    }
}

fn format_document_symbol(symbol: &DocumentSymbol, indent: usize, output: &mut String) {
    let indent_str = "  ".repeat(indent);
    output.push_str(&format!(
        "{}- {} ({:?}) at {}:{}\n",
        indent_str,
        symbol.name,
        symbol.kind,
        symbol.selection_range.start.line + 1,
        symbol.selection_range.start.character + 1
    ));

    if let Some(children) = &symbol.children {
        for child in children {
            format_document_symbol(child, indent + 1, output);
        }
    }
}

fn format_diagnostics(diagnostics: Vec<Diagnostic>) -> String {
    if diagnostics.is_empty() {
        return "No diagnostics found (no errors or warnings)".to_string();
    }

    let mut errors = 0;
    let mut warnings = 0;
    let mut infos = 0;
    let mut hints = 0;

    for diagnostic in &diagnostics {
        match diagnostic.severity {
            Some(DiagnosticSeverity::ERROR) => errors += 1,
            Some(DiagnosticSeverity::WARNING) => warnings += 1,
            Some(DiagnosticSeverity::INFORMATION) => infos += 1,
            Some(DiagnosticSeverity::HINT) => hints += 1,
            None | Some(_) => {}
        }
    }

    let mut output = format!(
        "Found {} diagnostic(s): {} error(s), {} warning(s), {} info(s), {} hint(s)\n\n",
        diagnostics.len(),
        errors,
        warnings,
        infos,
        hints
    );

    for diagnostic in &diagnostics {
        let severity = match diagnostic.severity {
            Some(DiagnosticSeverity::ERROR) => "ERROR",
            Some(DiagnosticSeverity::WARNING) => "WARNING",
            Some(DiagnosticSeverity::INFORMATION) => "INFO",
            Some(DiagnosticSeverity::HINT) => "HINT",
            None | Some(_) => "UNKNOWN",
        };

        let source = diagnostic
            .source
            .as_ref()
            .map(|s| format!("[{}] ", s))
            .unwrap_or_default();

        output.push_str(&format!(
            "{}{} at line {}:{}-{}:{}: {}\n",
            source,
            severity,
            diagnostic.range.start.line + 1,
            diagnostic.range.start.character + 1,
            diagnostic.range.end.line + 1,
            diagnostic.range.end.character + 1,
            diagnostic.message
        ));

        // Add related information if available
        if let Some(related) = &diagnostic.related_information {
            for info in related {
                output.push_str(&format!(
                    "  Related: {} at {}:{}:{}\n",
                    info.message,
                    info.location.uri.path(),
                    info.location.range.start.line + 1,
                    info.location.range.start.character + 1
                ));
            }
        }

        output.push('\n');
    }

    output
}

fn format_workspace_symbols(symbols: Vec<SymbolInformation>, query: &str) -> String {
    if symbols.is_empty() {
        return format!("No symbols found for query: {}", query);
    }

    let mut output = format!("Found {} symbol(s) matching '{}':\n\n", symbols.len(), query);

    for symbol in symbols {
        let kind_str = format!("{:?}", symbol.kind);
        let location_str = if let Ok(path) = symbol.location.uri.to_file_path() {
            format!(
                "{}:{}:{}",
                path.display(),
                symbol.location.range.start.line + 1,
                symbol.location.range.start.character + 1
            )
        } else {
            format!(
                "{}:{}:{}",
                symbol.location.uri.path(),
                symbol.location.range.start.line + 1,
                symbol.location.range.start.character + 1
            )
        };

        output.push_str(&format!(
            "- {} ({}) at {}\n",
            symbol.name,
            kind_str,
            location_str
        ));

        // Add container name if available (e.g., class or module name)
        if let Some(container) = symbol.container_name {
            output.push_str(&format!("  in: {}\n", container));
        }
    }

    output
}
