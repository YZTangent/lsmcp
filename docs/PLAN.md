# LSMCP Implementation Plan

This document tracks the step-by-step implementation of the Language Server Manager for MCP.

## Phase 1: Foundation

### 1.1 Project Setup
- [x] Initialize Rust project with Cargo
- [x] Set up project structure (src/, docs/, registry/, scripts/)
- [x] Configure `Cargo.toml` with dependencies
- [x] Set up `build.rs` for embedding registry
- [x] Add `.gitignore` for Rust projects
- [x] Initialize basic logging with `tracing`

### 1.2 Configuration System
- [x] Define configuration types (`LspPackage`, `InstallSource`, etc.)
- [x] Implement built-in defaults for TypeScript, Python, Rust, Go
- [x] Create registry file format (TOML)
- [x] Implement registry loader (parse TOML files)
- [x] Implement user config loader (`~/.config/lsmcp.toml`, `.lsmcp.toml`)
- [x] Implement 3-tier precedence system
- [x] Add file extension → language detection
- [x] Write tests for config system

### 1.3 Mason Registry Integration
- [x] Create `scripts/sync-mason-registry.rs`
- [x] Implement Mason YAML → our TOML converter
- [x] Sync initial set of LSPs (rust-analyzer, typescript-language-server, pyright, gopls)
- [x] Add 20+ popular LSPs from Mason
- [x] Embed registry files in binary via `build.rs`
- [x] Document registry update process

### 1.4 LSP Client Implementation
- [x] Implement `LspClient` struct with process management
- [x] Implement JSON-RPC message parsing (LSP protocol)
- [x] Implement LSP client initialization handshake
- [x] Implement request/response handling with futures
- [x] Implement `textDocument/didOpen` notification
- [x] Implement `textDocument/didClose` notification
- [x] Add request timeout handling (30s default)
- [x] Add process crash detection and recovery (via kill_on_drop)
- [ ] Write tests with mock LSP server (deferred to integration testing)

### 1.5 LSP Manager Implementation
- [x] Implement `LspManager` with `Arc<Mutex<HashMap>>`
- [x] Implement lazy LSP client initialization
- [x] Implement workspace root detection (CLI arg, git root, cwd) - in main.rs
- [x] Implement language detection from file extensions
- [x] Implement LSP process lifecycle (spawn, monitor, shutdown)
- [x] Add graceful shutdown for all LSP processes
- [ ] Write tests for manager lifecycle (deferred to integration testing)

### 1.6 MCP Server Integration
- [ ] Set up MCP server with `mcp-rs`
- [ ] Implement MCP tool registration framework
- [ ] Add CLI argument parsing (`--workspace`, `--log-level`)
- [ ] Implement stdio transport for MCP protocol
- [ ] Add structured logging for debugging
- [ ] Test MCP server with simple echo tool

## Phase 2: Core LSP Tools

### 2.1 Navigation Tools
- [ ] Implement `lsp_goto_definition` tool
  - [ ] Parse tool arguments (file, line, character)
  - [ ] Call LSP `textDocument/definition`
  - [ ] Format response for MCP
  - [ ] Add error handling
- [ ] Implement `lsp_find_references` tool
  - [ ] Parse arguments (file, line, character, includeDeclaration)
  - [ ] Call LSP `textDocument/references`
  - [ ] Format response with file paths and line numbers
- [ ] Implement `lsp_hover` tool
  - [ ] Call LSP `textDocument/hover`
  - [ ] Extract documentation and type information
  - [ ] Format as readable markdown

### 2.2 Symbol Tools
- [ ] Implement `lsp_document_symbols` tool
  - [ ] Call LSP `textDocument/documentSymbol`
  - [ ] Format symbol hierarchy (classes, functions, variables)
  - [ ] Include location information
- [ ] Implement `lsp_workspace_symbols` tool
  - [ ] Parse query string
  - [ ] Call LSP `workspace/symbol`
  - [ ] Format results with file locations

### 2.3 Diagnostics Tools
- [ ] Implement `lsp_diagnostics` tool
  - [ ] Call LSP `textDocument/publishDiagnostics`
  - [ ] Format errors, warnings, hints
  - [ ] Include source code context

### 2.4 Testing
- [ ] Create test workspace with TypeScript project
- [ ] Create test workspace with Rust project
- [ ] Write integration tests for each tool
- [ ] Test with real LSP servers

## Phase 3: Advanced Features

### 3.1 Call Hierarchy
- [ ] Implement `lsp_call_hierarchy_incoming` tool
  - [ ] Call LSP `textDocument/prepareCallHierarchy`
  - [ ] Call LSP `callHierarchy/incomingCalls`
  - [ ] Format call tree
- [ ] Implement `lsp_call_hierarchy_outgoing` tool
  - [ ] Call LSP `callHierarchy/outgoingCalls`
  - [ ] Format call tree

