<div align="center">
<img src="assets/driftline-logo.svg" alt="Driftline" width="360">
<h1>Driftline</h1>
<p><strong>Infinite canvas first. Scrolling-column tiling when you need it.</strong></p>
<p>A trackpad-first Wayland compositor with independent, practical tiling layouts on every output.</p>
<p>
  <a href="https://github.com/nationi/driftline/releases"><img alt="GitHub releases, including prereleases" src="https://img.shields.io/github/v/release/nationi/driftline?include_prereleases&amp;label=release"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg"></a>
</p>
<p><strong>This fork:</strong> <a href="https://github.com/nationi/driftline">nationi/driftline</a> ·
<strong>Upstream:</strong> <a href="https://github.com/malbiruk/driftwm">malbiruk/driftwm</a></p>
</div>

---

`driftline` treats the desktop as an infinite 2D canvas. Windows keep their natural size; you pan and zoom the view, and nearby windows snap into groups. When you want a more structured layout, switch to optional scrolling-column tiling. Every connected monitor gets its own layout, while the canvas remains available as the default way to work.

This is an independently maintained fork of [driftwm](https://github.com/malbiruk/driftwm), focused on usable multi-monitor tiling alongside the original canvas experience. It takes inspiration from [niri](https://github.com/niri-wm/niri), without aiming for feature parity.

## See driftline in action

The original upstream demo below shows the canvas workflow; it does not demonstrate this fork's tiling mode. Tiling controls and current limitations are described in the [tiling guide](docs/tiling.md).

https://github.com/user-attachments/assets/155511a6-0a6e-4681-9061-21be1e93e02a

## What makes this fork different

- **Independent tiling per monitor.** Each output has its own scrolling strip of columns, vertical window stacks, selection and camera.
- **Arrange windows with the mouse.** `Alt`+left-drag swaps windows, inserts them into stacks, places columns beside a target, or pulls a window into a new column. Drop markers and action labels preview the result; holding a window near a horizontal screen edge scrolls toward neighboring columns.
- **Move between monitors.** Use `Mod+Alt+Arrow` or continue an `Alt`+left-drag onto another monitor to transfer a tiled window.
- **Resize columns directly.** `Alt`+right-drag changes the column width, respecting client minimum sizes. The pointer stays on its starting monitor; slow edge scrolling lets a resize continue at the screen boundary. Keyboard width presets are also available.
- **Keep fullscreen local.** Fullscreen covers its output while tiling stays active on the others. Exiting fullscreen returns the window to its tile.
- **Switch modes explicitly.** `Mod+Ctrl+T` gathers windows into columns or returns them to their saved canvas positions, with animated transitions. Mode-specific bindings keep canvas operations from disrupting tiling.
- **Restore the arrangement.** Column order, stacks, widths, selection and output ownership are saved with restorable session windows.

With `[session] restore_windows = true` (the default), eligible apps saved as still open are relaunched on the next compositor start. They need a launchable desktop entry and a window Driftline can match. Explicitly suspended apps remain suspended. Driftline starts in canvas mode; enter tiling to rebuild the saved arrangement. See [session restore](docs/session.md) for details.

The [tiling guide](docs/tiling.md) covers drop zones, hover focus, monitor hotplug and limits. The layout deliberately has no workspaces, tabs or general floating-window layer.

## The canvas is still the foundation

- Pan, zoom and navigate a large 2D workspace with trackpad gestures, keyboard or mouse.
- Snap nearby windows into clusters that can move and resize together.
- Use named camera bookmarks, window rules, shader backgrounds and session restore.
- Run standard Wayland panels and tools through layer-shell, with optional X11 support through `xwayland-satellite`.

Explore the guides below for feature details and configuration.

## Quick start

### Build

This README describes `main`, including changes made after the latest published version. [v1.1.5](https://github.com/nationi/driftline/releases/tag/v1.1.5) is currently a **prerelease**. Build `main` for the behavior documented here.

Use a current stable Rust toolchain. The manifest declares Rust 1.88 as the minimum and uses the 2024 edition. Install a C toolchain, `pkg-config`, Git and the system libraries for Smithay's input and graphics backends:

**Fedora**

```bash
sudo dnf install git gcc make pkgconf-pkg-config libseat-devel libdisplay-info-devel libinput-devel systemd-devel mesa-libgbm-devel libxkbcommon-devel wayland-devel
```

**Ubuntu / Debian**

```bash
sudo apt install git build-essential pkg-config libseat-dev libdisplay-info-dev libinput-dev libudev-dev libgbm-dev libxkbcommon-dev libwayland-dev
```

**Arch Linux**

```bash
sudo pacman -S --needed git base-devel libdisplay-info libinput seatd mesa libxkbcommon
```

Install Rust with [rustup](https://rustup.rs/) if needed, then build the fork:

```bash
git clone --branch main --depth 1 https://github.com/nationi/driftline.git
cd driftline
cargo build --release --locked
```

### Install the session

After building, install the binary, display-manager session entry, user service,
portal configuration and shader examples:

```bash
sudo make install
```

The default prefix is `/usr/local`. Installation does not write your personal
config. Log out and select **driftline** in your display manager to start the
installed session. Panels, launchers and lock screens are external tools; an
optional dashboard and example configuration are provided in [extras/](extras/README.md).

### Run

For a quick preview, run nested inside an existing Wayland session:

```bash
./target/release/driftline --backend winit
```

A nested run skips session persistence unless you supply `--session-file PATH`.

Run on real hardware from a TTY:

```bash
./target/release/driftline --backend udev
```

The udev command uses all available outputs. To limit a test session to one connector, add `--drm-connector HDMI-A-1` and replace the name with a connector reported by your system. The [hardware testing guide](dev/docs/testing.md#hardware-acceptance) covers an isolated preview setup.

For X11 applications, install `xwayland-satellite` (0.7 or newer) on `PATH`; Driftline starts it automatically when enabled. Native Wayland apps do not need it.

## First controls

`Mod` is Super/Win by default and can be changed with `mod_key` in the config.

| Keys | Canvas mode | Tiling mode |
| --- | --- | --- |
| `Mod+Return` | Launch a terminal | Launch a terminal |
| `Mod+D` | Open the app launcher | Open the app launcher |
| `Mod+Ctrl+T` | Enter tiling | Return to canvas |
| `Alt+Tab` / `Alt+Shift+Tab` | Cycle windows forward / backward | Cycle windows without leaving tiling |
| `Mod+Arrow` | Center the nearest window or anchor | Focus a neighboring column or stacked window |
| `Mod+Alt+Arrow` | Send the focused window to an adjacent output | Send the focused tile to an adjacent output |
| `Mod+X` | Center the window under the pointer | Focus and center the tile under the pointer |
| `Mod` + left-drag | Pan the camera | Scroll the columns horizontally |
| `Alt` + left-drag | Move a window on the canvas | Swap, stack, place or transfer a window |
| `Alt` + right-drag | Resize a window and its snapped neighbors | Resize the column |
| `Mod+R` | — | Cycle the column width |
| `Mod+F` | Toggle fullscreen | Toggle fullscreen on the window's output |
| `Mod+Q` | Close the focused window | Close the focused window |
| `Mod+Ctrl+Shift+Q` | Exit Driftline | Exit Driftline |

Install a terminal and launcher (for example `foot` and `fuzzel`), or select them with `$TERMINAL` and `$LAUNCHER`. Without overrides, Driftline tries supported programs on `PATH`.

See [all tiling shortcuts](docs/tiling.md#default-tiling-keys) for stack and reorder controls. Canvas-only bindings are ignored while tiling; the shared drag bindings perform the tiling operations described above.

## Configuration

The default config path is `~/.config/driftline/config.toml` (or `$XDG_CONFIG_HOME/driftline/config.toml`). If it does not exist, an existing `driftwm/config.toml` under the same config directory is used. With neither file present, Driftline uses built-in defaults.

`--config PATH` selects an explicit file. Environment overrides use `DRIFTLINE_CONFIG`, with `DRIFTWM_CONFIG` accepted as a fallback. Existing configurations do not need to be renamed. To start a new configuration, create the directory and add only the options you want to change:

```bash
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/driftline"
```

For example, separate the same key from its canvas and tiling behavior:

```toml
[keybindings.canvas]
"mod+left" = "center-nearest left"

[keybindings.tiling]
"mod+left" = "focus-column left"
```

Check a config without starting the compositor:

```bash
./target/release/driftline --config /path/to/config.toml --check-config
```

Config changes are watched and reloaded while running. The [configuration guide](docs/config.md) explains the options; [`config.reference.toml`](config.reference.toml) is the canonical reference.

## Documentation

- [Scrolling tiling](docs/tiling.md) — mode switching, controls, multi-output behavior and limits
- [Configuration](docs/config.md) — generated option reference
- [Window rules](docs/window-rules.md) — per-application placement and appearance
- [IPC and screenshots](docs/ipc.md) — control driftline from scripts
- [Command-line reference](docs/cli.md) — compositor and `driftline msg` commands
- [Session restore](docs/session.md) — app relaunch, saved canvas views and tiling topology
- [Changelog](CHANGELOG.md) — published version highlights
- [Shaders](docs/shaders.md) · [Gigapixel wallpapers](docs/gigapixel-wallpapers.md)
- [Development documentation](dev/docs/README.md)

## Contributing

Contributions to the fork are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or pull request. For tiling work, start with the [development notes](dev/docs/README.md) and [current status](dev/docs/tiling-status.md).

## Credits and license

Driftline is maintained by [nationi](https://github.com/nationi). This project is based on [upstream driftwm](https://github.com/malbiruk/driftwm), originally created by Klim Kostiuk, built on [Smithay](https://github.com/Smithay/smithay), and informed by design ideas from [niri](https://github.com/niri-wm/niri). The fork is maintained independently; packages and releases from upstream do not include fork-specific changes.

Licensed under [GPL-3.0-or-later](LICENSE).
