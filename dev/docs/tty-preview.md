# Local TTY preview cheat sheet

These commands describe nationi's local isolated preview, not default driftwm
bindings. The launcher uses the current `target/release/driftwm` binary, a
separate config, runtime directory and session file.

For the current source checkpoint and validation, see
[status and remaining work](tiling-status.md) or [the general user guide](../../docs/tiling.md).
The preview runs the last release binary built locally; its version may lag Git HEAD.
The launcher/config are local files outside Git; a fresh clone does not include them.

## Start and return

1. Press **Ctrl+Alt+F3** and log in as `nationi`.
2. Run without sudo:

   ```bash
   ~/driftwm-previews/7e32bf5/run-tty.sh
   ```

3. Press **F2** to enter tiling.

The default test output is `HDMI-A-1` (GIGA-BYTE). To test the Dell instead:

```bash
~/driftwm-previews/7e32bf5/run-tty.sh HDMI-A-2
```

**Ctrl+Shift+F8** quits the preview. **Ctrl+Alt+F2** returns to the main desktop
without quitting it. Close an older preview and rerun the launcher after a new
release build. The directory name records the original preview setup; it does
not pin the binary to that commit.

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
- In a preview terminal, run `dolphin`. Open its context menu and press F5:
  the menu should keep its native size while following the parent window.
- Use F2 during an animation and check that returning to the canvas is clean.

The user confirmed the menu scenario stays in place without lag on the direct
udev backend. Artificially delayed clients still rely on automated coverage.
Nested NVIDIA presentation remains unsuitable for visual acceptance of blur.

## If only F1 works

Check that you launched with `run-tty.sh`, which restricts the preview to one
connector. Tiling requires exactly one output; a run enrolling both monitors
rejects F2 and its dependent layout commands. This does not explain failures of
F12 or the quit binding, which work independently of tiling.

## Log

Launcher output is saved to
`~/driftwm-previews/7e32bf5/tty-preview.log` (overwritten on each launch).
Single-connector testing does not implement multi-output tiling.
