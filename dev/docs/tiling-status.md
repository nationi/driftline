# Tiling development status and handoff

Read this together with [the parity roadmap](niri-tiling-parity-plan.md).
This is the maintained continuation point for future agent sessions. Verify the
working tree and history; update this file as implementation advances.

## Objective and decisions

- The user wants full behavioral parity with niri's tiling, beyond a similar
  horizontal layout. See the roadmap's scope and final acceptance criteria.
- Keep the existing canvas mode as the default and preserve its behavior.
- Reference revision: `12de51d6a60baad8896366920278a24f17921f52` in `/tmp/niri`.
  If that temporary checkout is gone, recover the pinned revision using the
  source URLs in the roadmap. Do not silently substitute current upstream HEAD.
- Workspaces/overview and shared-canvas versus independent-output ownership
  still need explicit architectural decisions at their roadmap milestones.
  Do not assume they are implemented or silently remove them from the goal.
- Continue implementation autonomously when the user asks to continue. Do not
  repeatedly ask for approval of ordinary local edits and checks. Publishing,
  pushing and contacting others require appropriate user authorization.

## Implemented baseline

| Commit | Delivered behavior |
| --- | --- |
| `2b61c79` | Experimental single-output mode; columns, focus, width cycle, mapping/close, canvas restoration and session fallback |
| `0fc1309` | Multiple windows per column, merge/split actions, vertical navigation, selection memory, height constraints and live-frame reservation |
| `41034d2` | Pinned niri action/schema inventory and reproducible source-index generator |
| `53b5903` | Stable column identity, structural reorder and camera/pointer anchor preservation |

The subsequent implementation adds stable `ColumnId` values, separate selected
windows and active column identity, column left/right and tile up/down reordering,
and horizontal view-anchor preservation. Merge retains the destination column ID;
split allocates a fresh ID. Selection, reorder and replacement preserve identity;
clearing the layout does not recycle IDs. Camera, in-flight target and pointer
coordinates translate together when left-side structure changes.

[The reference inventory](niri-tiling-inventory.md) indexes all 186 Action
variants and 318 named schema fields. Regenerate it with
`python3 dev/scripts/niri-parity-inventory.py /tmp/niri`; the generator refuses a
different reference revision. This is a source index with conservative statuses,
not completion of the semantic audit.

Implementation locations:

- `src/layout/scrolling/` — pure single/grouped geometry and reveal calculations.
- `src/stage/tiling.rs` — membership, ordering, selection and original canvas rects.
- `src/state/tiling.rs` — mode lifecycle, reflow, configures and view integration.
- `src/state/tiling_policy.rs` — exhaustive action policy and shared transition
  gate for input, IPC, direct window operations and invalidated layouts.
- `src/config/{types,parse}.rs`, `src/input/actions.rs` — public binding actions.
- `src/tests/tiling.rs`, session-restore tests — real headless Wayland scenarios.
- `config.reference.toml`, generated `docs/config.md`, `docs/tiling.md` — user interface.

The current implementation supports one output. It exits tiling for many
incompatible canvas actions and when another output appears. Tabs, snapping
physics, layout-native drag/resize, stationary floating and retained tiled
fullscreen are missing. These exits are temporary fallbacks, not parity.

Current action names: `toggle-tiling`, `focus-column left/right/up/down`,
`cycle-column-width`, `move-into-column left/right`, `expel-from-column`.
Structural actions: `move-column left/right`, `move-tiled-window up/down`.
Reference topology actions: `consume-window-into-column`,
`expel-window-from-column`; these retain selection in the destination/source
column, respectively.
Cardinal `center-nearest` also follows the tiling structure while in this mode.

## Next work: finish P0/P1 before expanding interaction

1. Audit inventory fields against custom decoders and public option names,
   nested values, defaults and ID-targeted action availability. Do not infer
   exact compatibility from the generated schema index.
2. Extend the new hermetic reference corpus beyond its zero-gap, undecorated,
   fixed-width subset. Include more selection histories, constraints,
   gaps, proportions and real camera/animation state, not just fit probes.
