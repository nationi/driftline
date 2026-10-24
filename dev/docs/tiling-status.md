# Tiling development status

Updated 2026-10-24. Driftline's latest release is 1.1.4. The `main`
branch is the only integration branch; the former `nationi/development` and
`rebrand/driftline` branches were
merged and deleted. Canvas remains the default, and niri is a design reference
rather than a parity target.

## Current behavior

- Every connected output has its own scrolling-column layout, selection and
  view. Windows can be focused, reordered, stacked, resized by column width,
  and transferred between outputs.
- `Mod+Ctrl+T` switches between canvas and tiling. Keyboard and pointer actions
  intended for one mode do not switch modes or run in the other. `Alt+Tab` cycles
  windows without leaving tiling.
- `Alt`+left-drag swaps tiled windows, shows the dragged picture, and outlines a
  valid same-output drop target without changing focus before release. An
  invalid drop animates the picture back to its original tile without changing
  the layout. Entering tiling briefly shows a fading `TILING MODE` label on each
  output. Exiting animates tiled windows back to their saved canvas positions
  and sizes.
  Fullscreen covers only its output and keeps its tile slot; closing that
  window removes it.
- Hover can focus a visible tile without recentering the tiled camera. When
  hover selects a partly clipped tile, the camera reveals it while preserving a
  48-logical-pixel strip of the previous selection, so the pointer can switch
  focus back. Keyboard and `Alt+Tab` focus navigation suppresses camera-follow
  hover until the user moves the camera, so a stationary cursor cannot undo the
  selected focus.
- `Alt`+right-drag widens or narrows the focused column horizontally; the
  resulting fixed width is saved with the session. Resizing respects the
  largest client-declared minimum width among the column's windows. A compact
  width label near the pointer updates during the drag and disappears on
  release. Layout movement follows the drag immediately without waiting for
  client resize commits; repeated pointer updates are coalesced to a 16 ms
  layout-refresh interval.
- Tiling topology is stored with durable session entries: output name, column
  and stack order, width, selected window and active column. Open apps with
  launchable desktop entries are started after restart and adopt their saved
  slots; explicitly suspended windows remain dormant. The compositor still
  starts in canvas mode, and the layout is rebuilt when the user enters tiling.
- If an output is disconnected during tiling, its columns transfer to a
  surviving output. Removing the last output restores canvas mode. Canvas
  geometry and each output's view are retained.

## Scope and limits

The project focuses on practical scrolling columns alongside the infinite
canvas. It does not add workspaces, tabs, or a general floating-window policy.
Trackpad column snapping, drag-based column reordering and per-window detach to
the canvas are not implemented. Suspended stand-ins do not join columns until
their applications are relaunched.

When a saved output name is unavailable at tiling entry, its restored windows
are placed on an available output. Apps without a usable desktop entry or whose
windows cannot be matched remain as suspended stand-ins. Tiling mode is not
restored at startup.

## Validation

`Alt`+right-drag now adjusts the tiled column width horizontally and saves the
final fixed width when the grab ends. The Alt-only resize binding is active for
tiles; `Alt`+`Shift`+right-drag stays canvas-only. Focus navigation also parks
sloppy-focus camera pulls for tiled `Alt+Tab` navigation. Hovering a clipped
tile can reveal it without hiding the previous selection from the pointer.
Tiling animation feedback includes a reorder-target
outline, a live width label, a return animation for invalid reorder drops, and a
brief per-output mode label. Successful swaps use the existing reflow
animation. Mode cues do not change camera or window geometry and are omitted
with `animation_speed = 1`. Focused fixture tests cover resize, mode isolation,
navigation, hover, drag feedback, invalid-drop return and mode-cue lifecycle
across multiple outputs. An ignored surfaceless-GPU pixel test verifies the
mode label appears and fades. `cargo fmt --check`, all 65 `tests::tiling::`
fixtures, that GPU test, all-target Clippy and `git diff --check` pass. The
generated configuration reference is in sync. Manual testing on a live session
remains.

The user confirmed the multi-monitor tiling workflow on 2026-10-24. Restart
coverage verifies that still-open eligible apps are relaunched and adopt their
saved entries while explicitly suspended windows remain dormant. Fixture
coverage also verifies saved window placements and Stage reconstruction of
columns, stacks, widths, selection and active column state. Compatibility with
a schema-v2 session without tiling fields is covered.

`dev/scripts/ci-local` passes on the release candidate: formatting, locked
check, all-target Clippy, test compilation, 1685 compositor tests including
ignored rendering cases, all integration targets and the isolated soak test.
The rendering tests use deterministic animation time for frame-driven checks.
Automatic app relaunch and clipped-tile hover return also have focused fixture
regressions. Manual verification of tiling restore across a real compositor
restart on the target multi-monitor setup remains.

## Start here

- User behavior and bindings: [tiling guide](../../docs/tiling.md)
- Session persistence: [session guide](../../docs/session.md)
- Scope and acceptance: [roadmap](scrolling-tiling-roadmap.md)
- Architecture and testing rules: [caveats](caveats.md), [testing](testing.md)
- Local hardware preview: [TTY guide](tty-preview.md)
