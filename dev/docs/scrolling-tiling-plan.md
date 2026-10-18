# Scrolling tiling: implementation proposal

The full-parity goal and subsequent execution order now live in
[Niri tiling parity: execution plan](niri-tiling-parity-plan.md). This document
retains the initial proposal and implementation history; its original limited
scope is not the completion criterion for full niri tiling parity.

Status: initial single-output scrolling mode implemented. Existing canvas
behavior remains the default. User instructions: [tiling](../../docs/tiling.md).

## Implementation progress

`src/layout/scrolling/` now calculates single- and multi-window columns from
stage IDs and caller-supplied order. It supports fixed/proportional frame
widths, content constraints, decorations, inter-column gaps, and a camera
reveal target at zoom 1 with output-local usable-area offsets. Invalid inputs
and coordinates outside the i32 canvas range return errors.

The initial overflow policy honors client minimum sizes, placing the next
column after the resulting frame. Oversized windows align their leading edge
when revealed. A conflicting maximum never overrides a minimum. Target
geometry is independent of client commits and camera motion.

Stage-owned topology now records column order, active selection and saved canvas
rectangles. Removal/crash prunes membership and replacement keeps the stable ID.
The compositor integrates map/close, focus, fixed width presets, camera reveal
and mode exit through `toggle-tiling`, `focus-column` and `cycle-column-width`.
Reflow updates stage positions without raising windows. A delayed/refused size
reserves the client's live width without repeatedly configuring it.

The initial mode requires one connected output. Connecting another output exits
it. Incompatible canvas actions, move/resize, fullscreen and fit/fill leave the
mode; windows and view restore through the shared exit-placement path. Dialogs,
widgets, pinned/fitted/filled windows and stand-ins are excluded. Durable sessions
save the original canvas rectangles and view, not temporary tiled geometry.

Steps 2 and the initial integration portion of step 3 are implemented. Individual
detach to canvas, configurable width presets, gesture settling,
multi-output ownership and persistent topology remain outstanding. Horizontal
pan currently reuses existing momentum without column snapping.

Step 4 now includes moving a window into an adjacent column, expelling it into
a separate column, vertical focus, per-column selection memory and equal height
allocation subject to min/max constraints. Impossible minimum heights overflow
downward. Delayed or refused client sizes reserve their current frames on both
axes. Height weights, vertical reordering and interactive tile resizing remain
future extensions.

Validation includes pure geometry tests, stage topology/model tests and headless
Wayland scenarios for transitions, map/close, focus, width cycling, delayed/refused
sizes, dialog exclusion, hotplug and saving the canvas session while tiling.
The full library suite passes with `PROPTEST_CASES=4000` (348 tests).
The full `dev/scripts/ci-local` sequence passes, including ignored scenarios and
the isolated soak test. Fourteen headless scenarios cover tiling and its session
integration. These checks include the pre-existing output-management changes in
the working tree. No graphical session has been launched for manual validation.

## Reference implementation

Inspected niri commit `12de51d6a60baad8896366920278a24f17921f52`
(local checkout: `/tmp/niri`). Source links below are pinned to that revision.

- [scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs):
  `ScrollingSpace` owns ordered columns, active-column selection, working area
  and `ViewOffset`. `Column` owns vertically arranged tiles, desired width,
  active tile and fullscreen/full-width state. Widths can be proportional or
  fixed. `update_tile_sizes_with_transaction` resolves constraints and vertical
  allocation; consume/expel operations change column membership.
- [workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs):
  a workspace combines scrolling and floating layouts; monitor/workspace
  management sits above the scrolling algorithm.
- In `scrolling.rs`, `view_offset_gesture_begin/update/end` distinguish direct
  gesture control from animation. Gesture end projects velocity and chooses
  column alignment points. Scrolling therefore includes focus and settling
  policy, not just translation of window rectangles.
- [transaction.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/transaction.rs)
  coordinates asynchronous changes with a deadline. Reuse the principle of
  bounded waiting, rather than importing niri's rendering infrastructure.
- [layout tests](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tests.rs)
  model requested versus committed geometry and clients refusing requested
  sizes. This distinction belongs in driftwm tests from the first integration.

Use these as design references. Porting the whole layout subsystem would also
import assumptions about workspaces, output ownership, rendering and animation.

## Behavior and design decisions

The initial feature is an opt-in horizontal scrolling layout: opening a normal
window inserts a column after the active column without shrinking existing
columns. Columns may later contain multiple windows sharing the working height.
Directional focus reveals the selected tile; column widths have configurable
presets. Dialogs, widgets and explicitly floating windows stay outside tiling.

Recommended scope: a switchable canvas/scrolling mode, without adding workspaces.
Keep the core data model capable of multiple strips, but initially validate one
strip before enabling independent tiling on several outputs.

The initial implementation follows the single-output, compositor-wide proposal.
These choices frame the remaining extensions:

1. Is the mode compositor-wide, output-specific, or local to a canvas group?
   This proposal starts with a compositor-wide opt-in mode. Group-local tiling
   would need explicit group creation, selection and lifetime rules instead.
2. Who owns sizing? A strip needs one designated output as its sizing authority.
   Other viewports may observe it but must not reflow it. Per-output strips need
   explicit ownership and placement on the shared canvas, plus transfer and
   disconnect rules; output ownership must not be inferred from visibility.
3. Recommended first interaction: enter at zoom 1, save canvas camera/zoom,
   scroll horizontally, and restore the saved view on exit. Define whether
   pinch/overview temporarily releases this constraint or leaves the mode.
   Camera movement must never change the layout origin or trigger reflow.
