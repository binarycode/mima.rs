pkgs: let
  cargoTOML = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  rust = import ./rust.nix pkgs;
in pkgs.rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;

  src = pkgs.nix-gitignore.gitignoreSource [] ../.;

  cargoLock.lockFile = ../Cargo.lock;

  buildInputs = [
    pkgs.which
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    pkgs.iproute2
    pkgs.qemu_kvm
    pkgs.procps
    pkgs.socat
  ];

  nativeBuildInputs = [ rust ];

  checkInputs = [ pkgs.which ];

  checkPhase = ''
    ${rust}/bin/cargo clippy -- -D warnings
    ${rust}/bin/cargo fmt -- --check
  '';

  meta = {
    inherit (cargoTOML.package) description license;
    homepage = cargoTOML.package.repository;
    maintainers = cargoTOML.package.authors;
    platforms = pkgs.lib.platforms.linux ++ pkgs.lib.platforms.darwin;
  };
}
