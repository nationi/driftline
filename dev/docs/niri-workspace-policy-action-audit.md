# Pinned niri workspace and mapped-window policy actions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds 24 action links (154 scoped consumers overall), with no new named schema
rows. It covers workspace up/down/move/name and opacity/urgency/dynamic-cast
policy. It does not execute niri, clients, PipeWire or GPU paths and makes no
driftwm runtime/public config change. Acceptance and remaining action scope stay open.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/tile.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tile.rs)
- [src/window/mapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mapped.rs)
- [src/screencasting/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/screencasting/mod.rs)

Earlier [binding/address](niri-binding-address-audit.md),
[size/transfer](niri-size-transfer-action-audit.md),
[navigation/view](niri-navigation-view-action-audit.md),
[lifecycle](niri-config-lifecycle-audit.md) and
[input/output identity](niri-input-output-integration-audit.md) audits retain
their target/merge/reload and acceptance limits.

## Workspace up/down and transfers

FocusWorkspaceUp/Down selects on the active monitor. In ordinary state it uses
saturating previous or clamped next index. During a DnD workspace gesture it
instead selects ceil(current_idx)-1 or floor(current_idx)+1, clamped to bounds:
the apparent workspace position matters. It calls activate_workspace even when
boundary selection yields the same index; do not claim every boundary has no
animation side effect. This is not automatic back-and-forth or cyclic wrapping.

UnderMouse variants are config-skipped. They choose output_under_cursor and call
that monitor's switch method directly, without a separate focus_output call.
They clear on-demand layer focus and invoke global focus warp, redrawing the
selected output. That does not prove the pointer's output became globally active.
Under-pointer column focus had a different workspace-under-cursor helper; these
interfaces should not be collapsed into one target-selection rule.

MoveWindowToWorkspaceUp/Down takes focus property default true, mapping true to
Smart activation and false to No. MoveColumnToWorkspaceUp/Down uses its activation
boolean; when the selected domain is floating, monitor code delegates to a single
window transfer using Smart/No. Up/down computes target from active index,
clamps and returns on source==target. Whole-column versus single-window removal,
selection and transaction behavior retain the previous transfer audit; source
comments/dispatch symmetry do not establish all client/animation permutations.

## Reorder versus rename

MoveWorkspaceUp/Down swaps the active workspace with its neighbor if possible,
maintains bottom/top empty workspace invariants, reactivates the moved workspace,
clears workspace transition and cleans up. It explicitly saves/restores previous
workspace identity around activation. This is not window transfer, and reordering
does not deliberately replace previous with the old ordinal. Empty insertion and
cleanup can shift indices, so fixtures must compare stable identity and contents.

SetWorkspaceName / ByRef first refuses a name already found with the reference
ASCII-insensitive lookup, then selects active workspace or explicit reference.
Index references use active monitor and get_mut after saturating subtraction;
invalid source index returns rather than silently selecting last. Names and IDs
search workspace identities. Assignment replaces the name, not the workspace ID.
The runtime action path does not use WorkspaceName's config duplicate decoder.

A pinned source edge deserves execution: after assigning a referenced workspace,
the empty-boundary maintenance block examines the active monitor for that
workspace ID, even if the reference targeted another monitor. Do not claim that
this path necessarily restores remote-monitor boundary invariants immediately.
An already-used name, including the target's existing name, returns before it.

UnsetWorkspaceName / ByRef resolves target then unnaming by stable ID. Workspace
unname only clears its name; monitor cleanup runs immediately only without a
workspace transition. With no outputs, an unnamed empty workspace is removed.
This is not a universal reset of saved per-workspace layout overrides. The earlier
lifecycle audit records persistence/reload limits separately. No naming or reorder
semantics here implement driftwm tiling workspaces, which remain missing.

## Opacity override and recomputation

