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
available in both modes. Entering tiling briefly shows a `TILING MODE` label on
each output. Leaving tiling animates the windows from their tiled positions and
sizes back to their saved canvas positions and sizes.

On the canvas, `Mod+Arrow` moves to and centers the nearest window or navigation
anchor. `Mod+Ctrl+Arrow` focuses the nearest window with the canvas's directional
reveal behavior. In tiling, `Mod+Arrow` focuses the adjacent column or the next
window in the current stack. This explicit tiling focus remains selected while
the layout scrolls, including when `focus_follows_mouse` is enabled.

With `focus_follows_mouse` enabled, hovering a partly clipped tile can scroll
the layout toward it. The movement keeps a strip of the previously selected tile
visible, so you can move the pointer back to it and switch focus again.

## Default tiling keys

| Keys | Action |
| --- | --- |
| `Mod+Ctrl+T` | Switch between canvas and tiling |
| `Alt+Tab` / `Alt+Shift+Tab` | Cycle through windows / cycle backward without leaving tiling |
| `Mod+Arrow` | Focus a column or a window in its stack |
| `Mod+X` | Focus the tiled window under the pointer and center it on screen |
| `Mod+Shift+Left/Right` | Move the focused column |
| `Mod+Shift+Up/Down` | Move the focused window within its column |
| `Mod+Ctrl+Left/Right` | Move the focused window into an adjacent column |
| `Mod+Ctrl+Shift+Left/Right` | Consume a window from the next column / expel the last window |
| `Mod+Ctrl+E` | Move the focused window into a new column |
| `Mod+R` | Cycle the focused column's width |
| `Mod+Shift+R` | Toggle half-width and full-width column |
| `Mod+Alt+Arrow` | Send the focused window to an adjacent output |

Hold `Alt` and drag a tiled window with the left mouse button. The window's
picture follows the pointer while its original slot stays in the layout. The
drop position chooses the operation:

- Release near the top or bottom edge of another tile on the same output to
  insert the dragged window into that tile's column, before or after it.
- Release over the middle of another tile to swap their positions, as before.
- Drag a window out of a stack and release just beyond the source column's left
  or right edge in open space to give it a new column immediately before or
  after its source column. The new column keeps the source column's width.
- Release near the left or right edge of another tile to place the dragged
  window's column beside it. A singleton column moves as a unit; a window from
  a stack becomes a new column there.

While holding a tiled window near the left or right edge of the monitor, the
view scrolls horizontally toward adjacent columns, even when the pointer is over
another tile. This works for narrow and full-width columns and stops at the end
of the layout.

The destination is marked with a bright cyan preview while dragging, independent
of window decoration settings. A full outline previews a swap; a horizontal bar
at a tile edge previews insertion into its stack, and a vertical bar previews
column placement. A label beside the pointer names the pending action,
including when a stacked window will become a new column. Focus and layout
change only on release. Dropping somewhere without a valid target smoothly
returns the picture to its original tile without changing the layout. On the
canvas, this gesture moves windows freely instead.

You can also keep holding `Alt`+left-drag across an output boundary and release
on another monitor to transfer the tiled window there. The dragged picture
follows the pointer, the destination output shows a `MOVE WINDOW HERE` cue, and
the transfer completes on release. A single-window column moves as a unit; a
window from a stack becomes its own column on the destination.

Hold `Alt` and drag with the right mouse button over a tiled window to widen or
narrow its column. Drag right to expand and left to shrink; the width applies to
every window stacked in that column. The column will not shrink below the
largest minimum width requested by its windows. A compact rounded label follows
the pointer with the current width in pixels, and neighboring columns track
the drag without waiting for each application to resize. At a monitor edge the
pointer stays on its starting output while its camera pans slowly, letting the
column keep growing or shrinking. `Alt`+`Shift`+right-drag and
`Alt`+`Shift`+left-drag of a snapped cluster remain canvas-only.

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
destination receives focus. Alt+left-drag can also transfer a window directly
to the output under the pointer.

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
column reordering, per-window detach to the canvas, or trackpad column snapping.
Column widths can be resized by mouse drag or set to one of the keyboard
presets.

For development scope and validation notes, see the
[tiling status](../dev/docs/tiling-status.md) and
[roadmap](../dev/docs/scrolling-tiling-roadmap.md).
