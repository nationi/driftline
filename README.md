<div align="center">
<img src="assets/driftline-logo.svg" alt="Driftline" width="360">
<h1>Driftline</h1>
<p><strong>Infinite canvas first. Scrolling-column tiling when you need it.</strong></p>
<p>A trackpad-first Wayland compositor with independent, practical tiling layouts on every output.</p>
<p>
  <a href="https://github.com/nationi/driftline/tree/v1.1.3"><img alt="Version 1.1.3" src="https://img.shields.io/github/v/tag/nationi/driftline?label=version"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg"></a>
</p>
<p><strong>This fork:</strong> <a href="https://github.com/nationi/driftline">nationi/driftline</a> ·
<strong>Upstream:</strong> <a href="https://github.com/malbiruk/driftwm">malbiruk/driftwm</a></p>
</div>

---

`driftline` treats the desktop as an infinite 2D canvas. Windows keep their natural size; you pan and zoom the view, and nearby windows snap into groups. When you want a more structured layout, switch to optional scrolling-column tiling. Every connected monitor gets its own layout, while the canvas remains available as the default way to work.

This is an independently maintained fork of [driftwm](https://github.com/malbiruk/driftwm), focused on usable multi-monitor tiling alongside the original canvas experience. It takes inspiration from [niri](https://github.com/niri-wm/niri), without aiming for feature parity.

## See driftline in action

The demo below shows the canvas-based workflow. Tiling controls and current limitations are described in the [tiling guide](docs/tiling.md).

https://github.com/user-attachments/assets/155511a6-0a6e-4681-9061-21be1e93e02a

## What makes this fork different

- **Independent tiling per output.** Each monitor has its own horizontal strip of columns, vertical window stacks, focus and scroll position.
- **Move windows between monitors.** `Mod+Alt+Arrow` sends the focused tile to an adjacent output and keeps it tiled.
- **Keep fullscreen local.** A fullscreen window covers its output while tiling remains active on the others; it returns to its column when fullscreen ends.
- **Choose your mode.** The canvas stays the default. `Mod+Ctrl+T` switches between canvas and tiling; canvas-only pointer gestures are isolated while tiling is active.
- **Reorder by dragging.** `Alt`+left-drag swaps tiled windows, with on-screen feedback while dragging.

Tiling order, stacks, widths, selected windows, and output ownership are saved with restorable session windows. After a restart, Driftline still opens in canvas mode; relaunch the restored apps and switch to tiling to restore the saved arrangement. See [Scrolling tiling](docs/tiling.md) for details.

## The canvas is still the foundation

- Pan, zoom and navigate a large 2D workspace with trackpad gestures, keyboard or mouse.
- Snap nearby windows into clusters that can move and resize together.
- Use named camera bookmarks, window rules, shader backgrounds and session restore.
- Run standard Wayland panels and tools through layer-shell, with optional X11 support through `xwayland-satellite`.

Explore the guides below for feature details and configuration.

## Quick start

### Build

The project uses Rust 2024 and requires Rust 1.88 or newer, plus the system development libraries for Smithay's Wayland, input and graphics backends. The latest published release is 1.1.3; the commands below build the current `main`, which includes post-release changes.

**Fedora**

```bash
sudo dnf install libseat-devel libdisplay-info-devel libinput-devel mesa-libgbm-devel libxkbcommon-devel
```

**Ubuntu / Debian**

```bash
sudo apt install libseat-dev libdisplay-info-dev libinput-dev libudev-dev libgbm-dev libxkbcommon-dev libwayland-dev
```

**Arch Linux**

```bash
sudo pacman -S libdisplay-info libinput seatd mesa libxkbcommon
```

Install Rust with [rustup](https://rustup.rs/) if your distribution's packaged compiler is older than 1.88. Then build the fork:

```bash
git clone --branch main --depth 1 https://github.com/nationi/driftline.git
cd driftline
cargo build --release --locked
```

### Run

Run nested inside an existing Wayland session:

```bash
./target/release/driftline --backend winit
```

Run on real hardware from a TTY:

```bash
./target/release/driftline --backend udev
```

The udev command uses all available outputs. To limit a test session to one connector, add `--drm-connector HDMI-A-1` and replace the name with a connector reported by your system. The [TTY guide](dev/docs/tty-preview.md) covers an isolated preview setup.

## First controls

`Mod` is Super/Win by default and can be changed with `mod_key` in the config.

| Keys | Canvas mode | Tiling mode |
| --- | --- | --- |
| `Mod+Ctrl+T` | Enter tiling | Return to canvas |
| `Mod+Arrow` | Center the nearest window or anchor | Focus a neighboring column or stacked window |
| `Mod+Alt+Arrow` | Send the focused window to an adjacent output | Send the focused tile to an adjacent output |
| `Alt` + left-drag | Move a window on the canvas | Swap tiled windows |
| `Mod+F` | Toggle fullscreen | Toggle fullscreen on the focused output |

Tiling also includes column width, stack, reorder and transfer controls. See [all tiling shortcuts](docs/tiling.md#default-tiling-keys). Canvas mouse and gesture bindings that manipulate position or size are ignored in tiling; switch back to canvas to use them.

## Configuration

The config file is `~/.config/driftline/config.toml` (or `$XDG_CONFIG_HOME/driftline/config.toml`). With no file, driftline starts with built-in defaults. Create a partial config to change only the options you need:

Existing `~/.config/driftwm/config.toml` files and `DRIFTWM_CONFIG` overrides
remain supported, so upgrading to Driftline does not require a config migration.

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
./target/release/driftline --check-config
```

The [configuration guide](docs/config.md) explains the options; [`config.reference.toml`](config.reference.toml) is the complete reference.

## Documentation

- [Scrolling tiling](docs/tiling.md) — mode switching, controls, multi-output behavior and limits
- [Configuration](docs/config.md) — generated option reference
- [Window rules](docs/window-rules.md) — per-application placement and appearance
- [IPC and screenshots](docs/ipc.md) — control driftline from scripts
- [Command-line reference](docs/cli.md) — compositor and `driftline msg` commands
- [Session restore](docs/session.md) — save and restore canvas windows and views
- [Shaders](docs/shaders.md) · [Gigapixel wallpapers](docs/gigapixel-wallpapers.md)
- [Development status](dev/docs/tiling-status.md) · [tiling roadmap](dev/docs/scrolling-tiling-roadmap.md)

## Contributing

Contributions to the fork are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or pull request. For tiling work, start with the [development notes](dev/docs/README.md) and [current roadmap](dev/docs/scrolling-tiling-roadmap.md).

## Credits and license

This project is based on [upstream driftwm](https://github.com/malbiruk/driftwm), built on [Smithay](https://github.com/Smithay/smithay), and informed by design ideas from [niri](https://github.com/niri-wm/niri). The fork is maintained independently; packages and releases from upstream do not include fork-specific changes.

Licensed under [GPL-3.0-or-later](LICENSE).
