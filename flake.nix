{
  description = "ZFS Prometheus Exporter - Monitor ZFS pools, datasets, ARC, L2ARC, and vdevs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # Overlay to add our package to nixpkgs (Linux only)
      overlay = final: prev:
        if prev.stdenv.isLinux then {
          zfs-prometheus-exporter = self.packages.${final.system}.default;
        } else { };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Build inputs for the Rust application
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        # Cargo.toml parsing for version
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.package.version;

      in
      {
        packages = pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "zfs-prometheus-exporter";
            inherit version;

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            inherit nativeBuildInputs buildInputs;

            # Tests require ZFS which may not be available in build sandbox
            doCheck = false;

            meta = with pkgs.lib; {
              description = "Prometheus exporter for ZFS metrics";
              longDescription = ''
                A high-performance Prometheus exporter for ZFS metrics written in Rust.
                Collects comprehensive metrics from ZFS pools, datasets, vdevs, ARC,
                L2ARC, special devices, and resilvering operations.
              '';
              homepage = "https://github.com/yourusername/zfs-prometheus-exporter";
              license = licenses.asl20;
              maintainers = [ ];
              platforms = platforms.linux;
            };
          };
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          inputsFrom = pkgs.lib.optionals pkgs.stdenv.isLinux [ self.packages.${system}.default ];

          packages = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
            clippy
            rustfmt
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            zfs
          ];

          shellHook = ''
            echo "ZFS Prometheus Exporter Development Shell"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build          - Build debug binary"
            echo "  cargo build --release - Build release binary"
            echo "  cargo test           - Run tests"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo "  cargo watch -x run   - Auto-rebuild on changes"
            echo ""
            echo "Note: Running the exporter requires root for ZFS commands"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_LOG = "debug";
        };

        # Formatter for `nix fmt`
        formatter = pkgs.nixpkgs-fmt;
      }
      // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
        # Apps for `nix run` (Linux only)
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    ) // {
      # NixOS module
      nixosModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.zfs-prometheus-exporter;
        in
        {
          options.services.zfs-prometheus-exporter = {
            enable = mkEnableOption "ZFS Prometheus Exporter";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              defaultText = literalExpression "pkgs.zfs-prometheus-exporter";
              description = "The zfs-prometheus-exporter package to use";
            };

            port = mkOption {
              type = types.port;
              default = 9134;
              description = "Port to listen on";
            };

            host = mkOption {
              type = types.str;
              default = "0.0.0.0";
              description = "Host address to bind to";
              example = "127.0.0.1";
            };

            openFirewall = mkOption {
              type = types.bool;
              default = false;
              description = "Open the firewall for the exporter port";
            };

            extraFlags = mkOption {
              type = types.listOf types.str;
              default = [ ];
              description = "Extra command-line flags to pass to the exporter";
              example = [ "--verbose" ];
            };

            environmentFile = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = "Environment file to load (for secrets, etc.)";
              example = "/run/secrets/zfs-exporter-env";
            };
          };

          config = mkIf cfg.enable {
            # Ensure ZFS is available
            assertions = [
              {
                assertion = config.boot.supportedFilesystems.zfs or false;
                message = "ZFS must be enabled for zfs-prometheus-exporter to work";
              }
            ];

            systemd.services.zfs-prometheus-exporter = {
              description = "ZFS Prometheus Exporter";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" "zfs.target" ];
              requires = [ "zfs.target" ];

              serviceConfig = {
                Type = "simple";
                ExecStart = "${cfg.package}/bin/zfs-prometheus-exporter --port ${toString cfg.port} --host ${cfg.host} ${lib.concatStringsSep " " cfg.extraFlags}";
                Restart = "always";
                RestartSec = "10s";
                
                # Security hardening
                DynamicUser = false; # Need root for ZFS commands
                User = "root";
                Group = "root";
                
                # Hardening options
                NoNewPrivileges = true;
                PrivateTmp = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadOnlyPaths = "/";
                ReadWritePaths = [ "/proc/spl" ];
                
                # Resource limits
                MemoryMax = "256M";
                TasksMax = 16;
                
                # Logging
                StandardOutput = "journal";
                StandardError = "journal";
                SyslogIdentifier = "zfs-exporter";
              } // (if cfg.environmentFile != null then {
                EnvironmentFile = cfg.environmentFile;
              } else { });

              environment = {
                RUST_LOG = mkDefault "info";
              };
            };

            # Open firewall if requested
            networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];
          };
        };

      # Overlay for adding to nixpkgs
      overlays.default = overlay;
    };
}
