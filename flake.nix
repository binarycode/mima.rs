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

  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem (system: let
    cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);

    pkgs = import inputs.nixpkgs {
      inherit system;
      overlays = [ inputs.rust-overlay.overlay ];
    };

    rust = import ./rust.nix pkgs;
  in {
    devShells.default = pkgs.mkShell {
      name = cargoTOML.package.name;
      buildInputs = [ rust ];
    };
    packages.default = import ./package.nix pkgs;
  }) // {
    nixosModules.default = import ./module.nix inputs;
  };
}
