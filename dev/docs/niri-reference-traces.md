# Pinned scrolling-layout reference traces

`src/stage/fixtures/niri-scrolling.json` contains hand-reviewed, source-derived
snapshots for niri `12de51d6a60baad8896366920278a24f17921f52`. These are not
recordings from a running niri process or a differential execution oracle.
The replay test is `src/stage/tiling_reference_tests.rs`; it runs offline against
Stage, pure geometry and `reveal`, without a display or a niri checkout:

```bash
cargo test --lib pinned_niri_geometry_and_fit_target_traces
```

For current implementation evidence and next work, see
[the maintained status](tiling-status.md). Renderer regression coverage does not
expand the reference corpus assumptions below.

## Reference assumptions

One normal scrolling layout; scale 1; zero gaps, borders, titlebars and struts;
fixed window/column widths; unconstrained client sizes; untabbed columns with
automatic height weight 1 for every tile. Focus centering is `never`, including
single-column centering. Coordinates start at `(0, 0)`. Clients have adopted the
requested geometry; there are no live buffers requiring extra reserved space.
Animations and configure transactions are outside these snapshots.

Each step asserts ordered columns, selected window and `[window, x, y, w, h]`
frame rectangles. A `camera` field is an independently supplied input for a fit
calculation; `target` is the expected output camera. These pairs are probes of
fit policy at that snapshot, not replay of the compositor's animation, anchor
translation or camera state across structural operations. Keyboard focus and
actual protocol requests remain covered separately by the headless fixture.

## Source derivation

All symbols below are in `src/layout/scrolling.rs` at the pinned revision:

| Snapshot behavior | Reference source |
| --- | --- |
| Column X positions | `column_xs`: successive widths plus gaps, starting at zero |
| Fixed width | `Column::resolve_column_width`: `Fixed` returns the requested width |
| Normal tile origins and vertical positions | `tiles_origin`, `tile_offsets_iter` |
| Equal automatic heights and integer rounding | `update_tile_sizes_with_transaction`: consume the remaining height and weight sequentially, rounding each requested client height |
| Consume first from next column | `consume_into_column`, `add_tile_to_column`, `Column::add_tile_at` |
| Expel last and retain source selection | `expel_from_column`, `remove_tile_by_idx`, `add_tile` with activation disabled |
| Column/window reordering | `move_left`, `move_column_to`, `Column::move_up` |
| Closure and new-column return | `remove_tile_by_idx`, `remove_column_by_idx`, `add_column`, `activate_column_with_anim_config` |
| Fit target | `compute_new_view_offset_fit`, `compute_new_view_offset` with zero padding and centering disabled |

Niri's horizontal offset is relative to the selected column: the absolute
camera is `column_x + offset` with a zero working-area origin. A fully visible
column leaves the supplied camera unchanged. A column wider than the viewport
returns offset zero, aligning its leading edge. Vertical fit is unchanged in
these cases because all selected tiles are inside the working area.

The corpus now has 60 snapshots across seven traces.

The structural trace uses width 100 and viewport `200 × 600`. It checks mapping,
focus, two consumes, tile reorder, two expels, column reorder, a width change,
a working-area resize and closing nonactive/trailing columns. The rounding
trace changes a three-tile column from height 600 to 4: the reference gives
`round(4/3) = 1`, then `round(3/2) = 2`, then 1. The oversized-column trace uses
width 300 in a viewport of width 200.

## First-buffer mapping boundary

The pinned reference keeps new toplevels in `unmapped_windows`
(`src/handlers/xdg_shell.rs::new_toplevel`). Initial rules, sizing and target
output/workspace information live in `window/unmapped.rs::InitialConfigureState`.
They are not columns in the scrolling layout.

In `handlers/compositor.rs`, the `is_mapped(surface)` branch removes the unmapped
entry, rechecks late floating hints, chooses activation/parent targeting and calls
`layout.add_window`. For an ordinary focused tiled window in one workspace,
`layout/workspace.rs::add_tile` handles `WorkspaceAddWindowTarget::Auto` with
`scrolling.add_tile(None, ...)`. `scrolling.rs::add_column` then inserts after
the active column. Consequently layout insertion and focus context are determined
at first-buffer mapping, not initial configure.

Three headless conformance scenarios in `src/tests/tiling.rs` now gate this:

