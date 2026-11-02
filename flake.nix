{
  description = "deadlocked - CS2 external cheat/utility for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      # NixOS module: installs the udev rule and kernel module needed
      # for /dev/uinput (virtual mouse/keyboard used by the aimbot).
      #
      # Usage in configuration.nix:
      #   services.deadlocked.enable = true;
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        with lib;
        let
          cfg = config.services.deadlocked;
        in
        {
          options.services.deadlocked = {
            enable = mkEnableOption "deadlocked uinput support (udev rule + kernel module)";

            package = mkPackageOption pkgs "deadlocked" { nullable = true; };

            extraGroups = mkOption {
              type = types.listOf types.str;
              default = [ "uinput" ];
              example = [
                "uinput"
                "input"
              ];
              description = ''
                Groups that must have access to <filename>/dev/uinput</filename>.
                Add your desktop user to these groups manually, e.g.
                <literal>users.users.yourName.extraGroups = [ "uinput" ];</literal>
              '';
            };
          };

          config = mkIf cfg.enable {
            boot.kernelModules = [ "uinput" ];

            services.udev.extraRules = ''
              KERNEL=="uinput", MODE="0660", GROUP="uinput", TAG+="uaccess"
            '';

            users.groups.uinput = { };

            environment.systemPackages = mkIf (cfg.package != null) [ cfg.package ];
          };
        };
    }
    // flake-utils.lib.eachSystem supportedSystems (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        toolchain = pkgs.rust-bin.stable.latest.minimal.override {
          extensions = [
            "rustfmt"
            "clippy"
          ];
        };

        deadlocked = pkgs.stdenv.mkDerivation (finalAttrs: {
          pname = "deadlocked";
          version = "1.4.1";

          src = ./.;

          strictDeps = true;

          nativeBuildInputs = [
            toolchain
            pkgs.pkg-config
            pkgs.makeWrapper
          ];

          buildInputs = [
            # cpal (via rodio) probes alsa via pkg-config on every Linux build
            pkgs.alsa-lib.dev
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            # xkbcommon-dl / x11-dl dlopen libxkbcommon(-x11) at runtime
            pkgs.xorg.libxkbcommon.dev
            pkgs.wayland
            pkgs.libglvnd
            pkgs.vulkan-loader
            pkgs.mesa.drivers
          ];

          # offline vendoring is not set up, keep network on for cargo
          doCheck = false;

          cargoBuildFlags = [
            "--release"
            "--bin"
            "deadlocked"
          ];

          installPhase = ''
            runHook preInstall

            mkdir -p $out/bin
            cp target/release/deadlocked $out/bin/deadlocked
            chmod +x $out/bin/deadlocked

            runHook postInstall
          '';

          postFixup = ''
            makeWrapper $out/bin/.deadlocked-wrapped $out/bin/deadlocked \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
                pkgs.alsa-lib
                pkgs.xorg.libX11
                pkgs.xorg.libxkbcommon
                pkgs.wayland
                pkgs.libglvnd
                pkgs.vulkan-loader
              ]}"
          '';

          meta = with pkgs.lib; {
            description = "CS2 external overlay, aimbot and radar utility for Linux";
            homepage = "https://github.com/Domanffe/deadlocked";
            license = licenses.unfree;
            platforms = platforms.linux;
            mainProgram = "deadlocked";
          };
        });
      in
      {
        packages = {
          inherit deadlocked;
          default = deadlocked;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            pkgs.pkg-config
            pkgs.patchelf
            pkgs.git
            # cpal (via rodio) probes alsa via pkg-config on every Linux build
            pkgs.alsa-lib.dev
            # build-time C dependencies of the x11/wayland/gl crates
            pkgs.xorg.libX11.dev
            pkgs.xorg.libXcursor.dev
            pkgs.xorg.libXi.dev
            pkgs.xorg.libXrandr.dev
            # winit dlopen()s libxkbcommon-x11.so at runtime via xkbcommon-dl
            pkgs.xorg.libxkbcommon.dev
            pkgs.wayland.dev
            pkgs.libglvnd.dev
            pkgs.vulkan-headers
            pkgs.vulkan-loader.dev
            pkgs.mesa.drivers
            # runtime tools
            pkgs.hyprland
            pkgs.gdb
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.alsa-lib
              pkgs.xorg.libX11
              pkgs.xorg.libxkbcommon
              pkgs.wayland
              pkgs.libglvnd
              pkgs.vulkan-loader
            ]}:$LD_LIBRARY_PATH"
          '';
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