3. P1 now has stable column/layout IDs, separate tile/column sizing inputs
   and a shared transition gate. Extend transitions as floating and retained
   fullscreen become supported; broad mode-exit fallbacks still exist.
4. Extend P2 to grouped bounded commit transactions before layout-native
   interactive resizing. Early size offers and unmapped mapping-time ownership
   now have coverage. Do not reserve phantom columns before first buffer.
   Then P3 settings and P5 physics.
5. Extend the new reference consume/expel topology subset. Existing focused
   move/split commands are preserved. First/last/index, swap, animation and fullscreen/tab
   contexts of structural operations remain outstanding.

Important differences to preserve explicitly while adding reference actions:

- Our merge action moves the focused window into its neighbor. The distinct
  reference consume action now pulls the first window of the next column.
- Our short expel action splits the focused window. The new reference expel
  action splits the last window while keeping the source active. Directional
  consume-or-expel variants remain missing.
- Current presets are hardcoded 1/3, 1/2, full width. Reference defaults and a
  separate full-width toggle still need implementation.
- Current height allocation equalizes content heights; reference automatic
  weights and frame-space allocation are not implemented.

Do not break documented existing actions merely to reuse reference names.

The source-derived `pinned_niri_consume_expel_topology_trace` test cites the
pinned scrolling-layout methods and checks ordered membership, active selection,
source selection and column identity. It is not a trace captured from running
niri. The headless reference-action scenario additionally checks keyboard focus
and requested grouped/split sizes. The additional JSON corpus now covers
geometry and fit probes for a controlled subset; full view/animation fixtures
remain outstanding. Random membership histories include both actions.

## Reference geometry and fit corpus

[Reference-trace methodology](niri-reference-traces.md) documents the new offline
JSON corpus, its pinned source derivation and limits. Sixty snapshots
cover map/focus, consume/expel, tile/column reorder, fixed-width changes,
working-area resize, interior/nonactive/trailing closure, new-column return
and oversized-column fit.
Each checks ordered membership, selection and exact rectangles; explicit camera
inputs additionally check fit targets. These are source-derived expectations,
not recordings or full camera-lifecycle replay.

The corpus exposed front-loaded auto-height rounding. Allocation now rounds
sequentially among the remaining unconstrained tiles, matching niri's unit-weight
integer sizes in this subset. A conservation property also found a pre-existing
case where combined min/max limits stranded a usable pixel; bounded water-level
allocation fixes it and the failing seed is retained. A real-client test checks
`201, 202, 201` configure heights and matching positions for three grouped windows
in 604 available pixels.

Closure now selects the next surviving tile/column, or the preceding one at the
end. A newly opened column remembers its predecessor until another column is
selected, the active column is reordered, or a column to its left disappears.
Repeated selection in the same column preserves this memory. Invariants check
that the return target is a live immediate predecessor. Clearing the layout
clears the memory. Crash pruning handles several dead members without using an
old index against an already-shortened list.

Tiled closes bypass canvas MRU/spatial focus-follow. After normal unmap or the
dead-client sweep, reflow restores structural keyboard focus when the previously
selected tile disappeared and no other live focus intent exists. A deliberate
empty-canvas focus clear is respected. Headless tests cover interior tile and
column closure, return/cancellation for newly mapped columns, client crash,
background closure and intentional focus clearing. Broader modal/layer and
multi-disconnect reference cases remain outstanding. Max-height handling and
mixed-decoration weighting still differ.

The close return now saves the stationary horizontal camera offset relative to
its predecessor, using the pending animation target when present. Reflow consumes
an explicit stage close-return event once, verifies both column IDs and restores
the saved offset from the surviving column's current frame X. `reveal` adjusts
the target for changed width/usable area. This starts a flight from the current
camera instead of teleporting it, and subsequent no-op flushes retain the flight.
The stage event survives no-op liveness sweeps and same-column focus sync, but is
cleared by explicit layout exit, new enrollment or incompatible structural/focus
changes. Navigation to eligible tiling windows now only raises/focuses them;
reflow owns the camera and does not inherit a flight to temporary canvas placement.

