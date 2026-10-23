# Local TTY preview cheat sheet

These commands describe nationi's local isolated preview, not default driftwm
bindings. The multi-monitor launcher builds the current release and uses a
separate config, runtime directory and session file.

For the current source checkpoint and validation, see
[status and remaining work](tiling-status.md) or [the general user guide](../../docs/tiling.md).
The launcher/config are local files outside Git; a fresh clone does not include them.

## Start and return

1. Press **Ctrl+Alt+F3** and log in as `nationi`.
2. Run without sudo. This launcher enables every connected output; do not pass
   a `--drm-connector` argument:

   ```bash
   ~/driftwm-previews/7e32bf5/run-multimon-tty.sh
   ```

3. Press **F2** to enter tiling.

**Ctrl+Shift+F8** quits the preview. **Ctrl+Alt+F2** returns to the main desktop
without quitting it. The launcher compiles the current checkout before starting.

## Keys in this preview

| Key | Action |
| --- | --- |
| F1 | Open a terminal |
| F2 | Switch between canvas and tiling |
| F3 / F4 | Focus the left / right column |
| F5 | Cycle the focused column's width |
| F6 | Move the focused window into the left column |
| F7 | Split the focused window into a new column |
| F8 / F9 | Focus the window above / below |
| F10 / F11 | Move the column left / right |
| F12 | Close the focused window |
| Ctrl+Shift+F8 | Quit the preview |

## Quick visual check

- Use F5 repeatedly: content should resize smoothly without trails or flashing.
- Use F6, then F8/F9 and F7 to check stacked columns and focus.
- Move the pointer to the second monitor, pan its canvas to a different region,
  then press F1 to open a window there before entering tiling with F2.
- In a preview terminal, run `dolphin`. Open its context menu and press F5:
  the menu should keep its native size while following the parent window.
- Use F2 during an animation and check that returning to the canvas is clean.

## Two-output acceptance

Before marking multi-output tiling ready for a release, record each monitor's
resolution and scale, then check the full path below on the physical TTY
session:

1. With the two canvases panned apart, open a terminal on each monitor and
   press F2. Each window should enter that monitor's own strip.
2. Move the pointer between monitors. F3/F4, F5 and F8/F9 should affect only
   the strip on the pointer's monitor; the other strip should retain its
   selection, width and window order.
3. Open another window on each monitor and verify it joins the strip on the
   output showing its canvas center. If output views overlap, the pointer-focused
   output receives the new window.
4. Move a window across outputs with the configured `send-to-output` binding.
   Confirm focus follows it, then unplug one output and verify its columns remain
   usable on the survivor. Reconnect it and confirm a fresh empty strip appears.
5. Press F2 to return to canvas. Check that windows return to their saved canvas
   positions and that neither output shows stale frames or leaves a window lost.

Automated fixtures cover output ownership, transfer and simulated hotplug; they
do not replace this physical-session acceptance. Record any failure and the
output geometry/scale in `tiling-status.md` before changing the release
readiness note.

The user confirmed the menu scenario stays in place without lag on the direct
udev backend. Artificially delayed clients still rely on automated coverage.
Nested NVIDIA presentation remains unsuitable for visual acceptance of blur.

## If tiling does not start on both outputs

Check that you used `run-multimon-tty.sh`, not the older single-connector
launcher, and that both outputs were connected before pressing F2. The launcher
log is saved to the path printed at startup; inspect it after exiting if an
output failed to initialize.

## Log

Launcher output is saved to
`~/driftwm-previews/7e32bf5/multimon-tty.log` (overwritten on each launch).
