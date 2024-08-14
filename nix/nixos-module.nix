inputs: { config, lib, pkgs, ... }: let
  networks = config.mima.networks;
in {
  options.mima = let
    network = lib.types.submodule ({ name, ... }: {
      options = {
        name = lib.mkOption {
          type = lib.types.str;
          readOnly = true;
          description = ''
            Network name
          '';
        };
        id = lib.mkOption {
          type = lib.types.str;
          description = ''
            Network identifier
          '';
        };
        bridge = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = null;
          description = ''
            Bridge IP address with prefix length (e.g. 192.168.1.1/24)
          '';
        };
      };
      config.name = name;
    });
  in {
    networks = lib.mkOption {
      type = lib.types.attrsOf network;
      default = {};
      description = ''
        Mima networks
      '';
    };
  };

  config = {
    nixpkgs.overlays = [ inputs.rust-overlay.overlays.default ];

    environment.systemPackages = [
      (import ./package.nix pkgs)
      pkgs.iproute2
      pkgs.qemu_kvm
      pkgs.procps
      pkgs.socat
      pkgs.which
    ];

    systemd = {
      network = let
        mapNetworks = f: builtins.listToAttrs (builtins.map f (builtins.attrValues networks));
      in {
        enable = true;

        # For every mima network we need to set up a bridge to put mima vifs into
        # We also need to create dummy interface and put it into the bridge, otherwise the bridge
        # will be in NO-CARRIER state.
        #
        # Configuration for following example settings:
        #   config.mima.networks = {
        #     pub = {
        #       id = "AA";
        #       bridge = "192.168.1.1/24";
        #     };
        #     mgt.id = "BB";
        #   };
        # will look like this:
        #   config.systemd.network = {
        #     netdevs = {
        #       "10-mima-pub" = {
        #         Kind = "bridge";
        #         Name = "mima-pub";
        #         MACAddress = "52:54:00:00:AA:F1";
        #       };
        #       "10-mima-mgt" = {
        #         Kind = "bridge";
        #         Name = "mima-mgt";
        #         MACAddress = "52:54:00:00:BB:F1";
        #       };
        #       "10-mima-pub-root" = {
        #         Kind = "dummy";
        #         Name = "mima-pub-root";
        #         MACAddress = "52:54:00:00:AA:F2";
        #       };
        #       "10-mima-mgt-root" = {
        #         Kind = "dummy";
        #         Name = "mima-mgt-root";
        #         MACAddress = "52:54:00:00:BB:F2";
        #       };
        #     };
        #     networks = {
        #       "10-mima-pub" = {
        #         matchConfig.Name = "mima-pub";
        #         networkConfig = {
        #           Address = "192.168.1.1/24";
        #           IPMasquerade = true;
        #         };
        #       };
        #       "10-mima-mgt".matchConfig.Name = "mima-mgt";
        #       "10-mima-pub-root" = {
        #         matchConfig.Name = "mima-pub-root";
        #         networkConfig.Bridge = "mima-pub";
        #       };
        #       "10-mima-mgt-root" = {
        #         matchConfig.Name = "mima-mgt-root";
        #         networkConfig.Bridge = "mima-mgt";
        #       };
        #     };
        #   };
        netdevs = let
          bridge = network: {
            name = "10-mima-${network.name}";
            value.netdevConfig = {
              Kind = "bridge";
              Name = "mima-${network.name}";
              MACAddress = "52:54:00:00:${network.id}:F1";
            };
          };
          dummy = network: {
            name = "10-mima-${network.name}-root";
            value.netdevConfig = {
              Kind = "dummy";
              Name = "mima-${network.name}-root";
              MACAddress = "52:54:00:00:${network.id}:F2";
            };
          };
        in (mapNetworks bridge) // (mapNetworks dummy);

        networks = let
          bridge = network: {
            name = "10-mima-${network.name}";
            value = {
              matchConfig.Name = "mima-${network.name}";
              networkConfig = lib.mkIf (network.bridge != null) {
                Address = network.bridge;
                IPMasquerade = true;
              };
            };
          };
          dummy = network: {
            name = "10-mima-${network.name}-root";
            value = {
              matchConfig.Name = "mima-${network.name}-root";
              networkConfig.Bridge = "mima-${network.name}";
            };
          };
        in (mapNetworks bridge) // (mapNetworks dummy);
      };
    };
  };
}
