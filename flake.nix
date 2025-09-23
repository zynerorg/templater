{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs = {
        fenix.follows = "fenix";
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      naersk,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        name = "templater";

        pkgs = nixpkgs.legacyPackages.${system};
        fenix' = fenix.packages.${system};
        toolchain =
          with fenix';
          combine [
            default.cargo
            default.rustc
          ];
        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      {
        packages.default = naersk'.buildPackage {
          inherit name;
          src = ./.;
          meta.mainProgram = name;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            toolchain
            rust-analyzer
          ];
        };
      }
    );
}
