# Experimental scrolling tiling

Scrolling tiling arranges normal windows in a horizontal strip, with one window
per column. It currently supports a single connected output. The canvas remains
the default; no keybindings change until you configure them.

Add bindings to `~/.config/driftwm/config.toml`:

```toml
[keybindings]
"mod+ctrl+t" = "toggle-tiling"
"mod+alt+left" = "focus-column left"
"mod+alt+right" = "focus-column right"
"mod+alt+r" = "cycle-column-width"
```

Merge these entries into an existing `[keybindings]` table, then reload your
configuration. The actions are also available in mouse and gesture bindings.

Entering the mode sets zoom to 1 and arranges eligible windows. Columns start
at half the usable output width. Width cycling selects full width, one third,
then one half again. Client size constraints and compositor decorations are
included. Panels reserve their usual space. A client that delays or refuses a
resize keeps enough space for its current width.

New windows join after the active column. Closing a window removes its column.
`focus-column` moves without wrapping and scrolls the selected frame into view;
the existing `center-nearest left` and `center-nearest right` actions do the
same while tiling. Trackpad panning is horizontal, using existing momentum.
Column snapping at the end of a gesture is not implemented yet.

Toggle again to restore the saved canvas positions, sizes, camera and zoom.
Windows opened during tiling return to the canvas placement they received when
opening. Saved sessions retain these canvas rectangles and the pre-tiling view;
tiling topology itself is not restored across restarts.

Dialogs, widgets, pinned windows, fitted/filled windows and suspended stand-ins
are not enrolled. Fullscreen, fit/fill, pinning, free window movement/resizing
and other canvas actions leave tiling before running. Connecting another output
also leaves tiling. A zoom change leaves tiling and restores the saved view.

This first version does not provide multiple windows in a column, independent
strips on several monitors, drag-to-reorder, per-window detach or configurable
width presets. The fixed presets are one third, one half and full usable width.
