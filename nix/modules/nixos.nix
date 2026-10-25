{ config, lib, pkgs, ... }:

let
  cfg = config.programs.edged;
  tomlFormat = pkgs.formats.toml { };
in
{
  meta.maintainers = [ ];

  options.programs.edged = {
    enable = lib.mkEnableOption "Edged, a pixel-accurate PowerToys Screen Ruler replica for Wayland";

    package = lib.mkPackageOption pkgs "edged" {
      default = [ "edged" ];
    };

    settings = lib.mkOption {
      type = tomlFormat.type;
      default = { };
      description = ''
        System-wide configuration written to {file}`/etc/xdg/edged/config.toml`.
        See <https://github.com/TXG0Fk3/Edged/blob/master/config.example.toml>
        for all available configuration fields and comments.
      '';
      example = lib.literalExpression ''
        {
          general = {
            default_mode = "spacing";
            close_on_copy = true;
          };
          monitor = {
            target = "focused";
            crop_to_monitor = true;
          };
          capture = {
            method = "auto";
            timeout_seconds = 5;
          };
          ruler = {
            initial_tolerance = 15;
            continuous = true;
            draw_feet = true;
          };
        }
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    environment.etc."xdg/edged/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = tomlFormat.generate "edged-config.toml" cfg.settings;
    };
  };
}
