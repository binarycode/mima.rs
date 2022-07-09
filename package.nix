pkgs: rust: let
  cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in pkgs.rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;

  src = pkgs.nix-gitignore.gitignoreSource [] ./.;

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = [
    pkgs.iproute2
    pkgs.qemu_kvm
    pkgs.procps
    pkgs.socat
    pkgs.which
  ];

  nativeBuildInputs = [ rust ];

  checkInputs = [ pkgs.which ];

  checkPhase = ''
    ${rust}/bin/cargo clippy -- -D warnings
    ${rust}/bin/cargo fmt -- --check
    ${rust}/bin/cargo test
  '';

  meta = {
    inherit (cargoTOML.package) description license;
    homepage = cargoTOML.package.repository;
    maintainers = cargoTOML.package.authors;
    platforms = pkgs.lib.platforms.linux;
  };
}