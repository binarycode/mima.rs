inputs: { pkgs, ... }: {
  environment.systemPackages = [
    (import ./package.nix pkgs)
  ];

  nixpkgs.overlays = [ inputs.rust-overlay.overlay ];
}
