# Pinned niri local focus and layout-state actions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds 39 action links (130 scoped action rows total). It traces selected local
focus/reorder, tabs, floating, fullscreen and overview consumers. These enum
payloads add no named schema rows. No niri/input/protocol/GPU execution or driftwm
runtime/config change is claimed; full interaction acceptance remains open.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs)
- [src/window/mapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mapped.rs)

Earlier [navigation/view](niri-navigation-view-action-audit.md),
[preset/topology](niri-preset-topology-action-audit.md),
[focus](niri-input-focus-audit.md), [layer/overview](niri-layer-overview-audit.md)
and [binding/address](niri-binding-address-audit.md) audits retain their limits.

## Local focus and under-pointer selection

Scrolling column left/right returns at strip boundaries without wrapping. Window
up/down clamps its selected tile index within the active column. Top/bottom picks
that column's first/last tile. DownOrTop/UpOrBottom tries directional focus first,
then the corresponding end fallback. Four Up/DownOrColumnLeft/Right actions try
vertical tile focus, then horizontal column focus if vertical movement failed;
they do not add the output/workspace fallbacks from the earlier compound actions.

Workspace dispatch uses floating directional focus/topmost/bottommost methods
when floating is active. The four vertical-then-column actions in floating only
try floating vertical focus, not scrolling column fallback. This is context
policy, not a universal column-index operation. Input local focus actions try
configured focus warp, clear on-demand layer focus and request redraw.

FocusColumnLeft/RightUnderMouse is config-skipped. Dispatch selects the workspace
under the pointer with extended bounds, resolves its stable workspace ID, and
calls that workspace's directional focus directly. This is not the same as first
activating whichever output/workspace the pointer is over. Cursor warp still
uses the global active focus helper. Workspace hit resolution includes overview/
transition policy; complete under-pointer behavior needs fixtures before parity.

CloseWindow sends close to current layout focus; no focus means no request.
Like the earlier ID close audit, it is a polite protocol request, not forced
process termination or proof of immediate layout removal.

## Local reorder

MoveColumnLeft/Right uses scrolling column reorder; boundaries return without
movement. Moving a column preserves its selected tile, cancels its interactive
resize, adjusts camera offset and animates moved/intervening columns. First/Last
uses that reorder helper. Floating workspace dispatch instead interprets left/
right as floating movement and returns true, while First/Last is ignored.

MoveWindowUp/Down swaps selected scrolling tile/data with its neighbor, updates
selected index and movement offsets; boundaries are no-ops. Floating workspace
uses directional window movement and reports success. Input can route ordinary
move directions to screenshot selection instead of layout. Do not infer a tile
reorder or monitor/workspace fallback solely from action names.

## Tabbed display transitions

ToggleColumnTabbedDisplay flips Normal/Tabbed; SetColumnDisplay takes the exact
normal/tabbed enum string and returns for an unchanged mode. Workspace returns
for floating focus. Scrolling guards empty columns, cancels resize, updates
column display/geometry and recomputes tile sizes. Tab-indicator placement can
change column extents immediately.

When switching to non-tabbed with more than one tile, it explicitly clears
fullscreen and maximized state using the active window. This is not a purely
visual hide/show switch. Tab selection, height reset, activated hints, insertion
and move/resize transactions keep their previously audited context dependencies.
Full tab render/input/client/animation acceptance remains P7/P13 work.

## Floating membership and focus

ToggleWindowFloating chooses explicit ID or current window and finds its
workspace; unknown mapped IDs no-op in input dispatch. MoveWindowToFloating/Tiling
are set operations: if already in the requested membership, workspace returns.
FocusFloating/FocusTiling only request a focus-domain switch as needed; the
shared switch helper returns if either domain is empty, otherwise flips its
floating-active state. They do not create windows or force an empty-domain focus.

Normal workspace toggle removes/re-adds the tile between layouts, retaining
removed width/full-width metadata. It changes active focus domain only if the
target was active, so a background-ID toggle is not inherently a focus steal.
On entry to floating it uses stored/default position; otherwise it derives one
from rendered position plus (50,50), or zero offset under center-focused-column
Always, clamps to working area and stores relative position. It stops previous
movement then animates from old to new rendered position. Returning to scrolling
uses the existing add-tile placement path, not necessarily the original column.

During interactive move, layout toggle has a separate branch that flips the
moving tile's intended floating state rather than using workspace removal. Entry
requests stored floating or expected/default window size with min/max handling,
animates opacity toward one and ends workspace DnD scrolling. Return to tiling
animates toward interactive-move alpha and holds it. Set operations respect that
moving state. Full size/position restore, drag insertion and cancellation need
independent protocol/input fixtures; the normal toggle alone does not prove them.

## Real versus windowed fullscreen

FullscreenWindow/ById toggles real fullscreen for layout focus or mapped target.
Interactive moving target is guarded. Scrolling compares pending column flag;
if entering from a normal stacked column with several tiles, it extracts target
into its own column to the right. Tabbed columns use their existing column.
It cancels resize and updates sizing/indicator geometry. Fullscreen and maximized
pending column state remain distinct.

A floating target enters scrolling for real fullscreen and records restoration
intent on normal-to-expanded transition. On exit it can return directly to
floating if restore intent applies and pending-maximized is false; a transition
toward maximized deliberately defers that return. This is reference layout-native
restoration, not canvas fullscreen exit. Complete output ownership, geometry,
configure ordering and popup/layer behavior remain P8/P9/P11/P13 acceptance.

ToggleWindowedFullscreen/ById first removes real fullscreen when current pending
sizing is fullscreen, then toggles the window's pending windowed-fullscreen flag.
Mapped::request_windowed_fullscreen changes protocol Fullscreen/Maximized hints
and marks configure needed; it does not itself perform monitor-sized placement.
Pending versus committed flags are distinct: acknowledged changes are committed
against configure serial on a later client commit. A generic fullscreen-unset
path can first clear pending windowed fullscreen and return. This is not an
alias for the real fullscreen toggle or a synchronous actual-state flip.

## Overview action state

ToggleOverview flips desired overview_open and animates from current progress
toward 0/1 using overview-open-close, then propagates monitor state. OpenOverview
and CloseOverview are idempotent against desired state and return whether they
changed it; input redraws conditionally for those set operations. Closing desired
state can coexist with nonzero animated progress. The earlier overview audit
records zoom clamp/render/focus differences; canvas zoom is not this workspace
Overview. Gestures and interrupted animations still need executable acceptance.

## Driftwm gaps and required acceptance

Driftwm has partial local navigation/reorder and close behavior, but no complete
tabbed/floating/workspace overview policy. Fullscreen and free manipulation leave
the current tiling session; that compatibility fallback is not niri
membership preservation. Existing action status remains Partial/Missing/Divergent;
source links do not imply these features were implemented.

Required executable probes include empty/one/many columns and tiles, ordinary/
wrap/compound boundaries, floating and under-pointer targets during overview,
background IDs, layer/cursor policy, reorder view anchoring and client stalls;
normal/tabbed/fullscreen/maximized transitions; active versus background floating
toggle, stored/default positions and interactive-move size/alpha/cancellation;
real/windowed fullscreen serial/commit/unset/restore sequencing; idempotent
Overview open/close and interruption. No such runtime test result is claimed here.

Next A0: workspace up/down/move/name actions, opacity/urgency/cast and remaining
integration/internal action scope, misc/root/recent-windows semantics and
independent executable reference fixtures. 130 action source links do not close
186 variants or feature-expansion acceptance.