4. Save pre-tiling window rectangles on entry. On exit restore surviving
   windows; windows created while tiling need a documented canvas placement
   fallback. Mode exit and individual window detachment are separate actions.
5. In tiling, full-width means column width equals working width. Fullscreen
   remains the existing protocol operation. Decide explicitly how existing
   fit/fill/snap actions behave; two policies cannot own the same window size.

## Integration boundaries

- `src/layout/scrolling/`: deterministic column geometry, size allocation and
  visibility calculations. Inputs are IDs, constraints, working area, chrome
  and settings. Results are desired rectangles and a camera target; no Wayland
  requests or compositor state access.
- `src/stage/`: authoritative strip/column membership keyed by `ElementId`,
  ordering and saved canvas placement. Store topology here rather than keeping
  a second window registry in the compositor. Define removal, replacement by a
  suspended stand-in and restoration invariants alongside existing membership.
- `src/state/tiling.rs`: policy adapter applies layout results through the
  existing stage-backed mutation paths and sends size configures. Keep methods
  outside `state/mod.rs`; add only necessary state declarations there.
- `src/handlers/xdg_shell.rs` and `src/handlers/compositor.rs`: initial map,
  first buffer, configure acknowledgement and committed geometry. Integrate
  before canvas auto-placement can reposition a tiled window. Separate target
  geometry from client geometry and reject stale layout completions.
- `src/state/window_lifecycle.rs`: close/crash cleanup and reflow. Note that
  `map_window` currently calls `Stage::map`, which also raises the window.
  Moving every tile must not repeatedly change stacking or activation; add a
  dedicated stage-backed position update if necessary.
- `src/state/navigation.rs`, viewport animation and input actions/gestures:
  topology-based navigation in tiling, existing spatial navigation in canvas.
  Reuse the existing camera and animation machinery; avoid a second render
  transform. Preserve destination-based window hit-testing rules.
- Output sizing, usable areas and hotplug: include layer-shell reservations,
  decorations, fractional scale and output transforms. Keep output membership
  routed through `refresh_window_outputs`; sizing ownership and surface
  visibility on another monitor are separate concepts.
- Config, window rules, IPC and session persistence expose the same policy.
  Existing `SwitchLayout` refers to keyboard layout: use distinct action names
  for window arrangement. Regenerate `docs/config.md` from the reference.

## Delivery sequence

Each numbered stage should be a focused, reviewable change; create the upstream
design issue before feature implementation, following `CONTRIBUTING.md`.

1. **Specify semantics and invariants.** Resolve the choices above, define mode
   transition examples and action precedence, and document unsupported actions
   for the first release. Acceptance: deterministic expectations for entry,
   exit, new windows, close, fullscreen and a second monitor.
2. **Build the pure layout model.** Ordered columns, one tile per column,
   fixed/proportional widths, gaps, constraints and camera reveal targets.
   Add unit tests and model-based operation sequences. Acceptance: no duplicate
   IDs, dangling selection or negative geometry; adding a column preserves
   existing requested widths unless working-area constraints require a change.
3. **Integrate a usable vertical slice.** Manual opt-in, one sizing output,
   map/close, focus left/right, width presets, detach-to-canvas and mode exit.
   Include the configuration/actions needed to exercise this slice. Test actual
   configure/commit sequences, stale responses, refusing clients and crash
   cleanup in the headless fixture. Never wait indefinitely for an application.
4. **Add multi-window columns.** Consume/expel, focus and reorder up/down,
   weighted height allocation and constraint overflow policy. Interactive
   resizing changes column width or tile allocation rather than free canvas
   geometry. Define deterministic behavior when minimum heights cannot fit.
5. **Integrate gestures and transitions.** Horizontal pan, velocity-based
   settling, focus reveal, interruption by new gestures, and existing window
   animations. Pinch/overview, fullscreen restoration, fit/fill, pinning,
   dialogs, snap clusters and suspended windows need explicit transition tests.
6. **Complete multi-output support.** Implement the agreed ownership model,
   transfer commands, reconnect/disconnect, working-area/scale changes and
   simultaneous views of the same strip. Ensure one window never receives
   competing size requests from two outputs. Until this lands, constrain
   activation to the supported topology rather than silently rearranging it.
7. **Finish public interfaces and persistence.** Window rules, IPC inspection
   and commands, config reload behavior, versioned session restoration and user
   documentation. Restore topology with stable persisted identities, not
   process-local `ElementId` values. Ignore missing windows safely.

Tabbed columns, drag insertion hints and configurable centering policies can
follow the initial feature. They need not block a usable scrolling mode.

## Validation

- Pure tests: width/height allocation, rounding, constraints, focus reveal and
  stable results for unchanged inputs. Property tests cover insert/remove,
  reorder, detach, mode transitions and working-area changes.
- Stage model tests: exclusive membership, teardown, fullscreen restoration,
  suspended replacement and no change to existing canvas invariants.
- Headless fixture: map/ack/commit ordering, delayed/refused sizes, close while
  reflow is pending, dialogs, mode exit during a pending resize and hotplug.
- Manual nested session: gesture feel, animation interruption, decorations,
  popup hit-testing and real terminal size increments. Real multi-monitor
  validation follows the ownership implementation.
- Run relevant tests during each change, then the documented CI sequence via
  `dev/scripts/ci-local`. For layout/stage/config changes, also run the deeper
  property-test pass from `dev/docs/testing.md`.

The mode is covered by the headless compositor fixture; gesture feel and visual
behavior still need manual validation in a nested compositor session.
