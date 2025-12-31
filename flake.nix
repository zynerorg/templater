{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
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
  };

  outputs =
    {
      nixpkgs,
      systems,
      fenix,
      naersk,
      ...
    }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
      toolchain =
        system:
        with fenix.packages.${system};
        with default;
        combine [
          cargo
          rustc
          clippy
          rustfmt
        ];
    in
    {
      packages = eachSystem (system: {
        default =
          let
            name = "templater";
            toolchain' = toolchain system;
            naersk' = naersk.lib.${system}.override {
              cargo = toolchain';
              rustc = toolchain';
            };
          in
          naersk'.buildPackage {
            inherit name;
            src = ./.;
            meta.mainProgram = name;
          };
      });
      devShells = eachSystem (system: {
        default =
          with nixpkgs.legacyPackages.${system};
          mkShell {
            buildInputs = with pkgs; [
              (toolchain system)
              rust-analyzer
            ];
          };
      });
    };
}
