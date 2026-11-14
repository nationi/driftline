{
  description = "Driftline Wayland compositor";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt);

      packages = forAllSystems (system: {
        driftline = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix { };
        default = self.packages.${system}.driftline;
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.driftline ];
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              foot
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
              with pkgs;
              [
                libglvnd
                wayland
                libxkbcommon
                libx11
                libxi
                libxcb
                libgbm
              ]
            );
          };
        }
      );

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          configuration =
            (nixpkgs.lib.nixosSystem {
              inherit system;
              modules = [
                self.nixosModules.default
                {
                  programs.driftline.enable = true;
                  system.stateVersion = "26.05";
                }
              ];
            }).config;
        in
        {
          package = self.packages.${system}.default;
          module =
            assert configuration.hardware.graphics.enable;
            assert configuration.security.polkit.enable;
            assert configuration.xdg.portal.wlr.enable;
            assert builtins.elem self.packages.${system}.default
              configuration.services.displayManager.sessionPackages;
            assert configuration.systemd.user.services.driftline.enableDefaultPath == false;
            pkgs.runCommand "driftline-module-check" { } "touch $out";
        }
      );

      nixosModules.default = import ./nix/module.nix self;
    };
}
