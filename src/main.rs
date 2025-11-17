use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

/// Language Server Manager for Model Context Protocol
///
/// Provides LSP capabilities to CLI LLM clients like Claude Code and Gemini CLI.
#[derive(Parser, Debug)]
#[command(name = "lsmcp")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Workspace root directory
    ///
    /// If not specified, attempts to auto-detect from:
    /// 1. Current directory's git root
    /// 2. Current working directory
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Log to file instead of stderr
    #[arg(long)]
    log_file: Option<PathBuf>,
}

fn setup_logging(log_level: &str, log_file: Option<PathBuf>) -> Result<()> {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(level.into());

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    if let Some(log_path) = log_file {
        let file = std::fs::File::create(log_path)?;
        subscriber.with_writer(file).init();
    } else {
        subscriber.with_writer(std::io::stderr).init();
    }

    Ok(())
}

fn detect_workspace_root(provided: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = provided {
        return Ok(path.canonicalize()?);
    }

    // Try to find git root
    let current_dir = std::env::current_dir()?;
    let mut dir = current_dir.as_path();

    loop {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            info!("Detected git root: {}", dir.display());
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Fall back to current directory
    info!("Using current directory as workspace root");
    Ok(current_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(&args.log_level, args.log_file)?;

    info!("Starting LSMCP v{}", env!("CARGO_PKG_VERSION"));

    // Detect workspace root
    let workspace_root = detect_workspace_root(args.workspace)?;
    info!("Workspace root: {}", workspace_root.display());

    // Initialize configuration loader
    let config = match lsmcp::ConfigLoader::new() {
        Ok(config) => std::sync::Arc::new(config),
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    info!("Configuration loaded successfully");

    // Initialize LSP manager
    let lsp_manager = match lsmcp::LspManager::new(workspace_root, config) {
        Ok(manager) => std::sync::Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to create LSP manager: {}", e);
            return Err(e.into());
        }
    };

    info!("LSP manager initialized");

    // Create MCP server
    let mcp_server = lsmcp::McpServer::new(lsp_manager.clone());

    info!("LSMCP server starting - ready to accept MCP requests on stdio");

    // Run MCP server (this blocks until client disconnects)
    match mcp_server.run().await {
        Ok(()) => {
            info!("MCP server stopped normally");
        }
        Err(e) => {
            eprintln!("MCP server error: {}", e);
            return Err(e);
        }
    }

    // Shutdown LSP manager
    lsp_manager.shutdown().await;

    info!("LSMCP shut down successfully");

    Ok(())
}
