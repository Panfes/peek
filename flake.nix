{
  description = "Rust development shell, using the fenix overlay for nightly version of the software";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, fenix, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };

      toolchain = fenix.packages.${system}.stable.completeToolchain;
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          toolchain
          pkgs.gcc
          pkgs.pkg-config
          pkgs.python3
        ];

        shellHook = ''
          export RUST_SRC_PATH="${toolchain}/lib/rustlib/src/rust/library"
          echo "Rust development shell, using the fenix overlay for nightly version of the soft"
        '';
      };
    };
}
