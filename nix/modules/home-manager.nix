args@{ ... }:

let
  isModuleArgs = args ? config && args ? lib && args ? pkgs;
  moduleBody = { self ? null }: { config, lib, pkgs, ... }:
    let
      cfg = config.programs.edged;
      tomlFormat = pkgs.formats.toml { };

      system = pkgs.stdenv.hostPlatform.system or pkgs.system;

      defaultPackage =
        if pkgs ? edged then
          pkgs.edged
        else if self != null && self ? packages.${system}.default then
          self.packages.${system}.default
        else
          pkgs.callPackage ../package.nix { };
    in
    {
      options.programs.edged = {
        enable = lib.mkEnableOption "Edged, a pixel-accurate PowerToys Screen Ruler replica for Wayland";

        package = lib.mkOption {
          type = lib.types.package;
          default = defaultPackage;
          defaultText = lib.literalExpression "pkgs.edged or (inputs.edged.packages.\${pkgs.stdenv.hostPlatform.system}.default)";
          description = "The Edged package to install.";
        };

        settings = lib.mkOption {
          type = tomlFormat.type;
          default = { };
          description = ''
            Configuration written to {file}`$XDG_CONFIG_HOME/edged/config.toml`.
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
              appearance = {
                theme = "dark";
                line_thickness = 1.5;
              };
              units = {
                show_pixel_unit = true;
                secondary_unit = "none";
              };
              clipboard = {
                format = "compact";
              };
            }
          '';
        };
      };

      config = lib.mkIf cfg.enable {
        home.packages = [ cfg.package ];

        xdg.configFile."edged/config.toml" = lib.mkIf (cfg.settings != { }) {
          source = tomlFormat.generate "edged-config.toml" cfg.settings;
        };
      };
    };
in
if isModuleArgs then
  moduleBody { } args
else
  moduleBody args
