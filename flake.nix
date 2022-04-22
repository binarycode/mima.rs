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
      rust = pkgs.rust-bin.nightly.latest.default;
    in {
      defaultPackage = pkgs.rustPlatform.buildRustPackage {
        pname = "mima-rs";
        version = "0.7.2";
        src = ./.;
        nativeBuildInputs = [ rust ];
        doCheck = false; # FIXME requires a lot of packages for some reason
        cargoSha256 = "sha256-iTrAbBfp+xFQl7IO+yr0GjyK9LybVnw/xm8CSFS9slU=";

        # XXX: use this hash when updating versions
        #cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      };
      devShell = pkgs.mkShell {
        name = "mima-rs";
        buildInputs = [ rust ];
      };
    }
  );
}
