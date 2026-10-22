# Pinned niri size, position and scoped transfer actions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 audit traces
SizeChange/PositionChange and 16 selected action consumers. Together with the
[binding/address audit](niri-binding-address-audit.md), 28 action rows now link
scoped source evidence. There are no new named schema fields: these arguments
are enum payloads in previously indexed source. No executable niri/parser/GPU
acceptance or driftwm runtime change is claimed.

## Pinned sources

- [niri-ipc/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs)
- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs)

## Size and position scalar parsing

Both FromStr helpers split an optional terminal percent sign; anything after it
is an error. The first byte of the numeric portion decides the operation: + or -
means adjustment, otherwise setting. Percent values parse as f64 percentage
points, not already divided proportions. Fixed SizeChange values parse i32;
fixed PositionChange values parse f64. Empty values fail. Neither helper trims
input or adds a separate finite/range check around numeric parsing.

| String | SizeChange | PositionChange |
| --- | --- | --- |
| 50% | SetProportion(50) | SetProportion(50) |
| +10% | AdjustProportion(10) | AdjustProportion(10) |
| -10% | AdjustProportion(-10) | AdjustProportion(-10) |
| 100 | SetFixed(100) | SetFixed(100) |
| +10 | AdjustFixed(10) | AdjustFixed(10) |
| 10.5 | Fixed integer parse error | SetFixed(10.5) |

These are source-derived expectations, not executed test results. A leading minus
cannot request an absolute negative coordinate through this string syntax; the
serialized enum SetFixed(-10) is a different interface. Scientific notation with
an internal exponent sign does not itself select adjustment. Nonfinite, overflow,
whitespace and extra-percent diagnostics still require parser fixtures.

SetWindowWidth/Height and SetColumnWidth take a string-decoded SizeChange config
argument. ID variants use the IPC conversion path recorded in the earlier audit.
MoveFloatingWindowById is config-skipped and has optional ID plus x/y PositionChange;
public IPC MoveFloatingWindow maps to it. There is no independently bindable
MoveFloatingWindow variant in this pinned config enum.

## Size units and layout dispatch

Selected width/height dispatch uses explicit ID when supplied, otherwise current
layout selection. Unknown mapped IDs are rejected before layout calls. During
interactive move, layout width/height commands return for the moved target or an
implicit target; unrelated explicit targets can proceed. Workspace dispatch
chooses floating or scrolling based on target membership/current floating focus.
SetColumnWidth goes to floating window width when floating is active. Width and
height commands without ID can instead resize screenshot UI selection in input
dispatch; ID variants do not use that shortcut branch.

In scrolling, a fixed column width is deliberately converted from the selected
window's requested content width to tile width, including frame extents. Fixed
adjustment adds to resolved column width and stores a fixed width. Proportional
set divides percentage points by 100; adjustment adds percentage points to a
proportional value or converts a fixed current width using working width, gaps
and extra size. Thus +10% is not multiplying width by 1.1. The fixed-to-proportion
path uses (current_px + gaps + extra_width)/(working_width - gaps), with a special
zero-denominator fallback. Source limits clamp fixed tile width to [1,100000]
and proportions to [0,10000]. A change clears preset/full-width/pending-maximized
state and updates tile sizes.

Scrolling height first preserves/rebalances automatic height weights as needed
because only one tile can hold a non-auto height. Fixed values refer to content
height. Proportional conversion starts with (working_height - gaps)*prop - gaps -
extra_height and converts tile height to window height. Other windows' minimum
heights/gaps limit available space, except hidden tabs don't reserve that sum.
Client maximum then minimum constraints apply, followed by [1,100000] safety
clamp; this can overflow a too-small work area rather than violate minimum size.
Preset/reset/automatic-height families still require their own audit/fixtures.

Floating width/height uses expected-or-current window/tile size. Fixed set is
content size; fixed adjustment uses saturating integer addition. Proportional
set/adjust uses full working-area extent and tile-to-window conversion, unlike
scrolling's gap accounting. It rounds/clamps content to [1,100000], applies client
min/max and requests size once, retaining/constraining the other dimension.
These formulas are not a complete explanation of client refusal, frame changes,
transactions or animated resize; existing driftwm matrices do not prove niri sizing.

## Position and stored floating state

