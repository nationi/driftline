# Pinned niri monitor navigation and view actions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source audit
adds 40 action-row links (91 scoped action consumers overall). It covers monitor
selection helpers, selected compound fallbacks, centering and available-width
expansion. No named schema rows are added. This is not executed niri/input/GPU
acceptance or full topology/view parity; driftwm runtime remains unchanged.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs)

Earlier [binding/address](niri-binding-address-audit.md),
[size/transfer](niri-size-transfer-action-audit.md),
[preset/topology](niri-preset-topology-action-audit.md),
[output identity](niri-input-output-integration-audit.md#output-identity-and-lookup)
and [focus/warp](niri-input-focus-audit.md) audits retain their limits.

## Monitor target selection

Directional helpers start from current output geometry. Left/right candidates
must have center x strictly on that side and overlap a horizontally extended
strip with current output's y/height. Up/down applies the transposed rule. The
minimum center distance along the selected axis wins among candidates. This is
not nearest Euclidean distance or arbitrary diagonal adjacency; perpendicular
strip overlap matters. Equal-center candidates are excluded. Ties, fractional
mode geometry and disconnected/stale output ordering need executable probes.

Previous/next uses sorted_outputs, wraps to last/first, and filters out current
output. With only the current output it returns None. This is cyclic ordering,
not focus history. The earlier output audit records identity sorting and explicit
position ordering; do not substitute connector enumeration or MRU order. Named
FocusMonitor uses the reference identity helper rather than these geometry rules.

FocusMonitor directional, previous/next and named branches focus the resolved
output, try centered configured focus warp, otherwise move cursor to output
geometry center, and clear on-demand layer focus. Missing target yields no
switch. Output focus can change independently of a window focus action; lock,
layer, tablet and pointer constraints retain their earlier integration gates.

## Compound local-first fallbacks

| Actions | Selected fallback behavior |
| --- | --- |
| FocusWindowOrMonitorUp/Down | Try workspace focus in that direction; only when it fails focus the resolved neighboring output. Without a neighbor, retain ordinary local focus attempt. |
| FocusColumnOrMonitorLeft/Right | Try local directional focus before output focus. Local success does not switch output. |
| MoveColumnLeftOrToMonitorLeft / Right | Try local workspace movement; on failure transfer to neighboring output. Without a neighbor, input dispatch runs ordinary local movement. |
| FocusWindowOrWorkspaceDown/Up | Try workspace focus down/up, then switch workspace on failure. This is not monitor adjacency. |
| MoveWindowDownOrToWorkspaceDown / Up | Try local reorder/movement, then move to adjacent workspace with Smart activation on failure. |

Workspace local methods dispatch to floating or scrolling according to focus.
For floating they can perform directional geometry operations, so a boundary
condition in scrolling cannot be generalized to floating. Monitor/workspace
helpers return early without an active monitor/workspace where guarded. Empty
local workspace does not itself prove a compound action is a no-op: fallback
can still switch output/workspace.

The monitor-fallback helper's bool reports whether it took the fallback path,
not a universal guarantee that a window moved or gained keyboard focus. Input
uses it to choose centered cursor/output fallback versus ordinary focus warp;
focus monitor compounds clear on-demand layer focus. Screenshot UI has separate
branches for column/window movement actions. Existing layout-native floating
and Smart-activation transfer source edges remain in the earlier audit.

Workspace up/down selection saturates/clamps at boundaries through its own
monitor methods. These compounds do not implicitly inherit workspace-auto-back-
and-forth from FocusWorkspace, nor monitor previous/next cyclic ordering. Full
workspace gesture/DnD apparent-index selection and no-output behavior still need
fixtures beyond the selected helper paths.

## Directional monitor transfers

MoveWindowToMonitor{Left,Right,Down,Up,Previous,Next} resolves targets with the same
geometry/order helpers. Outside screenshot UI it moves selected window using
Smart activation, explicitly focuses target output and uses centered focus warp
or output-center fallback. The column family uses move_column_to_output with
activation=true then focuses target. While screenshot selection owns the branch,
these move actions move that selection to a target output instead of tiling
content. This is not a statement about screenshot rendering parity.

MoveWorkspaceToMonitor in these six forms requests current workspace ownership
transfer then tries centered warp/output fallback. Unlike the named form's
conditional layout-result branch, the directional dispatch shown here does not
condition cursor handling on the transfer helper's return value. Missing
neighbor yields no branch. Whole ownership migration and hotplug/original-output
restoration remain P10/P11 acceptance. IDs, explicit workspace targets and
floating column delegation retain their separate earlier audit; reviewing six
symmetric dispatch branches does not prove every transfer lifecycle.

## Center column, window and visible group

CenterColumn on scrolling animates active column to centered view and cancels
its interactive resize. While floating focus is active it delegates to floating
window centering. CenterWindow/ById resolves the target workspace and guards the
actively moved target; in scrolling it only centers if the target's column is
already the active column. It does not first focus an arbitrary background ID.
Floating uses its window-position centering policy. Unknown mapped ID no-ops
before layout dispatch. CenterVisibleColumns is ignored while floating is active.

Scrolling group centering returns when there are no columns or focused-column
centering policy is already active. That policy means center-focused-column=always
or always-center-single-column with at most one column. It computes visibility
against target_view_pos, the camera animation destination, not current pixels.
Columns must fit completely inside working area with gaps; partially visible
columns are not included. If active column is not fully included, it returns.

For the included contiguous run, width_taken sums each width plus gap. Free space
is working_width - width_taken + gap. The new view x centers the run from its
leftmost column and working-area origin. It cancels active-column resize, animates
the offset and invokes normal active-column fit afterward. Do not assume it
centers all mapped windows or changes focus/topology merely to center them.

## Expand to available width

Expansion is ignored for floating focus, no columns, non-normal pending sizing
mode or already full-width column. Under active centering policy it toggles
full-width and cancels resize. Otherwise it examines fully visible columns using
the target camera position and gap-aware work-area boundaries, as group centering
does. If active column is not fully visible it returns.

Available growth is working_width - gap - width_taken - active_column.extra_width.
Nonpositive growth returns. If no other fully visible column was counted, it
uses full-width toggle to allow a later reversal. Otherwise it sets fixed width
to current active width plus available growth, clears preset/full-width state,
updates tile sizes and animates view so the leftmost counted column fits, followed
by active-column fit. Partially visible neighbors do not reserve the same space
as fully visible ones in this algorithm. The source notes limitations with small
fixed-size maximized windows; that comment is not acceptance evidence.

## Driftwm gaps and executable acceptance

Driftwm has canvas monitor viewports, center-nearest actions and a single-output
tiling fallback. Those are not the reference output ordering, compound workspace
fallback, group centering or expandable-column policy. Existing Partial/Missing
status is unchanged by source links. Multiple connected outputs still leave
driftwm tiling; this audit does not implement P11.

Required probes include aligned/diagonal/touching/equal-center monitors, distance
ties, sorted-order wrap and single/no output; local success versus failure in
normal/tabbed/floating/empty layouts; workspace boundaries and gestures; screenshot,
layer/lock/warp paths; active versus background center ID; partially visible
columns and exact gap boundaries; fractional work-area origins and animation
current/target disagreement; always-center/single-column policies, extra frame
width, nonpositive growth, full-width/pending-maximized and client constraints;
and interrupted transfers/reflow. These source-derived requirements did not run
as executable tests in this documentation step.

Next A0: remaining local/under-pointer focus/reorder, tabs, floating/fullscreen and
state/target action families, plus misc/root/recent-windows semantics and
independent reference fixtures. Ninety-one action source links do not close the
186-variant audit or any implementation acceptance gate.
