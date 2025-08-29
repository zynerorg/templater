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

  outputs = { self, nixpkgs, fenix, naersk, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system: 
    let
      name = "templater";
      target = "x86_64-unknown-linux-musl";

      pkgs = nixpkgs.legacyPackages.${system};
      fenix' = fenix.packages.${system};

      toolchain = with fenix'; combine [
        default.toolchain
        targets.${target}.latest.rust-std
      ];
      naersk' = naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      package = { mode ? "build" }:
      naersk'.buildPackage {
        inherit mode name;
        pname = name;
        src = ./.;

        CARGO_BUILD_TARGET = target;
      };
    in {
      packages = rec {
        default = package { };
        test = package { mode = "test"; };
        clippy = package { mode = "clippy"; };

        dockerImage = pkgs.dockerTools.streamLayeredImage {
          inherit name;
          config = {
            Cmd = [ "${default}/bin/templater" ];
          };
        };
      };

      devShells.default = pkgs.mkShell {
        buildInputs = [ toolchain ];
      };
    }
  );
}
