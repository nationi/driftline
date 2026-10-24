# Practical scrolling tiling roadmap

Updated 2026-10-24. This roadmap replaces the former niri parity objective. The
project is a trackpad-first infinite canvas compositor with an optional,
practical scrolling-column mode. Niri informs interaction choices where useful;
matching every niri behavior is not a goal.

Read [current status](tiling-status.md) for verified implementation and recent
validation, and [the user guide](../../docs/tiling.md) for current behavior.

## Product goal

Make scrolling-column tiling work across all connected monitors while preserving
canvas mode as the default. Each output should have its own horizontal strip of
columns, with windows stacked vertically inside columns. Users should be able to
focus, rearrange, combine and split columns, change widths, and move focus
between outputs without losing layout state. Removing an output must leave its
windows in a predictable usable state.

## Multi-output model

Borrow niri's ownership boundary, not its whole workspace system:

```text
Driftline
└── connected outputs
    └── per-output tiling state
        └── scrolling columns and their windows
```

Each output's state owns its layout identity, column order and selection,
configure/commit transactions, snapshots and view anchor. Layout geometry uses
that output's usable area and scale. Driftline's existing output state continues
to own each camera. A window belongs to at most one output layout; focusing
another monitor selects that monitor's layout. New windows join the output where
they are created, using their canvas-space center: if the active (pointer-focused)
output shows that center, it wins; otherwise the output whose viewport contains
the center is selected, with the active output as the final fallback. When
viewports overlap, the active output therefore owns the new window. This policy
is covered by compositor tests. Moving a tiled window or column transfers
ownership without changing stable window identity or saved canvas geometry.

There is no workspace layer in this design: one scrolling layout per output is
enough for the current product. On output removal, transfer its columns to a
surviving output and reflow there; if no output survives, end tiling and restore
windows to canvas. Cancel and drain transactions and render snapshots owned by
the removed output before dropping its state.

## Explicitly out of scope

These are not parity gaps and should not be implemented just to mirror niri:

- Workspaces, workspace overview, tabs and tab indicators.
- A complete floating-window layer or tiled fullscreen/maximize policy.
- Every niri action, configuration field, animation, gesture, or edge case.
- Exact niri workspace/overview semantics.
- Exhaustive source inventories, differential fixtures and compatibility
  percentages.

## What is already in place

The implementation already provides a substantial functional core: single-output
columns, stable layout/column IDs, separate per-column selection, directional
focus, column/window reorder, move-into-column and expel, width cycling, initial
size offers and first-buffer enrollment, client constraints, saved canvas
geometry/view restoration, close-return focus, and bounded resize/configure
handling. Rendering includes retained images, blur, SSD/CSD handling and animated
resize transitions. Automated compositor tests and controlled rendering gates
cover many asynchronous and visual cases. See the status file for exact evidence
and limits; these mechanisms are not all evidence of user-facing completeness.

## Delivery sequence

### 1. Give each output independent layout ownership

Stage groundwork is now in place: it stores tiling layouts by output name,
keeps layout/column IDs globally unique, tracks the selected output layout, and
can enumerate tiled membership across layouts. Compositor configure, snapshot
and timer state now lives in an output-keyed view collection; rendering and
buffer responses can select the owning view. Tiling entry now creates one view
per connected output, assigns existing windows to output layouts, preserves
each output's camera and zoom, and global exit drains and restores every view.
Hermetic two-output coverage verifies independent layout creation, sizing,
configure progress, saved views and exit; an ignored GPU regression verifies
each output renders its own retained snapshot and that one output's client
response does not release the other's snapshot.

Keep configure, commit, snapshot, animation and input state routed through the
owning output layout. Preserve stable window/column identity and canvas restore
geometry. Do not resize one window independently for multiple outputs.

Acceptance: entering tiling with two outputs creates two independent strips;
operations and newly mapped windows affect the intended output; each output can
render and resize its own windows without changing the other layout. The
implementation is in place. Fixture coverage verifies assignment of existing
and late-mapped windows, navigation to a tiled window on another output,
independent configure progress and output-specific size offers. An ignored GPU
case verifies snapshot rendering and independent release on the two outputs.
The user confirmed manual multi-monitor acceptance on 2026-10-24.

### 2. Handle output topology changes safely

