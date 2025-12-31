# LSMCP Architecture

**Language Server Manager for MCP** - A Rust-based MCP server that provides LSP capabilities to CLI LLM clients.

## Vision

Build a Model Context Protocol (MCP) server that acts as a universal bridge between LLM CLI clients (like Claude Code, Gemini CLI) and Language Server Protocol servers, providing rich code intelligence without grep/cat operations.

## High-Level Architecture

```
┌─────────────────┐
│  Claude Code /  │
│   Gemini CLI    │
└────────┬────────┘
         │ MCP Protocol (stdio/SSE)
         │
┌────────▼────────────────────────────┐
│    lsmcp (Rust Binary)              │
│                                     │
│  ┌──────────────────────────────┐   │
│  │  MCP Server (mcp-rs)         │   │
│  │  - Tool handlers             │   │
│  │  - Request routing           │   │
│  └──────────┬───────────────────┘   │
│             │                       │
│  ┌──────────▼───────────────────┐   │
│  │    LSP Manager               │   │
│  │  - Arc<Mutex<HashMap>>       │   │
│  │  - Lazy initialization       │   │
│  │  - Process lifecycle         │   │
│  └──────────┬───────────────────┘   │
│             │                       │
│  ┌──────────▼───────────────────┐   │
│  │  LSP Client Pool             │   │
│  │  - Tokio child processes     │   │
│  │  - JSON-RPC over stdio       │   │
│  │  - Per-language clients      │   │
│  └──────────┬───────────────────┘   │
└─────────────┼───────────────────────┘
              │ LSP Protocol (JSON-RPC)
     ┌────────┴────────┐
     │                 │
┌────▼─────┐    ┌─────▼──────┐
│typescript│    │  rust-     │
│-language-│    │  analyzer  │ ...
│ server   │    │            │
└──────────┘    └────────────┘
```

## Core Components

### 1. MCP Server Layer

**Responsibility**: Handle MCP protocol communication with LLM clients

**Key Features**:
- Expose LSP operations as MCP tools
- Translate between MCP requests and LSP operations
- Manage async request/response lifecycle
- Provide structured error responses

**Technology**: `mcp-rs` SDK

### 2. LSP Manager

**Responsibility**: Lifecycle management of LSP server processes

**Key Features**:
- **Lazy initialization**: Start LSP servers on-demand when a file is accessed
- **Language detection**: Auto-detect from file extensions (.ts → typescript-language-server)
- **Process management**: Spawn, monitor, and gracefully shutdown LSP servers
- **Workspace awareness**: Track workspace roots, handle multi-project scenarios
- **State management**: Maintain "open files" state required by LSPs
- **Thread-safe**: Use `Arc<Mutex<HashMap>>` for concurrent access

**Data Structure**:
```rust
pub struct LspManager {
    // Language ID -> LSP Client
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>,
    workspace_root: PathBuf,
    config: ConfigLoader,
}
```

### 3. LSP Client Wrapper

**Responsibility**: Abstract LSP JSON-RPC protocol complexity

**Key Features**:
- **Protocol abstraction**: Hide JSON-RPC details from tools
- **Request/Response handling**: Async request management with proper error handling
- **Document sync**: Keep LSP servers synchronized with file state
- **Capability negotiation**: Handle different LSP server capabilities
- **Process I/O**: Manage stdin/stdout communication with LSP server

**Implementation**:
```rust
pub struct LspClient {
    language: String,
    process: Child,
    request_id: AtomicU64,
    stdin: ChildStdin,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
}
```

### 4. Configuration System

**Responsibility**: Multi-tier LSP configuration with precedence

**Three-Tier Hierarchy**:

```
┌─────────────────────────────────────┐
│  User Config (~/.config/lsmcp.toml) │  ← Highest Priority
│  - Custom LSP paths                 │
│  - Override defaults                │
│  - Per-project settings             │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Mason Registry (curated subset)    │  ← Medium Priority
│  - 50+ LSP configs                  │
│  - Installation metadata            │
│  - Auto-update capability           │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Built-in Defaults (hardcoded)      │  ← Lowest Priority
│  - TypeScript, Python, Rust, Go     │
│  - Zero config for common langs     │
│  - Embedded in binary               │
└─────────────────────────────────────┘
```

