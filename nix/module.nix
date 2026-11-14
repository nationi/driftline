flake:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.driftline;
in
{
  options.programs.driftline = {
    enable = lib.mkEnableOption "Driftline Wayland compositor";
    package = lib.mkOption {
      type = lib.types.package;
      default = flake.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "Driftline package to use for the session.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.displayManager.sessionPackages = [ cfg.package ];
    systemd.packages = [ cfg.package ];
    systemd.user.services.driftline = {
      restartIfChanged = false;
      enableDefaultPath = false;
    };
    hardware.graphics.enable = true;
    security.polkit.enable = true;
    security.pam.services.swaylock = { };
    services.graphical-desktop.enable = true;
    programs.dconf.enable = lib.mkDefault true;
    services.xserver.desktopManager.runXdgAutostartIfNone = lib.mkDefault true;
    xdg.portal = {
      enable = true;
      wlr.enable = true;
      extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
      configPackages = [ cfg.package ];
    };
  };
}