Headless tests cover an unfinished previous flight, changed minimum width,
normal and crashed-client closure, repeated flushes, and cancellation after
switching columns. Remaining limits: integer reveal rounding, gesture-specific
stationary offsets, multiple newly enrolled windows in one reflow and complete
reference animation timing. The JSON corpus remains fit probes; these lifecycle
checks are in the compositor fixture, not recordings of niri.

## P2 configure-lifecycle foundation

`layout/scrolling/transaction.rs` tracks a generation keyed by the stable layout
ID and a revision of the desired pure placements (before live-frame reservation).
Each live tile retains only its latest tiling-issued serial, requested content
size, original deadline and progress (`Pending`, `Applied`, `TimedOut`). A new
layout drops incompatible/dead requests; unchanged requests keep their deadlines
even when columns move. Exiting tiling drops the batch and cancels its timer.

Reflow reads the committed cached xdg serial, separately from live geometry.
Only the current generation, a matching-or-newer serial (wraparound aware) and
matching geometry can complete a pending request. Ack-only and empty commits
with old geometry do not complete it. A per-view calloop timer wakes reflow at
the earliest deadline; waiting ends after 300 ms without requiring another
client event. Timer callbacks are scoped to the layout ID, all-complete batches
remove the timer, and timed-out requests do not reopen on late commits. The
`tiling_pending_configures` debug counter exposes outstanding waits.

This tracker is bookkeeping and readiness protection; the separate pre-commit
group described below now blocks related buffer commits. Positions and view updates now wait for buffer application or the deadline
as described below; live-frame reservation still
protects against overlap, including after timeout. Client buffers are not rejected
for being late, and unchanged desired sizes are not resent on timeout. Rounded
or refused sizes currently time out; they are not treated as exact adoption.
Smithay still owns its protocol configure history; bounding our records does not
bound a client's unacknowledged protocol history.

Pure tests cover superseded generations/serials, a different layout ID, serial
wraparound, independent completion, exact deadlines, terminal timeout and pruning.
Headless tests cover ack before commit/buffer, old-size commits after a newer
request, unchanged-request suppression and late replies after mode exit. The
next P2 steps are rendering/snapshot coordination and extending
supersession/reference coverage. Extend the P0
semantic audit and reference cases in parallel; do not mark P2 complete yet.

## Initial sizing before the first buffer

Ordinary windows created while tiling is active now get a content-size offer in
the existing first configure, after rule and decoration resolution. The preview
uses the pure one-tile column calculation, shared `ColumnWidth::default`, client
min/max and `mapping_chrome`; it does not send a separate initial configure or
focus/map an unbuffered tile. The normal first-buffer path still establishes
canvas placement and enrolls the window after the active column.

Each preview retains its issued serial, size and original timestamp in the
owning `TilingView`. It shares the 300 ms deadline timer and pending counter.
On enrollment, a matching target transfers to the generation-scoped batch with
its original deadline; changed constraints/decorations get a fresh normal request.
Dead windows and windows mapped into excluded roles retire their preview record.
Exiting before the first buffer cancels the offer with a client-choice size
configure and drops the waiter, so a late buffer cannot recreate tiling state.

Already-parented dialogs, widgets, pinned/fullscreen/fit windows, explicit
size/position/output rules and potential suspended-window adoptions keep their
existing initial sizing path. Matching pending relaunches are only inspected,
not consumed early. A late parent declaration is rechecked at first buffer.
When a client declines the offer, canvas restoration saves its first committed
size rather than the compositor's pending offer. If it honors the offer, the
accepted size is the available fallback; its unreported preferred size is unknown.

Headless scenarios cover CSD, per-window SSD titlebar/borders, undecorated
windows, min/max, one resolved initial configure, no pre-buffer focus/membership,
size stability after adoption, explicit rules, dialogs, a late parent change,
exit before first buffer and declined-offer restoration. Topology is still
buffer-driven: several unbuffered windows do not reserve columns in creation
order, and early requests do not yet participate in grouped commit blockers.
This completes ordinary initial size offers, not all P2 transaction semantics.

## Mapping-time ownership: reference correction

