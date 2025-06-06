{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      derivation = pkgs:
        let naersk' = pkgs.callPackage naersk { };
        in
        naersk'.buildPackage {
          src = ./.;
        };

    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };

        in
        {
          # For `nix build` & `nix run`:
          defaultPackage = derivation pkgs;

          # For `nix develop`:
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [
                cargo
                cargo-edit
                rust-analyzer
                rustc
                rustfmt
              ];
          };
        }
      ) // {
      overlay = final: prev:
        {
          advent-of-code-2023 = derivation final;
        };
    };
}
