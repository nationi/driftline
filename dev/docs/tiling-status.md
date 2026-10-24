# Tiling development status

Updated 2026-10-24. The fork's 1.0.0 baseline is integrated into both `main`
and `nationi/development` at `f29bf70`; documentation was refreshed in
`6a4f418`. Canvas remains the default. Niri is a design reference, not a parity
target.

## Current behavior

- Each connected output owns an independent scrolling strip. Windows can be
  focused, reordered, stacked, widened, transferred between outputs and
  restored to their canvas geometry.
- Canvas and tiling keyboard actions are mode-exclusive. `Mod+Ctrl+T` toggles
  modes; `Mod+R` cycles column widths; `Mod+Shift+R` toggles half/full width.
- `Mod+Arrow` centers the nearest window or anchor on the canvas and focuses
  the adjacent column/window in tiling. Explicit tiling focus is protected from
  the camera-driven hover-focus pull when `focus_follows_mouse` is enabled.
- Alt+left-drag carries the window picture with the pointer while preserving its
  layout slot as the drop target; release swaps it with the window underneath
  on the same output. Fullscreen covers only its target output while the
  multi-output tiling session remains active; the fullscreen window retains its
  slot on exit and is pruned if it closes.
- Toggling back into tiling during the same compositor session restores output
  ownership, column order, vertical stacks, widths and selected tiles. Closed
  or ineligible windows are skipped; new windows get new columns. This topology
  is not persisted across compositor restarts.
- Removing one output transfers its columns to a surviving output. Removing the
  last output restores canvas mode. Canvas geometry and view are retained.

## Scope and known limits

The fork focuses on practical scrolling columns alongside the infinite canvas.
It does not add workspaces, tabs, a general floating-window policy, or layout
restore across compositor restarts. Fullscreen is output-local and leaves other
outputs in tiling mode. Trackpad column snapping and layout-native drag resizing
are not implemented.

The automated multi-output fixtures cover independent layouts, mapping,
cross-output transfer, disconnect/reconnect policy and tiling re-entry. The
full hardware checklist is in [the TTY acceptance guide](tty-preview.md). No
formal run record is kept in this repository, so hardware-specific behavior
should be confirmed on the target setup.

## Latest validation

At the 1.0.0 integration commit, `dev/scripts/ci-local` passed in full: format,
locked check, all-target clippy, test compilation, 1668 library/compositor
tests including ignored GPU cases, all integration targets and the isolated
soak test. The release build also passed. `.github/workflows/ci.yml` remains
the source of truth for CI; the local hardware checklist is documented above.

The 1.1.0 release binary is installed at `~/.local/bin/driftline` and
`/usr/local/bin/driftline`; the `driftwm` command remains a compatibility alias.
The active Wayland process keeps its already-loaded executable until the next
session start.

## Latest fix validation

`cargo fmt --check`, all-target clippy, 54 tiling tests, all 25 keybinding
integration tests, the generated config-doc gate and `git diff --check` passed.
`cargo build --release --locked` passed; the matching 1.0.0 binary is installed
at both user and system paths.

## Alt-drag feedback

The tiled reorder grab now publishes its source window and pointer delta to the
renderer. While held, the window follows the pointer at reduced opacity and its
Stage geometry remains unchanged, so it can still be dropped over another tile.
The feedback clears on grab unset; canvas Alt-drag keeps its existing free-move
behavior. `cargo fmt` and `cargo check --locked` passed. Automated verification
of the live pointer/render path and manual multi-monitor acceptance remain
pending.

## Output-local fullscreen in tiling

Fullscreen no longer exits the global tiling session. The target output is
covered by fullscreen while each output keeps its independent tiled layout;
`Win+F` is mode-neutral, and the tiled window retains its slot when fullscreen
ends. Closing it prunes that slot normally. Added regressions cover keyboard
toggle, fullscreen close and a second output retaining its layout. All 56
tiling tests and 161 fullscreen-matched compositor tests passed. The focused
window-rule case for a fullscreen-on-map window also passes. All-target clippy,
formatting and `git diff --check` passed. Manual testing on the user's
multi-monitor setup remains.

## Canvas input isolation

Pointer bindings are mode-filtered before their direct grab paths, which
previously let resize and snapped-cluster drags call `prepare_tiling_operation`
and exit tiling without passing through `execute_action`. Tiling permits the
dedicated Alt+left reorder for a tile and actions explicitly valid in both
modes; canvas resize, cluster-move, pan and zoom bindings are ignored. Pinned
and suspended-window chrome and fullscreen pointer paths use the same filter.
Continuous swipe/pinch canvas bindings are suppressed in tiling; threshold
actions continue through the existing action-mode guard. Regression coverage
for Alt+right, Alt+Shift+right, Alt+middle, Alt+Shift+middle,
Alt+Shift+left and Alt+left reorder is in place. The focused regressions pass;
`cargo test --locked --bin driftline` passed (1592 passed, 81 ignored),
`cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt --check` and
`git diff --check` passed. Manual pointer testing on a live session remains.

## Version 1.1.0

Version 1.1.0 includes arrow-key navigation updates,
output-local fullscreen while tiling, Alt-drag reorder feedback, and canvas
mouse/gesture isolation while tiling. The focused changes and full bin test
suite pass; live multi-monitor acceptance remains hardware-specific. On
2026-10-24, `dev/scripts/ci-local` passed every CI stage: formatting, locked
check, all-target clippy, test compilation, 1672 tests including ignored GPU
cases, integration targets and the isolated soak test.

## Horizontal pointer panning in tiling

Super+left-drag scrolls tiling columns horizontally again. The tiling camera
already clamps pointer panning to the horizontal axis; the input mode filter
had been rejecting the `PanViewport` mouse binding before the grab could start.
An in-process regression covers horizontal movement and a fixed vertical camera.

## Fullscreen configure while tiling

After `Win+F`, a fullscreen client's configure acknowledgement previously made
the ordinary layout refresh reject that output's fullscreen tile and exit
tiling. The refresh now pauses for the fullscreen output, preserving its tile
slot until fullscreen exits; normal layout refresh resumes after exit. The
regression exercises the actual `Win+F` key path and acknowledges both
fullscreen and restore configures. All 58 tiling tests, formatting and
`git diff --check` passed. The fix is superseded by the fuller 1.1.2 fix below.

## Fullscreen restore recenter

Follow-up testing found a second transition edge: when the fullscreen buffer is
larger than the saved tiled size, exit creates a pending recenter while waiting
for the client to commit its restored buffer. That temporary state must not
invalidate an already-owned tile. Tiling refresh now preserves that membership
until the recenter settles. A second issue occurred when that restored buffer
arrived: the commit handler used ordinary placement, which exits tiling for a
free-moved tile. Recenter commits now preserve existing tile membership. The
regression covers the restoring commit. All 59 tiling tests and the compositor
test suite (1594 passed, 81 ignored), all-target clippy, formatting and
`git diff --check` passed. These fixes are included in release 1.1.2.

## Documentation baseline

The README describes the fork and uses versioned release checkouts. The user
tiling guide covers behavior and controls; detailed implementation and
validation history belongs in this status and the roadmap. Machine-specific
preview paths and the stale general TODO archive were removed. No backup or
editor-temporary files were present in the tracked tree during this review.

## Start here

- User behavior and bindings: [tiling guide](../../docs/tiling.md)
- Scope, sequence and acceptance: [roadmap](scrolling-tiling-roadmap.md)
- Architecture and testing rules: [caveats](caveats.md),
  [testing](testing.md)
- Local hardware preview: [TTY guide](tty-preview.md)