- A is mapped; B and C receive initial configures but stay unbuffered. Neither
  creates a column or takes focus. C maps first at A's next column, then B maps
  after C: order A/C/B. C keeps its column ID and position when B appears.
- A window gets its initial configure while A is active; the user focuses B
  before its buffer arrives. The window maps after B.
- A window closes before its first buffer. Existing layout membership, selection,
  camera and target stay unchanged; the early configure waiter is retired.

These cases assume ordinary focus-on-open behavior, no parent, floating,
fullscreen, explicit target workspace/output or activation-token special policy.
They are source-derived tests, not recordings of a running niri compositor.

Do not add pre-buffer phantom columns or preserve initial configure ordering as
a parity feature: that would change the observed behavior above. Earlier roadmap
language calling for pre-buffer column ownership was too broad; retain early
sizing/protocol bookkeeping separately from mapped layout membership. Grouped
commit transactions remain necessary for already mapped windows.

## Discovered differences and limits

- Driftwm previously front-loaded rounding remainder (`2, 1, 1` for height 4),
  whereas niri rounds each remaining share (`1, 2, 1`). The replay now gates the
  corrected rounding. A separate property checks that bounded allocation uses
  the available budget up to the sum of maxima, or overflows only to satisfy
  minima. Its saved seed also covers the former stranded-space bug.
- General automatic height policies still differ. Niri allocates frame heights
  with weights and generally ignores an automatic tile's maximum height unless
  it is an exact size. Driftwm equalizes content heights and honors maxima.
  Mixed decorations, non-unit weights and non-exact maximum limits need their
  own reference cases and a deliberate sizing-policy implementation.
- Closure traces now cover next-neighbor selection for an active interior tile
  or column, returning from a newly inserted column, preserving the return on
  repeated same-column selection, cancellation after switching columns, and
  cancellation when the predecessor disappears. Stage stores the previous
  column's stable ID; it is cleared on active-column reorder, layout exit and
  removal to the left. Ordinary trailing closure falls back to the last survivor.
  Batched crash pruning chooses the next surviving original neighbor rather than
  indexing the already-shortened list with the old index; exact event-order
  comparisons for all multi-window disconnect permutations remain future work.
- `ViewOffset::stationary` saves the animation destination rather than its
  current intermediate value. `add_column` saves this offset; `remove_column_by_idx`
  restores it and applies fit again for resized columns. Driftwm now saves the
  stationary X relative to the previous column, matches a one-shot structural
  return event and computes a new camera target from the surviving frame X.
  Headless lifecycle checks cover restoration from an unfinished flight,
  width 600 growing to minimum 1150 in a viewport of width 1200, normal/crashed
  closure and cancellation after changing columns. For the resized case a saved
  offset of -100 becomes -50 to keep the right edge visible; without resize it
  stays -100. Camera position remains unchanged until animation ticks.
- JSON fit probes remain independent camera inputs. The lifecycle checks above
  are source-derived conformance scenarios in `src/tests/tiling.rs`, not a niri
  runtime oracle. Integer reveal rounding, gesture stationary offsets, batched
  enrollment, modal/layer contexts and animation timing remain outside this
  reference subset.
- Gap-aware proportions, centering, fractional scales, constraints, floating,
  fullscreen, tabs, multi-output ownership, gestures and animation/anchor state
  remain outside this corpus. Existing driftwm regression tests are not evidence
  of reference parity for these contexts.

Update the source derivation and expected snapshots together when extending the
corpus. Do not regenerate expected values from driftwm's own geometry output.
Changing the pinned revision requires a separate baseline review.


## Separate executable parser evidence

The [pinned parser corpus](niri-parser-reference-fixtures.md) now executes 16
configuration contracts against actual niri-config with recorded provenance.
It does not execute or upgrade the 60 source-derived geometry snapshots above.
The [executable geometry corpus](niri-geometry-reference-fixtures.md) separately
passed twenty-four actual niri layout contracts with nonzero gaps, borders and struts,
including static 1.25×/1.5×/2× frame rounding and fractional consume/expel/height
reset sequences plus height/width constraints, unequal Auto weights and
overconstrained height geometry, synthetic responses/refusal and dynamic bounds.
It does not retroactively upgrade the 60 source-derived snapshots into executed
reference evidence or establish a driftwm differential match.
