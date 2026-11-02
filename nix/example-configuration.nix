# Example NixOS configuration for running deadlocked.

{
  config,
  pkgs,
  ...
}:
{
  # Import the flake (adjust the URL/path to your checkout):
  #   nixpkgs.overlays / inputs — see flake.nix usage notes below.
  #
  # In flake-based configs:
  #   inputs.deadlocked.url = "github:Domanffe/deadlocked";  # or path:.
  #   nixosConfigurations.host = nixpkgs.lib.nixosSystem {
  #     modules = [
  #       ./configuration.nix
  #       inputs.deadlocked.nixosModules.default
  #     ];
  #   };

  services.deadlocked = {
    enable = true;
    package = pkgs.deadlocked; # omit if you don't add the overlay
  };

  # give your desktop user access to /dev/uinput
  users.users.yourName.extraGroups = [ "uinput" ];

  # optional: hyprland window rules are applied automatically at runtime
  # via hyprctl (see cheat/src/os/hyprland.rs), no extra config needed.
}
