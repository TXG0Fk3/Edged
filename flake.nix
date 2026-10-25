{
  description = "Pixel-accurate PowerToys Screen Ruler replica natively designed for Linux Wayland desktops";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          edged = pkgs.callPackage ./nix/package.nix { };
        in
        {
          default = edged;
          edged = edged;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/edged";
          meta.description = "Launch Edged screen ruler";
        };
        edged = self.apps.${system}.default;
      });

      overlays.default = final: prev: {
        edged = final.callPackage ./nix/package.nix { };
      };

      nixosModules = {
        default = import ./nix/modules/nixos.nix { inherit self; };
        edged = self.nixosModules.default;
      };

      homeManagerModules = {
        default = import ./nix/modules/home-manager.nix { inherit self; };
        edged = self.homeManagerModules.default;
      };

      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              pkg-config
              wayland
              libxkbcommon
              libGL
              vulkan-loader
              grim
              wl-clipboard
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
              wayland
              libxkbcommon
              libGL
              vulkan-loader
            ]);
          };
        }
      );
    };
}
