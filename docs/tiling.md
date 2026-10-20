# Experimental scrolling tiling

Scrolling tiling arranges normal windows in a horizontal strip. Each column can
contain several windows stacked vertically. It currently supports a single
connected output. The canvas remains the default; no keybindings change until
you configure them.

For this checkout’s isolated test session and F1–F12 keys, use the
[local TTY cheat sheet](../dev/docs/tty-preview.md). The bindings below are
configuration examples, not that preview’s keys. Development scope and remaining
work are tracked in [the current status](../dev/docs/tiling-status.md).

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

New ordinary windows receive their initial column content size in the first
configure, including client size limits and compositor decorations. They join
the strip after their first buffer, after the column active at that time. Windows
waiting for their first buffer do not reserve space or take focus. Their first
configure order does not determine their eventual column order. Dialogs and explicit size/placement rules
keep their existing initial sizing policy. If a client chooses another first
buffer size, that size is saved for its later return to the canvas. If it accepts
the initial offer, that accepted size becomes the canvas fallback.

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

Related size requests now coordinate new buffer commits: an early reply waits
for the other affected windows, for at most 300 ms. A window may choose a
different size without holding up the group. Closing a participant or leaving
tiling releases waiting buffers. Window positions and view adjustments wait
until the corresponding buffers apply or the deadline expires. Refused sizes
still reserve enough space for the visible window. Eligible visible windows
keep a snapshot of their previous image while waiting,
when capture is available. Background blur keeps updating beneath the saved
window shape; its shadow remains below the blur. Popup menus are included without
clipping them to the window. Opening, closing, moving or repainting a popup
updates its live image without exposing the parent's pending buffer. Menus
follow the animated window origin but keep their native size during resize;
opening scale still applies. Opening fades and ordinary
position animations can continue on the saved picture. When the group releases,
windows animate from the saved geometry to their new positions and sizes.
Old and new content blend before window opacity is applied, including
translucent pixels. Ordinary unblurred tiled resize transitions keep border
strokes and compositor title bars at their native thickness and height while
the window changes size. If another transaction starts during a resolved canvas
resize, its current geometry and blended image are held until the new buffers
apply or the deadline expires, then the transition resumes from that picture.
Ordinary unblurred tiled resize recaptures reuse the mixed texture directly,
avoiding extra filtering of rounded corners at fractional output scales.
Canvas resizes still held for a client reply can also be captured before a new
group takes over. A late buffer arriving after the group expires can start a
bounded resize transition on an otherwise settled window without restarting
the group or repeating the animation on subsequent repaints. Degraded stale
resize content and fullscreen/pinned transitions retain their live fallback.
Complete visual atomicity and all animation transitions are not implemented yet.
