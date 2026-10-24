# Tiling development status

Updated 2026-10-24. Driftline's latest published release is 1.1.3. The `main`
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
- `Alt`+left-drag swaps tiled windows and shows drag feedback. Fullscreen covers
  only its output and keeps its tile slot; closing that window removes it.
- Hover can focus a visible tile without recentering the tiled camera. Keyboard
  and `Alt+Tab` focus navigation suppresses camera-follow hover until the user
  moves the camera, so a stationary cursor cannot undo the selected focus.
- `Alt`+right-drag widens or narrows the focused column horizontally; the
  resulting fixed width is saved with the session.
- Tiling topology is stored with durable session entries: output name, column
  and stack order, width, selected window and active column. After restart, the
  compositor still starts in canvas mode; the layout is rebuilt when the user
  enters tiling and the corresponding app windows have adopted their stand-ins.
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
are placed on an available output. Layout state follows session-restored
windows; it does not auto-launch applications or restore tiling mode at startup.

## Validation

`Alt`+right-drag now adjusts the tiled column width horizontally and saves the
final fixed width when the grab ends. The Alt-only resize binding is active for
tiles; `Alt`+`Shift`+right-drag stays canvas-only. Focus navigation also parks
sloppy-focus camera pulls for tiled `Alt+Tab` navigation, and hover focus leaves
the tiled camera in place. Focused fixture tests cover resize, mode isolation,
navigation and hover; all 62 tiling tests, 35 hover-focus tests and 10 cycle
tests pass. The generated configuration reference is in sync. Formatting,
all-target Clippy and `git diff --check` pass. Manual testing on a live session
remains.

The user confirmed the multi-monitor tiling workflow on 2026-10-24. The restart
topology change has fixture coverage for saving and loading window placements,
and Stage coverage for rebuilding columns, stacks, widths, selection and active
column state. Compatibility with a schema-v2 session without tiling fields is
covered.

`dev/scripts/ci-local` passed after the restart-persistence implementation:
formatting, locked check, all-target Clippy, test compilation, 1675 tests
including ignored rendering cases, integration targets and the isolated soak
test. Manual verification of tiling restore across a real compositor restart
on the target multi-monitor setup remains.

## Start here

- User behavior and bindings: [tiling guide](../../docs/tiling.md)
- Session persistence: [session guide](../../docs/session.md)
- Scope and acceptance: [roadmap](scrolling-tiling-roadmap.md)
- Architecture and testing rules: [caveats](caveats.md), [testing](testing.md)
- Local hardware preview: [TTY guide](tty-preview.md)
