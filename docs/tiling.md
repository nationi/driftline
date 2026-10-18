# Experimental scrolling tiling

Scrolling tiling arranges normal windows in a horizontal strip. Each column can
contain several windows stacked vertically. It currently supports a single
connected output. The canvas remains the default; no keybindings change until
you configure them.

Add bindings to `~/.config/driftwm/config.toml`:

```toml
[keybindings]
"mod+ctrl+t" = "toggle-tiling"
"mod+alt+left" = "focus-column left"
"mod+alt+right" = "focus-column right"
"mod+alt+up" = "focus-column up"
"mod+alt+down" = "focus-column down"
"mod+alt+r" = "cycle-column-width"
"mod+alt+shift+left" = "move-into-column left"
"mod+alt+shift+right" = "move-into-column right"
"mod+alt+e" = "expel-from-column"
"mod+ctrl+alt+left" = "move-column left"
"mod+ctrl+alt+right" = "move-column right"
"mod+ctrl+alt+up" = "move-tiled-window up"
"mod+ctrl+alt+down" = "move-tiled-window down"
```

Merge these entries into an existing `[keybindings]` table, then reload your
configuration. The actions are also available in mouse and gesture bindings.

Entering the mode sets zoom to 1 and arranges eligible windows. Columns start
at half the usable output width. Width cycling selects full width, one third,
then one half again. Client size constraints and compositor decorations are
included. Panels reserve their usual space. A client that delays or refuses a
resize keeps enough space for its current width and height.

New windows create a column after the active column. `move-into-column left`
or `right` moves the focused window into the neighboring column, immediately
after its selected window. An empty source column disappears. `expel-from-column`
splits the focused window into a new column on the right, preserving column
width and its original canvas rectangle; it does nothing for a single-window
column. Moving past either end of the strip does nothing.

`consume-window-into-column` appends the first window from the next column to
this column, keeping the current selection. `expel-window-from-column` moves the
last window into a new column on the right, keeping the source column active.
If that last window was selected, focus moves to the preceding window in the
source column. These commands do nothing at the end of the strip or in a
single-window source column, respectively. They follow niri's basic topology
and selection rules; sizing policies and animations still differ. The shorter
`move-into-column` and `expel-from-column` commands retain the focused-window
behavior described above.

`move-column left/right` reorders the focused column as a whole.
`move-tiled-window up/down` reorders the focused window within its column.
Both preserve selection and do nothing at a boundary. Column identity survives
focus changes and reordering. Changes to columns on the left preserve the
selected column's horizontal position relative to the camera, including during
a camera animation; canvas restore positions remain unchanged.

Windows share column height equally, subject to their content size limits and
decorations. Space a maximum-limited window cannot use goes to the others. If
minimum heights cannot fit, the column grows downward without overlapping its
windows. Focusing an off-screen window reveals it vertically as needed.

`focus-column left/right` changes columns, remembering the selected window in
each. `focus-column up/down` changes windows inside a column. Navigation does
not wrap; the corresponding `center-nearest` actions follow the same structure
while tiling. Closing a window redistributes height; closing the last window
removes the column. Closing the selected window chooses the next window or
column, falling back to the previous one at the end. A newly opened column
returns focus to its source column when closed, until another column is focused
or the column is reordered. Repeated focus within the new column preserves this
return; removing a column to its left cancels it. Returning also restores the
previous horizontal view target, adjusted to keep a resized column visible.
The return animates from the current view. Background closure preserves
keyboard focus. Trackpad panning is horizontal, using existing momentum.
Column snapping at the end of a gesture is not implemented yet.

Toggle again to restore the saved canvas positions, sizes, camera and zoom.
Windows opened during tiling return to the canvas placement they received when
opening. Saved sessions retain these canvas rectangles and the pre-tiling view;
tiling topology itself is not restored across restarts.

Dialogs, widgets, pinned windows, fitted/filled windows and suspended stand-ins
are not enrolled. Fullscreen, fit/fill, pinning, free window movement/resizing
and other canvas actions leave tiling before running. Connecting another output
also leaves tiling. A zoom change leaves tiling and restores the saved view.

This version does not provide independent strips on several monitors,
drag-to-reorder, per-window detach to the canvas or configurable
width presets. The fixed presets are one third, one half and full usable width.
