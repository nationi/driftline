# NixOS

The repository provides a locked flake with a package, a development shell and a
NixOS module. Enable `nix-command` and `flakes` in your Nix configuration first:

```nix
nix.settings.experimental-features = [ "nix-command" "flakes" ];
```

## Install a session

Add Driftline to your system flake and import its module:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    driftline.url = "github:nationi/driftline";
  };

  outputs = { nixpkgs, driftline, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        driftline.nixosModules.default
        { programs.driftline.enable = true; }
      ];
    };
  };
}
```

Rebuild with `sudo nixos-rebuild switch --flake .#my-host`, then log out and
select **driftline** in your display manager. Keep your existing display-manager
configuration; the module registers the session without selecting a new manager
or replacing your default session. From a TTY, run `driftline-session`.

The module installs the package and systemd user units, enables graphics,
logind session access, polkit, dconf and the swaylock PAM service, and configures
GTK and wlr desktop portals. Screen sharing also needs PipeWire; enable
`services.pipewire.enable = true;` in your system configuration if needed.
Install a launcher, terminal, panel and lock screen separately. The package
includes xwayland-satellite for X11 applications.

`programs.driftline.package` can override the compositor package. By default it
uses the package built with this flake's locked nixpkgs. Both `x86_64-linux` and
`aarch64-linux` outputs are provided.

The module does not write your personal config or session state. Configuration
lives at `~/.config/driftline/config.toml` (or under `XDG_CONFIG_HOME`). Copy the
installed `share/driftline/config.reference.toml` as a starting point. See
[configuration](config.md), [tiling](tiling.md) and [session restore](session.md).

## Build and preview

From a checkout:

```bash
nix build
./result/bin/driftline --version
nix run
```

`nix run` starts nested in an existing Wayland session. Installing only the
package does not configure NixOS session permissions, portals or user units;
use the module for a full desktop session. The package installs the session
entry, icon, portal preferences, systemd units, reference config and shaders.
Its wrapper supplies libraries loaded at runtime from the Nix store.

## Development

```bash
nix develop
cargo build --locked
cargo run
# Match the project's Rust CI checks:
dev/scripts/ci-local
```

The shell includes Rust, native build dependencies, runtime graphics libraries
and foot for real-client tests. On NixOS, run hardware previews only after
enabling the module and rebuilding; see the [hardware testing guide](../dev/docs/testing.md#hardware-acceptance).
Use the system's graphics drivers rather than embedding a Mesa driver in the
package. `nix flake update` refreshes the pinned nixpkgs revision; commit the
resulting `flake.lock` after verifying a build.
