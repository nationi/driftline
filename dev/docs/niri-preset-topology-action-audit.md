# Pinned niri preset, maximize and column topology actions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds 23 action-row links, bringing scoped action consumers to 51 of 186. These
are action payload/consumer reviews, not new named schema fields. No niri/client/
GPU execution, complete animation acceptance or driftwm runtime change is claimed.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs)

The earlier [layout decoder](niri-layout-config-audit.md),
[lifecycle](niri-config-lifecycle-audit.md),
[binding/address](niri-binding-address-audit.md) and
[size/transfer](niri-size-transfer-action-audit.md) audits retain their limits.

## Preset width and height selection

Forward/back actions map to a shared boolean direction; ID variants first resolve
a mapped window and do nothing if absent. Layout wrappers locate its workspace,
while implicit targets use current selection. Workspace dispatch chooses floating
or scrolling for a window-specific preset. Column-width preset dispatch also
uses floating window width when floating focus is active. Scrolling window-width
presets change the containing column, using the targeted tile for content/frame
conversion; they do not create independent tile widths in a shared column.

When a stored preset index exists, width cycling increments/decrements modulo
list length. Full-width/pending-maximized column state invalidates that remembered
choice for selection. Without an index, forward scans list order for the first
resolved preset strictly greater than current size +1; backward reverse-scans
for the last preset whose resolved size +1 is strictly less than current. If no
candidate exists, forward chooses first and backward last. Comparison uses window
or tile size according to resolved preset units, not merely the stored fraction.
The one-logical-pixel tolerance handles fractional sizing. It is not nearest-size
selection, a list sort, or fixed next-index selection from an unknown size.

Scrolling width applies through set_column_width, then remembers the chosen index
and cancels interactive resize for the affected column. Floating width keeps a
per-tile index, resolves against full working width, applies window sizing and
ends that target's interactive resize. Expected-or-current sizes matter when
clients have pending replies. Fixed/proportional frame/gap units retain the
size/transfer audit; they are not universally interchangeable.

Scrolling height can convert auto-height weights before assigning a preset,
preserving apparent sizes where possible. A stored WindowHeight::Preset index
cycles unless pending-maximized; otherwise current allocated tile height is
compared to resolved/rounded/clamped candidate tile heights with the same strict
one-pixel allowance. It assigns the preset, clears pending-maximized and updates
sizes. Floating height instead uses expected-or-current window/tile height and
its per-tile index before applying floating size. Preset lists are normalized by
layout option handling; do not infer modulo-zero safety from the cycling helper
alone. Empty-list/default behavior is in the earlier layout audit.

On scrolling option update, changed width presets clear remembered index; changed
height presets convert heights to automatic and recompute sizes. This is not a
guarantee that a reload preserves a numeric index against a different list.
Full floating reload/client-refusal/constraint permutations still need fixtures.

## Reset and two maximize operations

ResetWindowHeight is ignored for floating targets. Scrolling reset in normal
display assigns Auto weight 1 to the selected/ID tile. Tabbed reset assigns Auto
weight 1 to every tile in that column, even if a particular ID was supplied, then
updates sizes and cancels affected interactive resize. Reset does not mean equal
content height for every mixed-frame/minimum-constrained window on the workspace.

MaximizeColumn toggles full-width on the active scrolling column and is a no-op
while floating focus is active. If the column is pending-maximized, its toggle
instead clears both pending-maximized and full-width; otherwise it flips the
full-width bit. The underlying width/preset state is distinct from this bit.

MaximizeWindowToEdges resolves current layout focus (or an explicit mapped ID)
and toggles pending maximized state, with a moved-target guard. A normal stacked
column with multiple tiles extracts the target into its own column on the right
before maximization; tabbed columns do not take that extraction branch. The
column maximized flag and a window's fullscreen/pending sizing mode are distinct,
so toggling reads the column flag even while fullscreen.

Floating windows cannot stay maximized in the floating layout: requesting
maximize moves the tile to scrolling and records restoration intent. Unmaximize
can return it to floating when its pending sizing mode is maximized and that
intent applies. This is not driftwm canvas fit/fill/fullscreen behavior. Complete
expanded geometry, decoration/layer stacking, restore sizes and transactions
remain P8/P9/P13 acceptance beyond this source consumer audit.

## Consume, expel and selected versus boundary tiles

Workspace topology commands below return for floating selection/target. Explicit
consume-or-expel IDs locate their own workspace and guard against operating on
the tile under interactive move. Source activity is computed for that actual
target, so background-ID operations must not be assumed to steal selection.

| Operation | Selected source and resulting topology |
| --- | --- |
| ConsumeOrExpelWindowLeft / Right (and ID variants) | If source column has one tile, remove it and append to adjacent column on that side; no neighbor means no-op. If source has several tiles, split the targeted tile into a new column on that side, even at the strip edge. |
| ConsumeWindowIntoColumn | Pull tile 0 from the next column to the end of the active column, without selecting the appended tile. At last column or fewer than two columns, no-op. |
| ExpelWindowFromColumn | Remove the last tile of active column into a new column on its right, keeping the source column active. A one-tile column is unchanged. |

These are distinct from moving the focused tile into a neighbor unconditionally.
Consume-or-expel preserves active-target intent when moving/splitting it and
carries removed width/full-width metadata when forming a new column. Left/right
paths update close-return/view offset state differently to keep the transition
anchored. Helpers remove empty columns and adjust tile selection; append versus
insert position is intentional. Expelling a selected last tile leaves selection
in the source through removal fallback, rather than selecting the new column.
Full nested close-return/camera/interruption traces still need independent cases.

## Swap behavior

SwapWindowLeft/Right returns at a missing neighbor. The actual boundary guard is
a no-op; a nearby source comment comparing edge behavior to consume/expel does
not override that code. Two single-tile columns use column reorder. Otherwise
swap exchanges the selected tiles of adjacent columns at their original tile
slots, adjusting neighbor indices if removing a single-tile source drains it.
A drained source column is recreated with the removed source's width/full-width
metadata; otherwise the exchanged tile is inserted into its source slot.

The code restores both columns' selected tile indices, animates both exchanged
tiles and activates the target column, following the original selected source
tile. It also forces alpha toward one. A documented source workaround stops
movement animations for the exchanged tile returning to the source column before
starting its new move; do not assume all previous animations survive swap.
Transaction grouping, geometry offsets, tab selection and repeated/background
operations require executable acceptance beyond this topology reading.

## Driftwm comparison and remaining acceptance

Driftwm already has bounded first-consume/last-expel topology/selection coverage,
plus focused move/expel actions and width cycling. This does not implement the
combined conditional actions, ID targeting, tabbed reset, floating presets,
reference preset search/units, maximize restoration or swap semantics above.
Existing Partial status remains Partial; no action becomes Exact through review.

Required independent probes include remembered versus unknown preset index,
unsorted lists, forward/back wrap and exact one-pixel boundaries, expected/current
size differences, fixed/proportional frames, list reload and client refusal;
normal/tabbed/ID reset; full-width versus pending-maximized and floating restore;
one/many-tile consume/split at both boundaries, explicit background IDs and
selection/close-return retention; first consume/last expel; one-to-one versus
mixed swap, drained source recreation, tabbed slots, opacity and interrupted
animations. These requirements have not been executed by this docs-only step.

Next A0: remaining directional/previous/next monitor/workspace compound fallbacks,
expand-to-available/centering and other target action families, misc/root/
recent-windows semantics and independent executable reference fixtures. Fifty-one
source-linked action rows do not close all 186 or the feature-expansion gate.
