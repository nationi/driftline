# Scrolling-column tiling

Driftline includes an optional scrolling-column layout alongside its
infinite canvas. The canvas remains the default. In tiling mode, each connected
output has its own horizontal strip of columns; windows in a column are stacked
vertically. This is a practical workflow integrated with driftline, not an attempt
to reproduce every feature of another compositor.

## Switch modes

With the default Super modifier, press `Mod+Ctrl+T` to switch between canvas and
tiling. Changing modes does not make navigation shortcuts switch modes. Canvas
navigation and geometry actions are ignored in tiling, and tiling actions are
ignored on the canvas. General actions such as closing a window remain
available in both modes.

On the canvas, `Mod+Arrow` moves to and centers the nearest window or navigation
anchor. `Mod+Ctrl+Arrow` focuses the nearest window with the canvas's directional
reveal behavior. In tiling, `Mod+Arrow` focuses the adjacent column or the next
window in the current stack. This explicit tiling focus remains selected while
the layout scrolls, including when `focus_follows_mouse` is enabled.

## Default tiling keys

| Keys | Action |
| --- | --- |
| `Mod+Ctrl+T` | Switch between canvas and tiling |
| `Alt+Tab` / `Alt+Shift+Tab` | Cycle through windows / cycle backward without leaving tiling |
| `Mod+Arrow` | Focus a column or a window in its stack |
| `Mod+Shift+Left/Right` | Move the focused column |
| `Mod+Shift+Up/Down` | Move the focused window within its column |
| `Mod+Ctrl+Left/Right` | Move the focused window into an adjacent column |
| `Mod+Ctrl+Shift+Left/Right` | Consume a window from the next column / expel the last window |
| `Mod+Ctrl+E` | Move the focused window into a new column |
| `Mod+R` | Cycle the focused column's width |
| `Mod+Shift+R` | Toggle half-width and full-width column |
| `Mod+Alt+Arrow` | Send the focused window to an adjacent output |

Hold `Alt` and drag a tiled window with the left mouse button. The window's
picture follows the pointer while its original slot stays in the layout; release
over another window on the same output to swap their positions. On the canvas,
this gesture moves windows freely instead.

Canvas-only mouse manipulations such as `Alt`+right-click resize,
`Alt`+`Shift`+right-click resize, and `Alt`+`Shift`+left-drag of a snapped
cluster are ignored in tiling. Switch back to the canvas to use them. Tiled
windows can still be focused and reordered with `Alt`+left-drag.

To assign the same key chord to different actions in each mode, use the
mode-specific tables:

```toml
[keybindings.canvas]
"mod+left" = "center-nearest left"

[keybindings.tiling]
"mod+left" = "focus-column left"
```

These tables can also override or mask defaults. See the full
[configuration reference](config.md#keybindings) for syntax and available
actions. The modifier follows `mod_key` in the configuration.

## Columns and windows

Entering tiling sets the view zoom to 1 and arranges eligible windows. Columns
start at half the output's usable width. `Mod+R` cycles through full, one-third
and one-half width; `Mod+Shift+R` toggles half and full width. A column's width
applies to every window stacked in it. Output panels reserve their usual space.
Client size limits and compositor decorations are included when sizing windows.

Arrow keys move focus between columns and between windows in a stack. Navigation
does not wrap at the ends. Reorder columns horizontally or windows vertically
with the shortcuts above. `Mod+Ctrl+Left/Right` moves the focused window into a
neighboring column; `Mod+Ctrl+E` gives it a new column. The consume/expel
shortcuts move the adjacent column's first window into the focused column, or
split the focused column's last window into its own column. Actions at a layout
boundary do nothing.

Closing the last window in a column removes that column. Windows opened during
tiling join the active output's layout after their first buffer is committed.
When output viewports overlap, the active output owns a window visible on more
than one output. Otherwise, a window joins the output showing its canvas-space
center, with the active output as the fallback.

## Multiple outputs

Each output maintains an independent strip, selection and view. Focus and layout
operations apply to the output under the pointer. `Mod+Alt+Arrow` transfers the
focused window to the adjacent output while keeping it tiled. A single-window
column moves as a unit; a window from a stack becomes a new column. The
destination receives focus.

An output connected while tiling is active gets an empty strip. If one of
several outputs disconnects, its columns move to a surviving output. Removing
the last output ends tiling and restores the canvas. When output viewports
overlap on entry, windows visible in both are assigned to the active output;
pan the views apart first if you want those windows on different strips.

## Return to the canvas

Switching back restores saved window positions and sizes, as well as each
output's camera and zoom. Tiling order, stacks, widths, output ownership and
selection are saved with the session. After a restart, windows restored from
session records return to their previous output, column order, stack order,
column width and selected position when you switch into tiling. The compositor
still starts in canvas mode. This requires the same output name to be connected;
if a saved output is absent, its windows join an available output.

Fullscreen covers only its target output while tiling stays active on every
output. The fullscreen window keeps its column slot and returns to it when
fullscreen ends; closing it removes that window from the layout. `Win+F` enters
and exits fullscreen without changing modes. Fit, fill, pinning, free movement
or resizing, and zoom changes return to the canvas before applying the action.
Dialogs, widgets, pinned windows, fitted/filled windows and suspended stand-ins
do not join columns.

## Current limits

Scrolling tiling is focused on columns and independent per-output layouts. It
does not include workspaces, tabs, a general floating-window layer, drag-based
column reordering, per-window detach to the canvas, trackpad column snapping, or
layout-native drag resizing. Column width presets are fixed.

For development scope and validation notes, see the
[tiling status](../dev/docs/tiling-status.md) and
[roadmap](../dev/docs/scrolling-tiling-roadmap.md).
