{
  description = "Development environment for the Som programming language";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable."1.90.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cargo-llvm-cov
            just
            cargo-edit
            cargo-watch
            cargo-deny
            llvm
            ripgrep
            fd
            bat
          ];

          shellHook = ''
            export PATH="$HOME/.cargo/bin:$PATH"

            # Install committer if not already installed
            if ! command -v committer &> /dev/null; then
              echo "Installing committer..."
              cargo install committer
            fi
          '';

          RUST_BACKTRACE = "1";
          RUST_LOG = "debug";
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