For a floating target, SetFixed adds working-area origin to the supplied coordinate;
AdjustFixed adds to current position. SetProportion divides percentage points by
100, clamps fraction to [0,10000], multiplies working extent and adds origin.
AdjustProportion derives the current fraction relative to origin, adds points,
clamps and resolves again, using max(1,extent) for the denominator. It does not
subtract window size to make 100% align the far window edge; it positions its
origin at that working-area fraction. Actual movement has further move_to policy.

For a tiled target, this action updates stored floating position rather than
immediately detaching or moving the tile. It first uses stored/default floating
position. If neither exists, only a request setting both coordinates absolutely
can initialize it; a relative adjustment alone returns. The resolved point is
stored in working-area-relative representation for later floating use. Interactive
move checks and target workspace lookup remain at layout level.

## Workspace transfers and focus

MoveWindowToWorkspace and MoveColumnToWorkspace take a WorkspaceReference and
focus boolean property default true. Resolution may identify another output;
same-output target normalization and cursor behavior are action-specific. Window
transfer uses ActivateWindow::Smart for focus=true and No for false, not an
unconditional Yes. Smart checks whether an explicit source window was active.
An implicit window uses current source selection. Named/ID workspace references
can target another output; ordinal references carry no explicit output.

Important explicit-ID case: MoveWindowToWorkspaceById with an ordinal reference
passes the target index to the window's own monitor, which can differ from the
active monitor. Do not generalize the focus action's current-output ordinal rule
to every target action. Monitor move_to_workspace clamps index to its last
workspace and returns for the source workspace. Cross-output move_to_output
instead rejects an out-of-range explicit target workspace index in the traced
path. Missing window/output/reference and no-output guards remain at dispatch.

Column transfer preserves a whole active tiling column and uses its bool focus
policy. Source edge: move_column_to_output delegates to move_to_output(None,
output, None, Smart) when floating is active, discarding its supplied workspace
index and activate bool in that branch. Record this pinned behavior for execution
before deciding compatibility; do not describe all floating column actions as
honoring the same focus/target semantics as tiled columns.

MoveWorkspaceToIndex/ByRef converts destination with saturating_sub(1). Monitor
code validates the source index, clamps destination, returns on no change,
maintains empty-workspace boundary invariants, adjusts active index to preserve
selection, clears workspace transition and cleans up. Active and referenced
background workspaces therefore need separate reorder probes. This is workspace
reordering, not moving a window into a workspace.

## Named monitor and workspace ownership actions

MoveWindowToMonitor resolves output by the audited identity helper. Without ID,
it requests Smart movement then explicitly focuses that output and uses centered
cursor warp/output fallback; screenshot UI has a separate branch. The ID variant
locates the source window, requests Smart movement and only applies cross-output
cursor fallback if active output actually changed. Treating those as the same
focus policy would be incorrect.

MoveColumnToMonitor moves the selected column, focuses the target and uses
centered cursor fallback, with screenshot/floating branches already noted.
MoveWorkspaceToMonitor operates on current workspace; ByRef resolves the source
workspace before requesting an ownership move. Cursor adjustment depends on the
layout result. Full ownership migration, original-output restoration, empty
workspace cleanup, fullscreen/floating geometry and hotplug remain P10/P11
acceptance, not proven by these dispatch paths. Directional/previous/next monitor
selection and compound fallback families remain a separate audit.

## Driftwm gaps and acceptance

Driftwm fixed width cycling and equal-height allocation do not expose the above
change grammar or all frame/weight units. Its canvas movement, leaving tiling for
floating operations and one-output fallback do not implement these transfer
families. Existing action status remains conservative; source links do not mark
Missing/Partial actions complete.

Independent executable probes must cover numeric parsing and sign selection,
zero/negative/nonfinite/bounds, +percentage-points versus scaling, mixed frames
and nonzero gaps/struts, fixed/proportional transitions, auto-height/tab/client
constraints, pending client replies, floating expected size, stored position
initialization, screenshot/interactive contexts, missing IDs, background-ID
ordinal targets, Smart versus No, floating column fallback, out-of-range workspace
transfer versus reorder, source/target output differences and interrupted topology.
These requirements have not been run by this documentation-only change.

Next A0: preset/reset/maximize/consume/swap and directional/previous/next monitor/
workspace compound action families, remaining misc/root/recent-windows semantics,
and independent reference fixtures. The 28 scoped action rows are not the full
186-action semantic audit or acceptance gate.
