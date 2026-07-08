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
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            config.allowUnfree = true;
            inherit system overlays;
          };
        in
        with pkgs;
        {
          default = mkShell rec {
            buildInputs = [
              # Rust
              just
              cargo

              # # misc. libraries
              # openssl
              # pkg-config
              # udev

              # # GUI libs
              # libxkbcommon
              # libxcb
              # libGL
              # fontconfig

              # # wayland libraries
              # wayland

              # # x11 libraries
              # libxcursor
              # libxrandr
              # libxi
              # libx11

              # Nix
              nil # LSP
              alejandra # Formatter
            ];

            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          };
        }
      );
    };
}
