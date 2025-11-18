Name:           lsmcp
Version:        0.1.0
Release:        1%{?dist}
Summary:        Language Server Manager for Model Context Protocol

License:        MIT OR Apache-2.0
URL:            https://github.com/YZTangent/lsmcp
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc

%description
LSMCP is a bridge between the Model Context Protocol (MCP) and Language Server
Protocol (LSP), enabling CLI-based LLM clients like Claude Code and Gemini CLI
to access rich code intelligence without grep/cat operations.

Features:
- Auto-installation of LSP servers
- Support for 24+ languages
- 6 core MCP tools (goto_definition, find_references, hover, etc.)
- Multi-location LSP discovery

%prep
%autosetup

%build
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/lsmcp %{buildroot}%{_bindir}/lsmcp

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/lsmcp

%changelog
* Sun Nov 17 2024 LSMCP Contributors <lsmcp@example.com> - 0.1.0-1
- Initial package
