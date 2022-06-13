{
  description = "Virtual environments manager";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem(system:
    let
      cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlay ];
      };

      rust = pkgs.rust-bin.nightly."2022-06-13".default;

      mima = import ./package.nix pkgs rust;
    in {
      defaultPackage = mima;
      devShell = pkgs.mkShell {
        name = cargoTOML.package.name;
        buildInputs = [ rust ];
      };
      nixosModules.default = import ./module.nix mima;
    }
  );
}