ToggleWindowRuleOpacity selects active workspace's active window, not necessarily
whatever layer owns keyboard focus. ById searches mapped workspace windows.
Dispatch toggles only when resolved opacity exists and raw value !=1. The flag
ignores that rule during rendering; it does not write opacity into config/rules.
Tile rendering uses alpha=1 while ignored, otherwise clamps rule opacity and
interpolates toward one at fullscreen. Opening/closing/other alpha can still apply.

On changed rule recomputation, the ignore flag resets when the new opacity is
not Some(value<1). If rules compare equal, recomputation returns before that reset.
This differs subtly from dispatch's !=1 condition: raw values above one and
nonfinite comparisons should get execution fixtures rather than assumed parser
clamping. The action is layout-visible and must not stay generically Outside
just because its name doesn't start with a navigation prefix. Driftwm has mapped
opacity rules but no corresponding ignore-rule toggle policy.

## Urgency and focus

Toggle/Set/UnsetWindowUrgent take mapped window IDs, search workspace windows and
request redraw even on absent target. Mapped::set_urgent refuses true while
is_focused. A changed urgent state marks rule recomputation, so matching
is-urgent can alter appearance/policy. set_is_focused clears urgency and requests
rule recomputation when its focus state changes. Do not treat urgency as an
unconditional boolean assignment independent of focus or a force-focus action.
Background urgency, focus transitions, activation-token urgency and tab/workspace
indicators need separate executable acceptance.

## Dynamic casting target versus session lifetime

SetDynamicCastWindow uses active workspace window; ById validates a mapped ID.
SetDynamicCastMonitor takes optional output name, otherwise active output, using
the reference identity helper. ClearDynamicCastTarget selects Nothing. StopCast
takes session ID, which is distinct from window/workspace ID. Cast actions are
integration scope: target state can affect window-rule matching and capture
privacy. They cannot remain blanket Outside compositor actions.

With the screencast feature, set_dynamic_cast_target updates only existing casts
marked dynamic_target, using target-derived refresh when available. Refresh
failure warns and skips that cast's target update in this loop; do not infer a
successful switch or session teardown from that branch. Clearing retains refresh
and sets target Nothing; it does not stop the session. A real target also starts
pending dynamic casts, with preparation/format/client failure paths outside this
bounded review. 
stop_cast removes matching pending dynamic entries, disconnects matching streams
and asks the DBus session interface to stop. Missing stream and stop failures
need integration probes. Without xdp-gnome-screencast the target/stop functions
are feature-gated no-op stubs. Source presence alone does not prove runtime
feature availability or a working PipeWire session.

refresh_mapped_cast_window_rules considers any current cast with matching Window
target, regardless of cast.is_active, when setting is_window_cast_target. A
changed flag marks rule recomputation. This is not simply focused window or only
a stream currently producing frames. Dynamic target changes, privacy block-out,
output movement/disconnect and feature-off behavior remain in-scope acceptance.

## Scope corrections and remaining acceptance

The inventory corrects seven formerly generic Outside rows: two opacity actions
are Integration/Missing, five cast actions Integration/Unverified. This is a
source-backed scope correction, not newly implemented support. Other previously
reviewed action statuses remain unchanged. Urgency remains Unverified integration;
workspace actions remain Missing where driftwm lacks those policies.

Independent executable requirements include DnD apparent-index and boundary
activation, active versus pointer output, Smart/No transfer, previous identity
through empty workspace reorder, same/duplicate/remote rename and cleanup;
opacity ignore/recompute with raw boundary values and other alpha; focused/
background/absent urgency; feature-on/off casting, dynamic versus fixed sessions,
refresh failure, clear versus stop, pending streams, rule target flags, capture
privacy and output lifecycle. No such runtime result is claimed by this review.

Next A0: remaining process/session/debug/capture/MRU/internal action scope,
misc/root/recent-windows semantics and independent executable reference fixtures.
154 source-linked actions do not close 186 variants or implementation acceptance.
