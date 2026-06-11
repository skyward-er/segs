{
  description = "SEGS devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      overlays = [ (import rust-overlay) ];
      system = "x86_64-linux";
      pkgs = import nixpkgs {
      	config.allowUnfree = true;
        inherit system overlays;
      };

      systems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    with pkgs;
    {
      devShells = forAllSystems (system: {
        default = mkShell rec {
          buildInputs = [
            # Rust
            just
	    cargo

            # misc. libraries
            openssl
            pkg-config
	    udev

            # GUI libs
            libxkbcommon
	    libxcb
            libGL
            fontconfig

            # wayland libraries
            wayland

            # x11 libraries
            libxcursor
            libxrandr
            libxi
            libx11

            # Nix
            nil # LSP
            alejandra # Formatter
	    
	    claude-code
          ];

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
    };
}

