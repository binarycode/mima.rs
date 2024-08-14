{
  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixos-24.05;
    rust-overlay = {
      url = github:oxalica/rust-overlay;
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs: let
    cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);

    pkgs = import inputs.nixpkgs {
      system = "x86_64-linux";
      overlays = [ inputs.rust-overlay.overlays.default ];
    };
    pkgs-aarch64-darwin = import inputs.nixpkgs {
      system = "aarch64-darwin";
      overlays = [ inputs.rust-overlay.overlays.default ];
    };

    rust = import ./nix/rust.nix pkgs;
    rust-aarch64-darwin = import ./nix/rust.nix pkgs-aarch64-darwin;
  in {
    devShells.x86_64-linux = rec {
      default = mima;
      mima = pkgs.mkShell {
        name = cargoTOML.package.name;
        buildInputs = [ rust ];
      };
    };
    devShells.aarch64-darwin = rec {
      default = mima;
      mima = pkgs-aarch64-darwin.mkShell {
        name = cargoTOML.package.name;
        buildInputs = [ rust-aarch64-darwin ];
      };
    };

    packages.x86_64-linux = rec {
      default = mima;
      mima = import ./nix/package.nix pkgs;
    };
    packages.aarch64-darwin = rec {
      default = mima;
      mima = import ./nix/package.nix pkgs-aarch64-darwin;
    };

    darwinModules = rec {
      default = mima;
      mima = import ./nix/darwin-module.nix inputs;
    };
    nixosModules = rec {
      default = mima;
      mima = import ./nix/nixos-module.nix inputs;
    };
  };
}