**Configuration Lookup Algorithm**:
1. Check user config for language override
2. Check user config for custom LSP definition
3. Check Mason registry for LSP by file extension/language
4. Check built-in defaults
5. Return error with helpful installation instructions

### 5. Mason Registry Integration

**Inspiration**: Leverage [mason-registry](https://github.com/mason-org/mason-registry) YAML package definitions

**Package Definition Structure** (Mason YAML):
```yaml
name: rust-analyzer
description: "Implementation of LSP for Rust"
homepage: https://github.com/rust-lang/rust-analyzer
licenses: [Apache-2.0, MIT]
languages: [rust]
categories: [LSP]

source:
  id: pkg:github/rust-lang/rust-analyzer
  # Platform-specific assets...

bin:
  rust-analyzer: rust-analyzer
```

**Our TOML Format** (converted):
```toml
name = "rust-analyzer"
description = "Implementation of LSP for Rust"
homepage = "https://github.com/rust-lang/rust-analyzer"
licenses = ["Apache-2.0", "MIT"]
languages = ["rust"]
file_extensions = ["rs"]

[source]
type = "GithubRelease"
repo = "rust-lang/rust-analyzer"
tag = "latest"

[bin]
primary = "rust-analyzer"
lsp_args = []
```

**Registry Sync Process**:
- Script (`scripts/sync-mason-registry.rs`) fetches Mason YAML files
- Converts to our TOML format
- Stores in `registry/*.toml`
- Embedded into binary at compile time via `build.rs`
- Periodic manual updates (or CI automation)

## MCP Tools Exposed

### Core Navigation

| Tool | Description | LSP Method |
|------|-------------|------------|
| `lsp_goto_definition` | Find where symbol is defined | `textDocument/definition` |
| `lsp_find_references` | All symbol usages | `textDocument/references` |
| `lsp_hover` | Documentation, type info, signatures | `textDocument/hover` |

### Code Structure

| Tool | Description | LSP Method |
|------|-------------|------------|
| `lsp_document_symbols` | File outline (classes, functions, vars) | `textDocument/documentSymbol` |
| `lsp_workspace_symbols` | Search symbols across workspace | `workspace/symbol` |
| `lsp_call_hierarchy_incoming` | Who calls this function? | `callHierarchy/incomingCalls` |
| `lsp_call_hierarchy_outgoing` | What does this function call? | `callHierarchy/outgoingCalls` |
| `lsp_type_hierarchy` | Supertypes/subtypes | `typeHierarchy/supertypes` |

### Code Quality

| Tool | Description | LSP Method |
|------|-------------|------------|
| `lsp_diagnostics` | Get errors, warnings, hints | `textDocument/publishDiagnostics` |
| `lsp_code_actions` | Available fixes/refactorings | `textDocument/codeAction` |

### IntelliSense

| Tool | Description | LSP Method |
|------|-------------|------------|
| `lsp_completion` | Code completions | `textDocument/completion` |
| `lsp_signature_help` | Function parameter info | `textDocument/signatureHelp` |

### Workspace Management

| Tool | Description | Notes |
|------|-------------|-------|
| `lsp_workspace_init` | Initialize workspace with languages | Custom |
| `lsp_workspace_status` | Show active LSP servers | Custom |

## Key Design Decisions

### 1. Lazy LSP Initialization

**Decision**: Don't start all LSPs at server startup; initialize on first request

**Rationale**:
- Fast startup time
- Lower resource usage for unused languages
- Scales to many languages without overhead

**Trade-off**: Slight delay on first request for a language

### 2. Workspace Management Strategy

**Decision**: Accept workspace root as CLI argument, with auto-detection fallback

**Options**:
- CLI argument: `--workspace /path/to/project`
- Git root detection: Find nearest `.git` directory
- MCP tool: `lsp_workspace_init` for dynamic switching

**Rationale**: Flexibility for different workflows

### 3. Document Synchronization

**Decision**: Use "textDocument/didOpen" when file is first accessed, rely on LSP reading from disk

**Rationale**:
- Lower memory usage
- Simpler implementation
- CLI use case: files change less frequently than in IDE

**Trade-off**: LSP sees disk state, not in-memory editor state (acceptable for CLI)

### 4. Error Handling Philosophy

**Strategies**:
- **LSP unavailable**: Return helpful error with install instructions
- **LSP crash**: Auto-restart once, then fail gracefully
- **Request timeout**: 30s default, configurable

**Goal**: Robust user experience with actionable error messages

### 5. Configuration Approach

**Decision**: Sensible defaults + optional user config

**Levels**:
- Built-in defaults: Top 4 languages (TS, Python, Rust, Go)
- Mason registry: 50+ LSPs embedded in binary
- User config: `.lsmcp.toml` for custom overrides

**Rationale**: Works out-of-box, customizable when needed

### 6. Technology Stack

**Language**: Rust
- Performance: Fast startup, low overhead
- Safety: Memory-safe process management
- Concurrency: Tokio async runtime
- Distribution: Single binary, no runtime dependencies

**Dependencies**:
- `mcp-rs`: Official Rust MCP SDK
- `lsp-types`: LSP type definitions
- `tokio`: Async runtime
- `serde`: Serialization
- `anyhow`/`thiserror`: Error handling

## Default Language Support

| Language | LSP Server | Install Command | Priority |
|----------|-----------|-----------------|----------|
| JavaScript/TypeScript | typescript-language-server | `npm i -g typescript-language-server typescript` | High |
| Python | pyright / pylsp | `npm i -g pyright` or `pip install python-lsp-server` | High |
| Rust | rust-analyzer | `rustup component add rust-analyzer` | High |
| Go | gopls | `go install golang.org/x/tools/gopls@latest` | High |
| Java | jdtls | Eclipse JDT LS | Medium |
| C/C++ | clangd | `apt install clangd` / `brew install llvm` | Medium |
| Ruby | solargraph | `gem install solargraph` | Medium |
| PHP | intelephense | `npm i -g intelephense` | Medium |

## Performance Expectations

With Rust implementation:

- **Startup time**: < 50ms (vs ~200ms for Node.js equivalent)
- **Memory per LSP client**: ~5-10MB overhead (vs ~50MB for Node.js)
- **Request latency**: < 1ms overhead (LSP server is the bottleneck)
- **Binary size**: ~5-10MB (statically linked)
- **LSP request response**: < 2s for typical codebases

## Security Considerations

1. **Process isolation**: Each LSP runs in separate process
2. **Input validation**: Validate all file paths, reject path traversal
3. **Resource limits**: Timeout LSP requests (30s default)
4. **Safe process spawning**: Use Tokio's secure process APIs
5. **No arbitrary code execution**: Only run configured LSP binaries

## Future Enhancements

### Phase 1 Additions
- **Smart caching**: Cache symbol locations, invalidate on file changes
- **Multi-workspace**: Support multiple projects simultaneously
- **LSP health checks**: Detect crashed/hung LSP servers

### Phase 2 Additions
- **LSP auto-install**: Detect and offer to install missing LSPs
- **Incremental sync**: For better performance with frequently changing files
- **Custom queries**: Domain-specific tools (e.g., "find all React components")

### Phase 3 Additions
- **Semantic search**: Combine LSP symbols with vector search
- **LSP multiplexing**: Multiple clients sharing same LSP instance
- **DAP integration**: Debug Adapter Protocol support

## Success Metrics

1. **Functionality**: Core LSP features working for TypeScript, Python, Rust, Go
2. **Performance**: LSP requests complete in < 2s for typical codebases
3. **Reliability**: Graceful handling of missing/crashed LSPs
4. **Usability**: Works out-of-box with default LSP installations
5. **Documentation**: Clear setup guide and tool reference

## References

- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Mason Registry](https://github.com/mason-org/mason-registry)
- [rust-analyzer](https://rust-analyzer.github.io/)
- [lsp-types crate](https://docs.rs/lsp-types/)