A source audit of pinned niri's `handlers/xdg_shell.rs`, `window/unmapped.rs`,
`handlers/compositor.rs`, `layout/workspace.rs` and `layout/scrolling.rs` confirms
that an initially configured, unbuffered window is outside the scrolling layout.
Only `is_mapped(surface)` routes it into `layout.add_window`; ordinary automatic
placement inserts after the column active then. A delayed B configured before C
can therefore appear after C if C maps and takes focus first. No early column
ID, blank strip space or focus is reserved for B.

The earlier suggestion to implement pre-buffer column reservation was incorrect
for the parity goal. An uncommitted prototype was discarded; no reservation
runtime change was committed. Keep `InitialTilingConfigure` as sizing/serial
bookkeeping, and allocate column identity when the first buffer enrolls the tile.
[The reference methodology](niri-reference-traces.md#first-buffer-mapping-boundary)
records derivation and assumptions. New real-client tests cover reversed first-
buffer arrival, focus changes while unbuffered and closure before the first
buffer, including geometry, identity, selection, camera and waiter cleanup.
This is a reference-conformance and roadmap correction; current runtime behavior
already matched these controlled cases. Mapped-window buffer grouping is now
connected as described below; position/rendering coordination remains pending.

## Layout identity and sizing inputs

`Stage::begin_tiling` returns an opaque `TilingLayoutId`, reused for the whole
mode lifetime even when no windows remain. `clear_tiling` ends that lifetime;
reentry allocates a new ID without recycling old values. Valid tile insertion
also begins a layout for pure-stage callers. The compositor starts the layout
at mode entry, stores its ID in `TilingView` and rejects a mismatched view during
reflow. These IDs are stage-local and do not implement configure generations,
workspace ownership or persistence.

The pure geometry API takes `Column { width, tiles }` and
`Tile { id, constraints, chrome }`. Width belongs to the column exactly once;
client limits and decorations remain per window. A minimum-limited tile can
widen the column while a maximum-limited neighbor remains narrower. Height
allocation and pixel rounding retain the existing behavior. The old singleton
`arrange` API and window-shaped `Column` input have been removed; singleton
regression tests construct one-tile columns through test helpers.

Stage and real-client tests cover empty-mode entry, last-window removal, a new
window in the same layout, explicit exit and fresh identity on reentry. A direct
geometry scenario covers shared width with conflicting per-window limits and
mixed decorations. This finishes the remaining P1 identity/input refactor; the
P0 semantic audit and broader reference geometry/view coverage still need work
before P2.

## Central transition gate

All automatic mode exits go through `prepare_tiling_operation`; only explicit
toggle-off and the gate call the canvas restoration routine directly. The action
match is exhaustive so adding an action requires a deliberate transition choice.
This consolidates existing behavior; it does not add floating or retained
fullscreen support.

| Operation | Current policy |
| --- | --- |
| Structural tiling actions, cardinal center-nearest | Preserve tiling |
| Launch, close, keyboard layout and trackpad settings | Preserve tiling |
| Other canvas actions, including diagonal center-nearest | Restore canvas before dispatch |
| New normal window or excluded dialog | Preserve mode; existing enrollment rules apply |
| Explicit placement of an already tiled window | Restore all canvas rectangles before placing it |
| Direct move, resize, rule placement, fit/fill, pin, fullscreen | Restore canvas before operation |
| Incompatible output, zoom, membership or invalid geometry | Restore canvas during reflow |

Headless regression scenarios additionally cover direct fit without action
dispatch and remapping a tiled window, including restoration of other windows,
requested sizes and the saved view. Existing mapping/dialog and output tests
cover the preserve/invalidation paths.

## Validation evidence

For the latest implementation batch:

- `dev/scripts/ci-local` passed every stage, including ignored tests and isolated soak.
- `PROPTEST_CASES=4000 cargo test --lib --quiet` passed all 366 library tests.
- The headless tiling/session coverage includes 46 scenarios, including direct
  operation restoration, reorder and
  preservation of camera/pointer coordinates and camera targets.
- The reference inventory regenerates byte-for-byte from the pinned checkout.
- The user tried the initial single-window-column version nested and reported
  that it worked. Multi-window/reorder behavior has headless coverage; real
  trackpad feel and current graphical behavior still need manual validation.

The mapping-boundary audit changed tests/docs only and reran the full CI gate.
The deeper library-property evidence above applies to the unchanged runtime
library; the discarded reservation prototype is not validation evidence.

Documentation-only handoff changes do not constitute a new runtime validation.
For new implementation, run relevant tests, the deeper property pass when
appropriate, then the full documented CI gate before claiming completion.

## Working-tree and repository precautions

At this handoff, the following changes predate tiling and are unrelated:

- `src/protocols/output_management.rs`
- `src/tests/output_management.rs`
- The `mod output_management;` addition in `src/tests/mod.rs`

Preserve them. Do not stage the entire tree or include them in tiling commits.
Tiling commits exclude them; the full working-tree CI includes
them. Recheck the current diff rather than assuming this list is exhaustive.

User fork: `https://github.com/nationi/driftwm`. At this handoff `origin` still
points to `https://github.com/malbiruk/driftwm.git`; do not push to it by default.
No push has been performed for this task. Check remotes before any authorized push.

A temporary manual-test config was created at `/tmp/driftwm-tiling.toml`; it may
have disappeared or been changed. Its original bindings use Ctrl+Shift+F8 for
toggle, F9/F10 for horizontal focus, F11 for width and F12 for a terminal.
It does not yet include the multi-window actions. Do not overwrite a user's
running-session config just to continue development. See `docs/tiling.md` for
current binding examples; use explicit `--backend winit` for a nested test.

The initial proposal is historical context. The parity roadmap and this status
are the continuation sources; keep them consistent as milestones are delivered.

## Pre-commit group foundation

`src/layout/scrolling/commit_group.rs` adds a separate barrier owner for immutable
configure groups. All participants share one 300 ms deadline. A new-buffer
pre-commit with the current generation and a sufficiently recent ack removes
that participant; client-selected geometry does not hold up the group. Removal,
cancellation and owner drop release existing blockers without discarding client
commits. Completion is terminal and only the transition reports a wake-up.
Five deterministic library tests cover peer coordination, timeout, cleanup,
generation rejection, wraparound and isolation.

Mapped-toplevel pre-commit hooks now install the group's barrier for new-buffer
commits whose acknowledged serial meets the participant request. Ack-only and
empty commits cannot mark a participant ready. All arrivals, timeout, dead-stage
participant removal, supersession and mode exit release queued buffers. Wake-ups
use calloop idle callbacks and `CompositorClientState::blocker_cleared`, avoiding
re-entrancy while Smithay constructs its transaction. Timer insertion failure
also releases the group. The existing per-view timer includes the group deadline.
Superseding groups carry only still-waiting requests, update replaced serials and
keep the earliest inherited deadline rather than extending a stalled peer's wait.
Initial unbuffered windows remain outside membership and these groups.

Six real-client scenarios cover cross-client waiting, completion, timeout without
a peer commit, disconnect, mode exit, client-chosen sizes and supersession. The
exact-size post-commit diagnostic tracker remains separate: a refused size can
release a group while its diagnostic request eventually times out.

Positions and view updates now wait as described below. Supersession releases
already queued older buffers, which can expose an interim size, as also noted
in pinned niri's `take_pending_transaction` comments. Rendering snapshots and
animations remain outstanding; do not mark P2 complete or claim frame atomicity.

## Progress estimate

The user-facing estimate is roughly 15–20% of the complete behavioral goal.
This is an engineering estimate, not a count of checked boxes: phases have very
different sizes and the plan defines no weighted percentage. P1's single-container
foundation is implemented; P0, P2, P3, P4 and P5 have partial evidence and behavior.
P6–P15 still need their main implementations/audits. Baseline functionality and
scaffolding in a phase do not satisfy that phase's completion criteria.

Validation for pre-commit integration: `PROPTEST_CASES=4000 cargo test --lib --quiet`
passed all 366 tests; the tiling-filtered binary run passed 44 tests, including
six new group scenarios. `dev/scripts/ci-local` passed every stage, including
ignored tests and the isolated soak. No graphical validation was performed.


## Deferred placement and view application

Reflow still changes stage topology/focus and sends configures immediately, but
now defers all tile positions, camera reveal/close return, anchor translation and
pointer compensation while the group or its wake-up callbacks are pending.
Presentation requests separately track new-buffer commits actually applied by
Smithay, before `on_commit_buffer_handler` consumes the current buffer assignment.
An ack or empty commit with the same serial cannot satisfy this check. Thus a
second blocker (for example DMA-BUF readiness) cannot make positions apply early.
Presentation shares the group's deadline, including when a later group supersedes
it, and retains outstanding applied-buffer waits independently of pre-commit
arrivals. Expiry uses live-frame reservation to avoid overlap with refused or
still-delayed geometry. Mode exit discards deferred placement with the view.

After all client wake-ups unwind, reflow recomputes current placements rather
than replaying an obsolete saved rectangle list. Wake-up counters are scoped to
the layout lifetime, so old callbacks cannot settle a newly entered mode.
Logical selection is tracked separately from the last presented tile: closing
or crashing the selected window still restores structural focus while placement
is pending. Deferred close-return intent survives until placement is applied,
and a focus change cancels incompatible return intent.

Two additional real-client scenarios cover a pending merge and anchor/camera/
pointer preservation through another surface blocker, an earlier ack-only commit
and a superseding resize. Existing group tests now also assert unchanged positions
while waiting, application after timeout, and latest placement after supersession.
Earlier geometry/camera tests explicitly deliver replies before asserting settled
coordinates; refusal tests release all participants before checking reservation.

The deferred-placement batch did not freeze imported textures. The snapshot
foundation below now preserves eligible windows' old pixels; general
frame-atomic rendering, niri animation physics and broader rapid supersession,
remap and cancellation traces remain outstanding.

Validation for deferred placement: all 46 tiling-filtered binary tests passed.
`dev/scripts/ci-local` passed fmt, locked check, strict all-target clippy, all
regular/ignored tests and the isolated soak after the final supersession fix.
The library was unchanged in this batch; the previous 4000-case property evidence
still applies. Graphical/rendered-frame validation remains outstanding.


## Frozen tiling presentation images

Before issuing a batch of size requests, reflow now bakes visible eligible tiled
windows into owned canvas snapshots. The normal output composer substitutes each
snapshot in its window's existing z-order slot until deferred placement can
apply. Superseding batches keep the first picture while still waiting. Snapshot
opacity and chrome are captured with the old geometry; these images do not tick
as closing animations. Completion, timeout and mode exit drop them, and dead
stage IDs are pruned. `tiling_snapshots` in debug counters exposes retained images.
A failed capture or missing backend preserves the live rendering fallback.

This reuses the existing closing-image baker and shares chrome policy with
closing snapshots. Pixel comparison found a pre-existing baker coordinate bug:
border/shadow shader areas were still surface-local while the framebuffer origin
was shifted to the union bounds. Rebasing those areas aligns chrome with content. The baked shadow also now
uses the border's outer edge as its casting body, matching live composition.
The shared change affects closing snapshots too, without changing their timing.

Four new GLES scenarios check old pixels while one new buffer already applied
and another is held by a separate surface blocker; image release after all
buffers, the deadline and mode exit; and shadow alignment. The no-shadow frame
comparison is exact. The shadow case allows one byte of rounding error from the
additional offscreen composition. GL scenarios take `gl::lock()` and self-skip
only when EGL/GLES is unavailable, following the existing pixel-suite convention.

Limits: windows with active window animations use live rendering. Popup trees
and background blur are included by the follow-ups below. Capture is limited to currently drawable windows; moving the view
can reveal uncaptured windows. Isolated capture paths still capture live content.
This is a best-effort frozen presentation foundation, not the complete niri
render/animation model or a guarantee across all rendering paths. Extend animation integration, fractional-scale tiling pixel cases, supersession and
remap traces before closing P2 or P13. Input continues to use stage/client
geometry, as with the existing compositor animation policy.

Validation for frozen presentation images: all seven GLES pixel scenarios passed
on the available surfaceless Mesa renderer, including four new tiling scenarios.
`dev/scripts/ci-local` then passed every stage, including strict all-target
clippy, ignored tests and the isolated soak. The pure layout library is unchanged.
No interactive nested-session or hardware presentation test was performed.


## Popup images in pending tiling presentations

A frozen window now stores separate, unclipped images for its mapped popup
surface trees. Popup origins use `stage_position + popup_offset - popup_geometry.loc`,
matching the live composer and pinned niri's popup placement. Each popup keeps its
own content bounds, so it can extend beyond the parent and its declared geometry.
Images preserve `PopupManager` order above the parent image; nested popups remain
above their ancestors. A failure to capture any popup falls back to the whole
window's live rendering, avoiding a partially missing menu. Unimportable buffers
retain this fallback, including single-pixel buffers unsupported by the existing
texture capture path.

The composer compares the current mapped popup identities, offsets and geometry
against the snapshot. New, destroyed, unmapped or repositioned popup trees cause
live rendering for that window. Renderer buffer presence is checked separately
from role/surface lifetime, so a null-buffer unmap cannot leave an old menu on
screen merely because its protocol objects still exist. Frozen images remain
bounded by the presentation deadline and are released with the parent snapshot;
`tiling_snapshots` now counts parent and popup images, not just window entries.

Five new pixel scenarios cover overhang and nonzero geometry offsets, nested
popup ordering, destruction, null-buffer unmap and opening a menu after freezing.
The fixture Popup type can now attach real SHM pixels using the same format and
lifetime rules as its existing window/layer helpers. Comparisons of unchanged
popup scenes are exact byte comparisons on the complete live frame.

Active window animations remain on live rendering. Blur integration follows
below. Popup movement uses
a safe live fallback rather than animated/rebased snapshot transitions. Further
P2/P13 work still needs animation integration, fractional-scale tiling scenes
and broader rapid supersession/remap tests; this does not complete niri parity.

Validation for popup presentations: all 12 GLES pixel scenarios passed on Mesa,
including five new popup cases. `dev/scripts/ci-local` passed every stage after
the final changes, including strict all-target clippy, ignored tests and the
isolated soak. The pure layout library was unchanged. Interactive session and
fractional-scale popup-presentation validation remain outstanding.


## Live frost beneath frozen tiling images

Blurred tiled windows now participate in frozen presentations. `SnapshotBlur`
retains their canvas position, client geometry, optional SSD bar height, effective
window blur policy and client-supplied regions. It converts those saved inputs
into a normal-layer blur request at the current camera/zoom/output scale. Region
clipping and whole-window fallback follow the live path, including suppressing
client-only blur when every region clips out. The backdrop remains live; it is
not captured into the frozen window image.

The composer submits snapshot elements to the existing blur pipeline with measured
bucket ranges. Shadows of blurred snapshots are drawn separately at their saved
geometry, below the frost, and excluded from the alpha-mask element range. Baking
shadow into the foreground would both pollute the rounded-corner mask and darken
frost with shadow above it. Nonblurred snapshots retain their existing baked
shadow. This does not allocate an extra mask snapshot. When images are released,
the blur geometry generation advances even if no window moved, so a refused
resize cannot leave an old alpha mask behind after the snapshot disappears.

Three deterministic renderer unit tests cover client-region clipping, CSD geometry
offsets, SSD bar offsets and HiDPI mask coordinates. Two new GLES scenarios at 2x
compare the complete live frame before and during a delayed presentation: one
plain translucent window and one with border, rounded corners and shadow. An
independent blocker holds another participant while the frosted window's new
buffer has already applied. Comparisons allow one/two byte rounding respectively;
after release the snapshot disappears and the new picture is shown.

Next: integrate active window animations with the frozen presentation, extend
fractional-scale and client-protocol blur-region pixel scenarios, and broaden
supersession/remap/cancellation traces. The existing blur engine's documented
alpha-only mask invalidation policy otherwise remains unchanged. No unconditional
frame-atomic rendering or complete niri animation parity is claimed.

Validation for frozen blur: three new renderer unit tests and both new GLES
scenarios passed. `dev/scripts/ci-local` passed every stage after the final
shadow-order and mask-invalidation changes, including all ignored pixel tests,
strict all-target clippy and the isolated soak. The pure layout library was
unchanged; no interactive hardware/nested-session validation was performed.


## Fractional-scale frozen presentation regression checks

Four new GLES scenarios extend the pending-presentation comparisons to 1.25x
and 1.5x. They cover plain content, unclipped nested popup trees with nonzero
window geometry offsets, shadows, and translucent frost with/without rounded
border chrome. One client's new buffer applies while an independent blocker
holds its peer; the complete frame is compared with the pre-transaction frame,
then snapshots must release when the last buffer applies.

The tests exposed three differences absent at integral scales. The shared
closing-image baker now expands its bounds to a logical grid whose translation
is integral at the bake scale (when representable within the fractional-scale
protocol's 120-step denominator), and rebuilds border/shadow uniforms in the
rebased coordinate system. Merely translating a shader area kept uniforms made
from negative half-pixel coordinates, whose rounding differs after translation.
This shared baker correction also applies to closing images.

Pending tiling shadows now always use the saved-geometry live shadow path,
including nonblurred windows, preserving output-space rounding and avoiding an
extra offscreen sampling pass. Popup placement rounds the saved root origin and
popup offset independently, matching the live composer instead of rounding their
sum. Parent geometry and popup identities remain checked before substitution.

Coverage is controlled single-output, stationary-view, unit-zoom CSD content
with the existing integer logical sizes. It does not establish parity for odd
fractional buffer extents, SSD chrome, negative/fractional camera movement,
output scale changes while pending, active window animations or every capture
path. Client-protocol blur regions still need their own pixel scenarios. Do not
mark P2/P13 or general fractional-scale parity complete.

Validation: all four new GLES tests passed on the available Mesa renderer.
`dev/scripts/ci-local` passed fmt, locked check, strict all-target clippy,
all regular/ignored tests (1423 binary cases plus the library/integration suites)
and the isolated soak. An initial run used stale output-management build
artifacts and failed three existing protocol tests; baseline isolation and a
forced rebuild of that unchanged module resolved the mismatch, and the complete
fresh run passed without excluding tests. Existing unrelated output-management
edits were verified byte-for-byte unchanged against the baseline copy and were
not incorporated into this change. The installed compositor
binary was not replaced and the running session was not restarted.

Next: integrate active window animations with frozen presentations, expand
supersession/remap/cancellation traces, and continue the P0 semantic audit.


## Animation start after frozen capture

Open and geometry animation entry points now release an existing tiling snapshot
only after their visibility/grab eligibility checks accept the animation. The
parent and all its popup images are dropped together; other windows' snapshots
and the pending configure/presentation group remain in place. Releasing the
picture invalidates blur geometry and marks outputs dirty, so the saved frost
mask cannot outlive the switch to animated live content. Ending/cancelling the
animation does not resurrect the removed picture.

Two GLES regression scenarios start an open fade or a position chase after a
new buffer has already applied while another client's buffer remains blocked.
Both include nested popup images and shadows. They failed before the fix with
all four images retained; after the fix only the peer's image remains. Complete
frame comparisons show the animation immediately and show further motion after
an injected 16 ms tick. Cancellation keeps the peer frozen, and releasing its
buffer still drains the presentation normally.

This closes an inconsistency in the existing live fallback: previously only
animations active at capture time were excluded, while a later animation was
hidden by the captured picture. It does not animate the frozen texture or
preserve old-buffer continuity when switching to live content. Full P2/P13
animation integration still needs a shared transform/content model (including
popups, shadow, blur and geometry freezes), interruption/supersession traces,
and manual validation. The running compositor binary remains unchanged.

Validation: both new GLES scenarios passed on Mesa. `dev/scripts/ci-local`
passed fmt, locked check, strict all-target clippy, all regular/ignored tests
(366 library and 1425 binary cases plus integration suites), and the isolated
soak test. The pure layout library was unchanged. Existing output-management
changes remain outside this commit; no push or compositor restart was performed.

Next: preserve frozen-content continuity through active animations, then expand
rapid supersession/remap/cancellation traces and continue the P0 semantic audit.
