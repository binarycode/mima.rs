{
  description = "mima.rs";

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
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlay ];
      };
    in {
      defaultPackage = pkgs.rustPlatform.buildRustPackage {
        pname = "mima-rs";
        version = "0.1.0";
        src = ./.;
        nativeBuildInputs = [ pkgs.rust-bin.nightly.latest.default ];
        doCheck = false; # FIXME requires a lot of packages for some reason
        cargoSha256 = "sha256-jh9LYZPAjC6vkcM7bkBgq8l7pGKErhCyVR27P1G60dI=";
      };
      devShell = pkgs.mkShell {
        name = "mima-rs";
        buildInputs = [ pkgs.rust-bin.nightly.latest.default ];
      };
    }
  );
}
