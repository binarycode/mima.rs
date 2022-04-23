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
        version = "0.6.1";
        src = ./.;
        nativeBuildInputs = [ pkgs.rust-bin.nightly.latest.default ];
        doCheck = false; # FIXME requires a lot of packages for some reason
        cargoSha256 = "sha256-AChzdTpFGwGPOtftbDYvUaAw5UChP9T0x92QMpTit+w=";

        # XXX: use this hash when updating versions
        #cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      };
      devShell = pkgs.mkShell {
        name = "mima-rs";
        buildInputs = [ pkgs.rust-bin.nightly.latest.default ];
      };
    }
  );
}
