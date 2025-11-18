{
  description = "Language Server Manager for Model Context Protocol";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        lsmcp = pkgs.rustPlatform.buildRustPackage {
          pname = "lsmcp";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];

          buildInputs = with pkgs; [
            # Add any system dependencies here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          meta = with pkgs.lib; {
            description = "Language Server Manager for Model Context Protocol - bridge between MCP and LSP";
            homepage = "https://github.com/YZTangent/lsmcp";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
            mainProgram = "lsmcp";
          };
        };
      in
      {
        packages = {
          default = lsmcp;
          lsmcp = lsmcp;
        };

        apps.default = {
          type = "app";
          program = "${lsmcp}/bin/lsmcp";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            pkg-config
          ];

          shellHook = ''
            echo "LSMCP development environment"
            echo "Run 'cargo build' to build the project"
          '';
        };
      }
    );
}