When an output connects during tiling, initialize its empty layout without
disturbing existing outputs. This connect path is implemented and covered by a
fixture that checks stable existing ownership/configures, a fresh layout
identity, unit zoom and unchanged selected layout. When an output disconnects,
its columns now append to a surviving output in order, preserving IDs and saved
canvas rectangles; the compositor cancels source-output transactions and
reflows on the survivor. Last-output removal still exits tiling. Reconnect must
not resurrect stale IDs, configure groups, snapshots or ownership. Keep the
user-facing behavior predictable when output names are reused.

Acceptance: connect, surviving-output transfer, last-output removal and
same-name reconnect have passing fixture coverage. The unplug case with pending
configure groups and retained snapshots is covered by a fixture and an ignored
GPU pixel regression. The fixture keeps configure requests pending through
unplug and drains the survivor's reflow. The GPU case confirms the survivor's
held pixels remain stable while the removed output view is discarded.
`dev/scripts/ci-local` passes with both regressions included.

### 3. Add practical cross-output navigation and transfer

Make it clear which output receives focus and new windows. The existing
`send-to-output` action now transfers a focused tiled window while retaining
tiling: singleton columns move intact, and a window from a stack becomes a new
destination column. Window ID and saved canvas geometry remain stable; the
destination becomes focused and reflows. Keep
fullscreen, fit/pin and canvas-specific operations on their current documented
fallback path unless a concrete requirement expands scope.

Acceptance: the existing transfer has Stage and compositor fixture coverage
for stable identity, saved geometry, destination focus, configure convergence,
and different output sizes. The compositor test also covers a 1.5x destination
and a 700-pixel minimum width; a second compositor test covers splitting a
stacked tile across the two outputs. The user confirmed the multi-output
workflow; a separate whole-column binding can be considered if a concrete use
case emerges.

### 4. Validate the multi-output user journey

Exercise entry, focus, reorder, merge/expel, width cycling, new/closed windows,
cross-output transfer, hotplug and exit on a real multi-monitor session. Then
review config reload and output-specific rendering. Keep canvas as the default
and update the guide to describe only observed behavior.

Acceptance: focused tests and required CI pass; the user confirmed the manual
multi-monitor workflow and topology restore after a compositor restart on
2026-10-24.

### 5. Keep keyboard controls mode-exclusive

This step is implemented: standard tiling keybindings exist, canvas navigation
and geometry bindings are ignored while tiling, and tiling actions are ignored
on the canvas. `toggle-tiling` remains available in either mode. The default
canvas `Mod+Arrow` action centers the nearest window or anchor; the same chord
focuses columns in tiling. Camera-driven hover focus is suspended for explicit
tiling navigation, so `focus_follows_mouse` does not undo the selection. The
focus bindings avoid collisions with the existing send-window-to-output keys.
`[keybindings.canvas]` and `[keybindings.tiling]` let users assign different
actions to the same chord, with per-mode `none` masks and tap bindings. The
config guide documents the defaults and tests verify both mode filters.

Acceptance: a mode-specific chord dispatches only in its own mode, and the
toggle works in both. Automated acceptance passes, and the user confirmed the
manual multi-output workflow above.

### 6. Restore tiling topology across restarts

Store each restorable window's output name, column and stack order, width,
selected-window state and active-column state alongside its durable session
entry. On startup, associate that record with its stand-in's fresh stage ID;
when the client adopts the stand-in, the stage ID remains stable. Rebuild the
topology when the user next enters tiling, leaving canvas as the startup mode.
Ignore missing windows and outputs safely, and retain canvas geometry as the
restore rectangle.

Acceptance: a restart round-trip preserves columns, stacks, width, selection
and output ownership for restored windows; older session files still load;
missing outputs do not prevent startup or lose windows. Implementation,
focused regression coverage and manual restart testing on the user's
multi-monitor setup pass.

## Working rules

- `Stage` remains the authority for window membership and geometry. Never use
  smithay `Space` window APIs; follow `dev/docs/caveats.md`.
- Keep pure layout decisions separate from compositor protocol effects.
- Do not weaken tests or turn historical niri fixtures into a parity gate.
- Update `tiling-status.md` after each delivered change. Keep detailed session
  and test evidence there or in history, not in this roadmap.
- A roadmap entry does not authorize pushing, publishing or contacting others.
