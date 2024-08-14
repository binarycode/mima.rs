inputs: { pkgs, ... }: {
  nixpkgs.overlays = [ inputs.rust-overlay.overlays.default ];

  environment.systemPackages = [
    (import ./package.nix pkgs)
  ];
}