### 3.2 Type Hierarchy
- [ ] Implement `lsp_type_hierarchy` tool
  - [ ] Call LSP `textDocument/prepareTypeHierarchy`
  - [ ] Call LSP `typeHierarchy/supertypes` or `subtypes`
  - [ ] Format type tree

### 3.3 Code Actions
- [ ] Implement `lsp_code_actions` tool
  - [ ] Call LSP `textDocument/codeAction`
  - [ ] Format available actions

### 3.4 Completions (Optional)
- [ ] Implement `lsp_completion` tool
  - [ ] Call LSP `textDocument/completion`
  - [ ] Format completion items
- [ ] Implement `lsp_signature_help` tool
  - [ ] Call LSP `textDocument/signatureHelp`
  - [ ] Format signature information

## Phase 4: Workspace Management

### 4.1 Workspace Tools
- [ ] Implement `lsp_workspace_init` tool
  - [ ] Accept workspace path and language list
  - [ ] Initialize LSP servers for specified languages
  - [ ] Return status
- [ ] Implement `lsp_workspace_status` tool
  - [ ] List active LSP servers
  - [ ] Show server status (running, crashed, etc.)
  - [ ] Show capabilities per server

### 4.2 Multi-Workspace Support
- [ ] Support multiple workspace roots
- [ ] Route requests to correct workspace
- [ ] Handle workspace switching

## Phase 5: Polish & Distribution

### 5.1 Error Handling
- [ ] Improve error messages with actionable suggestions
- [ ] Add LSP installation instructions per platform
- [ ] Detect missing LSP binaries and suggest install commands
- [ ] Handle edge cases (invalid paths, unsupported languages, etc.)

### 5.2 Documentation
- [ ] Write comprehensive README.md
  - [ ] Installation instructions
  - [ ] Quick start guide
  - [ ] MCP client configuration examples
- [ ] Write docs/TOOLS.md (tool reference)
- [ ] Write docs/CONFIGURATION.md (config file reference)
- [ ] Write docs/SETUP.md (LSP installation guide)
- [ ] Add inline code documentation
- [ ] Create usage examples

### 5.3 Testing & CI
- [ ] Set up GitHub Actions CI
- [ ] Add unit tests (target: >80% coverage)
- [ ] Add integration tests with real LSPs
- [ ] Add end-to-end tests with MCP client
- [ ] Set up test fixtures

### 5.4 Distribution
- [ ] Set up cross-compilation (Linux, macOS, Windows)
- [ ] Create GitHub release workflow
- [ ] Publish binaries for all platforms
- [ ] Create Homebrew formula (macOS)
- [ ] Publish to crates.io
- [ ] Add `cargo install lsmcp` support

### 5.5 Performance Optimization
- [ ] Profile LSP request handling
- [ ] Optimize JSON-RPC parsing
- [ ] Add response caching (if beneficial)
- [ ] Benchmark against different codebases

## Phase 6: Extended Features (Future)

### 6.1 Auto-Install
- [ ] Implement LSP auto-installer
  - [ ] Detect package manager (npm, pip, cargo, brew, apt)
  - [ ] Install LSP on-demand with user confirmation
  - [ ] Verify installation
- [ ] Add `lsmcp install <lsp-name>` command
- [ ] Add `lsmcp doctor` command (check LSP availability)

### 6.2 Additional Languages
- [ ] Add Java (jdtls)
- [ ] Add C/C++ (clangd)
- [ ] Add Ruby (solargraph)
- [ ] Add PHP (intelephense)
- [ ] Add Lua (lua-language-server)
- [ ] Add Zig (zls)
- [ ] Add Elixir (elixir-ls)
- [ ] Add Haskell (haskell-language-server)
- [ ] Sync 50+ LSPs from Mason registry

### 6.3 Advanced Caching
- [ ] Implement symbol cache
- [ ] Implement file watcher for cache invalidation
- [ ] Persist cache to disk

### 6.4 DAP Integration (Debug Adapter Protocol)
- [ ] Research DAP protocol
- [ ] Implement DAP client wrapper
- [ ] Add debugging tools

## Progress Tracking

### Completed
- [x] Architecture design
- [x] Implementation plan

### In Progress
- [ ] Phase 1: Foundation

### Not Started
- [ ] Phase 2: Core LSP Tools
- [ ] Phase 3: Advanced Features
- [ ] Phase 4: Workspace Management
- [ ] Phase 5: Polish & Distribution
- [ ] Phase 6: Extended Features

## Notes

- Each checkbox should be checked off as the task is completed
- Update this document as new tasks are discovered
- Add notes for any deviations from the plan
- Track blockers and issues in a separate section if needed

## Current Blockers

_None yet_

## Recent Changes

_Track significant changes to the plan here_
