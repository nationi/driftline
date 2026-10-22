# Tiling development history

Historical evidence, investigations and decisions, preserved from the former
status file. Statements such as “next”, “pending”, test totals, remotes and
progress estimates describe their entry’s time and may be superseded below.
For current instructions use [the maintained status](tiling-status.md),
[roadmap](niri-tiling-parity-plan.md) and [TTY guide](tty-preview.md).
Temporary logs and local launchers referenced here may no longer exist.

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


## Translating frozen content during a pending presentation

Position-only canvas chases started after capture now retain the old tiling
picture. Eligibility is deliberately narrow: no size request (including one
later discarded by the resize threshold), fullscreen role, explicit frame seed,
or inherited open fade; the seed must match the current client dimensions.
Other animation starts keep the explicit live fallback described above.

The captured parent and popup textures receive the existing render-element
translation after their stationary output-space rounding. The saved shadow
uses the same physical affine through the live shadow composer. Frost translates
its already snapped mask rectangle in physical pixels; translating a logical
origin before truncating it made the mask trail fractional animation motion.
The backdrop itself remains live. The stage, configure group, input geometry
and sizing authority are unchanged, and old texture dimensions remain fixed
even after the client's newly sized buffer has applied.

The new regression scenes compare against the same position animation rendered
with old live content before the configure batch. They include nested popup
trees, border/shadow chrome, translucent frost and rounded corners. They also
check an injected motion tick, retained old pixels after a new buffer applied,
interruption without a frame jump, cancellation back to the saved resting
picture, and release when the independently blocked peer finally commits.
Resize/open regression cases retain their live fallback checks.

Scope remains a stationary camera and unit zoom. This does not yet capture
windows already animating before the batch, animate frozen resize/open/fade
content, provide release-time geometry continuity, or establish full P2/P13
parity. Popup changes still use the existing live fallback. Additional scale,
geometry-freeze, mode-exit/timeout, remap and rapid-supersession combinations need
further coverage before that guarantee can expand.

Validation: two new GLES scenarios passed with/without shadow/rounded chrome.
Parent/popup comparisons cover 1x, 1.25x and 1.5x; frost comparisons cover 2x,
1.25x and 1.5x. Exact comparisons are used without shadow; existing one/two-byte
offscreen tolerances apply to shadow/frost. The fractional-motion tick also
checks mask displacement and unchanged mask size directly. An initial full gate
found that the updated resize fallback test incorrectly expected immediate
motion during its intentional size hold; only move/open cases now require an
immediate animated tick. The complete final `dev/scripts/ci-local` run passed
fmt, locked check, strict all-target clippy, regular/ignored tests (366 library
and 1427 binary cases plus integration suites), and isolated soak. The pure
layout library was unchanged. No manual nested/hardware validation, installed
binary replacement, session restart or push was performed. Existing unrelated
output-management changes remain outside this batch.

Next: extend the retained-picture model to capture during active translations
and test timeout/exit/supersession during motion, before implementing frozen
resize/open/fade transforms and release-time continuity. P0 semantic audit and
broader remap/cancellation traces remain unfinished roadmap work.


## Capture during an active translation and motion-time cleanup

An already moving window can now join a frozen tiling presentation. The
animation state exposes a narrow eligibility query: canvas space, normal role,
no pending size request, stale buffer, start hold or open fade; the visual size
must equal the committed client size and the frozen picture must be neither
bare fullscreen nor pinned. Capture additionally excludes live resize-crossfade
overlays. Unsupported animation pictures keep the live fallback.

The retained texture is still baked at its stationary stage coordinates, then
receives the current render translation through the previous batch's shared
physical transform. Thus capture does not cancel/reseed the chase or bake its
current displacement into the picture twice. Visibility and flattening scale
use the animated visual rectangle. A window still visible while moving toward
an offscreen destination therefore remains eligible and bakes at the visible
output's scale. Tiling entry continues to cancel canvas animations as before;
the new coverage exercises a later grouped reflow while tiling is already active.

Five new GLES scenes cover capture mid-chase with nested popup trees and shadow
at 1x/1.25x/1.5x, a visible chase with an offscreen destination, a superseding
width request with another applied buffer, deadline release and mode exit.
A sixth scene covers mid-chase frost capture with/without rounded chrome/shadow
at 2x/1.25x/1.5x. Complete pre/post-capture frames keep the existing exact or
one/two-byte offscreen tolerances. An independent surface blocker keeps the peer
pending after the moving client's new buffer applies. Supersession retains the
first picture even after a newer buffer applies, and release drains all images.
A headless policy test rejects opening, resize, scaled-seed and screen-space
entries while accepting a plain canvas chase before/after a tick.

`refresh_tiling_at` provides the reflow's transaction clock, with ordinary calls
using `Instant::now()`. Deadline pixel tests now advance that clock and drain
normal client wakeups instead of sleeping. Renderer capture timestamps and the
real calloop timer keep their existing clock paths; this does not introduce a
second production clock or change the shared deadline policy.

Limits remain: frozen resize/open/fade transforms, release-time size continuity,
SSD, view/zoom/scale transitions, popup-only visibility outside the parent,
remapping/disconnect amid motion and hardware motion quality. The new traces
cover controlled scenes and do not close P2/P13 or the broader parity roadmap.

Validation: the 23-case GLES render-pixel suite passed, including the five new
motion/capture/cleanup scenarios; the new frost scene passed all six scale/chrome
combinations. The final `dev/scripts/ci-local` passed fmt, locked check, strict
all-target clippy, all regular/ignored tests (366 library and 1434 binary cases,
plus integration suites), and isolated soak. An initial clippy pass requested
`if let` instead of checking `Option::is_some()` then unwrapping in the new
fixture helper; the final gate includes that correction. The pure layout
library is unchanged. No installed binary replacement, session restart, push or
manual nested/hardware validation was performed. Existing output-management
edits remain outside the commit.

Next: implement frozen resize/open/fade transforms and release-time geometry
continuity, with interrupted/superseding requests and refused sizes. Extend
motion-time remap/disconnect and changing-view/scale cases; continue the P0
semantic audit before treating any renderer subset as complete niri parity.


## Opening scale and opacity over retained content

An open animation started after a tiling snapshot exists now keeps the parent
and popup images. Opening progress scales the captured content dimensions around
their original centre and applies the same affine/alpha to parent, menus,
saved border and shadow. A position-only chase can inherit the fade without
unfreezing content or restarting progress. The committed live buffer can have a
new size; opening still uses the retained picture's old dimensions. Cancellation
returns to the retained resting picture, and the group/deadline/exit still own
image lifetime. A size/frame-conversion animation keeps its existing live fallback.
Capture while an open fade is already active remains unsupported.

Snapshot chrome now saves the border separately and disables its baked shader,
as was already done for the shadow. Pixel comparison exposed up to 14 bytes of
alpha error when a border baked with opaque content was faded as one texture:
the overlapping edge must blend each part independently, as in live rendering.
The output composer draws the saved border using the same affine and counts it
inside the frost-mask range, above the separately excluded shadow. Static and
translation paths use the same border policy.

The snapshot baker accepts an optional physical affine, keeping closing
snapshots on their existing default path. Frost transforms its snapped bounds
and client regions through that affine. Region coordinates are transformed in
absolute output space then rebased to the new mask; a region shrinking below a
physical pixel remains explicitly empty instead of enabling whole-window frost.
Two deterministic unit tests cover scaling and the empty-region case.

Opening start, cancel/end and nonzero ticks invalidate retained fade masks. This
also covers `animation_scale = 1.0`, where alpha changes without a different mask
rectangle. It does not change the general live blur engine's alpha-only cache
policy: reference pixel scenes explicitly rebuild their live oracle masks at
each tick to compare against fresh rendering. Output redraw scheduling continues
to use the existing animation driver, with no extra focus or input mechanism.

The upgraded opening pixel scene compares old live content before the batch
against the retained picture at progress zero and two injected 16 ms ticks. It
includes nested popups and tests inheritance into a chase without a frame jump,
cancellation and normal buffer release at 1x/1.25x/1.5x, with/without shadow.
Additional GLES cases cover opening-time deadline/exit cleanup, rounded frost
with/without border/shadow at 2x/1.25x/1.5x, and constant-size opacity-only frost.
The pending client's new buffer has applied while its peer is independently
blocked. Comparisons retain the existing exact or one/two-byte tolerances.

Scope remains stationary view, unit zoom and controlled CSD scenes. SSD, active
fade capture before a batch, resize/crossfade integration, changing outputs/view,
client-protocol region pixels, supersession during a fade and release-time size
continuity still need work. This is partial P2/P13 integration, not parity.

Validation: opening comparisons and cleanup passed on Mesa, including six
scale/shadow combinations for nested popup content, six for frost, and a
constant-scale fade. The final `dev/scripts/ci-local` passed fmt, locked check,
strict all-target clippy, every regular/ignored test (366 library and 1439 binary
cases plus integration suites), and isolated soak. The two new mask-region unit
tests passed in that gate. The old opaque-bake alpha error was fixed by separate
border composition rather than widening pixel tolerances. Cancellation assertions
read synchronous state before pumping the real event loop so CPU readback time
does not accidentally turn them into 300 ms deadline tests; explicit deadline
cases continue to use the injected reflow clock. The pure layout library was
unchanged. No installed binary replacement, session restart, push or manual
nested/hardware validation was performed; unrelated output-management edits
remain outside this batch.

Next: capture during an already active open/fade, test fade supersession and
client-protocol blur-region pixels, then integrate retained resize/crossfade and
release-time geometry continuity. P0 semantic audit, motion-time remap/disconnect
and view/scale transitions remain outstanding.


## Paused native NVIDIA preview investigation (2026-10-19)

The user explicitly paused work to continue later. Do not infer that the native
rendering problem was fixed. Installed compositor/session were not replaced.
The isolated preview was closed and its clean config restored on pause.

User observation: severe artifacts immediately after preview startup, before
entering tiling. Controlled live reloads confirmed: opaque/no-effects clean
preview works; compositor opacity 0.88 without blur works; enabling blur alone
with shadows off reproduces artifacts. This is not evidence against only the
new tiling snapshots: it reproduces in canvas before F2.

Uncommitted investigation edits: `src/backend/winit.rs` and
`dev/docs/smithay-api.md` (plus this handoff). Preserve unrelated existing
output-management changes. Composition now occurs before window buffer age
acquisition. Smithay pinned GLES `Bind<EGLSurface>` only creates a target;
`Renderer::render` makes it current. `make_window_current` starts/finishes an
empty frame to restore the window before age query and again before swap after
capture passes. This removes repeated `EGL_BAD_SURFACE` errors on the user's
NVIDIA RTX 3060 / driver 615.78.08, but the user explicitly confirmed that visible
blur artifacts STILL REMAIN. Calling backend.bind() alone had not fixed either.
Do not present these edits as a fix for the user's visible problem.

The final investigation build and `dev/scripts/ci-local` passed all stages:
log `/tmp/driftwm-ci-winit-current-20261019.log` (1439 binary tests plus suites
and isolated soak). No new automated backend test was added; the native bug is
outside the surfaceless Mesa coverage. No investigation commit/push was made.

Preview launchers/configs are in `/home/nationi/driftwm-previews/7e32bf5/`:
`run.sh` takes an optional config path; `run-clean.sh` selects config-clean.toml
(opaque kitty --config NONE, no shadow/blur, normal animation). Default run.sh
still selects original blur-heavy config. Runtime and D-Bus are isolated per
launch. Latest preview socket was
`/run/user/1000/driftwm-preview.8xv31z/driftwm/ipc-wayland-1.sock` and is now closed;
do not reuse it. Never issue quit to the real desktop's socket.

Evidence files: `preview-before-egl-fix.log`,
`preview-blur-before-current-fix.log`, latest `preview.log` in that preview dir;
`/tmp/driftwm-host-visible.png` and `/tmp/driftwm-nested-visible.png`. The host
screenshot includes private desktop content; keep local and do not publish.
A fresh screencopy from the nested compositor looks substantially cleaner than
the presented host window, whose top contains stale/fragmented decorations.
This suggests investigating window presentation/damage versus fresh offscreen
composition, but that cause remains UNCONFIRMED. Next experiment considered:
force full redraw for the nested blur scene to test buffer-age/damage history,
while preserving idle behavior in the final solution. Do not silently disable
blur and claim the renderer fixed. Resume troubleshooting before extending
features or asking the user to replace the installed compositor.

## Resumed blur investigation: niri pyramid (2026-10-19)

User resumed and requested an existing blur implementation. A forced full redraw
was very slow and still showed artifacts; release mode with normal damage was
also unusable. Neither is an accepted fix. Normal buffer age is restored.

Adapted the Dual Kawase texture pyramid and tap coordinates from official niri,
`src/render_helpers/blur.rs` at the pinned reference commit. Each downsample
level now has its own half-size texture, reused until size/pass count changes;
upsampling writes back through those levels to the root. Unlike the previous
full-size ping-pong subrectangles, texture filtering cannot reach stale storage
outside a mip. Constant per-level offset and destination-based downsample taps
match niri. Smithay still owns framebuffer/program state; existing driftwm mask,
crop, backdrop and cache integration remain. This is an adaptation, not niri's
complete framebuffer-effects pipeline or its wallpaper-only xray mode.

All 15 blur pixel tests passed on the actual NVIDIA RTX 3060, including the new
incremental/full-frame equivalence test for normal/flipped output transforms.
Log: `/tmp/driftwm-niri-blur-pixels.log`. Surfaceless coverage does not establish
native winit presentation correctness or interactive performance. User validation
of this pyramid remains pending; do not claim the native artifacts fixed.
Full gate log: `/tmp/driftwm-niri-blur-ci.log`; release build log:
`/tmp/driftwm-niri-blur-build.log`. Preview launcher now selects target/release.
Installed compositor remains untouched; unrelated output-management edits are
still outside this investigation. Next: validate the isolated blur-only preview
before proceeding with tiling features or installing a compositor binary.

Validation completed: full `dev/scripts/ci-local` passed all stages, including
strict all-target clippy, included GL tests and isolated soak. Release build
succeeded. Opened blur-only isolated preview runtime `driftwm-preview.D01Wnu`;
startup has no EGL_BAD_SURFACE or bind/submit errors. Nested screencopy
`/tmp/driftwm-niri-blur-nested.png` shows three terminals with blur; host-visible
motion/artifacts still require the user's reply to the pending feedback question.
No investigation commit made before that validation.

## Direct TTY validation confirmed (2026-10-19)

The user reports the latest release runs excellently when launched directly
with `--backend udev` from another TTY. The same release's nested winit blur
preview still has severe artifacts/lag. Treat this as a backend-dependent
reproduction; the precise nested presentation defect remains unresolved. Do not
attribute success solely to the pyramid: the old blur was not compared under
identical direct-udev conditions. The TTY probe is not full tiling parity or a
complete manual acceptance pass for every animation/scale case.

Prepared `/home/nationi/driftwm-previews/7e32bf5/run-tty.sh` and
`config-tty-blur.toml`; launcher isolates runtime, D-Bus and durable session file,
uses logind and refuses desktop terminals/root. Current main desktop is on TTY2;
Ctrl+Shift+F8 quits the probe, Ctrl+Alt+F2 returns. Config and shell syntax pass.
Installed binary remains unchanged. Prefer direct TTY for further visual checks;
track nested rendering separately rather than blocking tiling on it.

Next delivery: review and checkpoint the blur adaptation/backend changes in
separate focused commits, preserving unrelated output-management work. Resume
remaining P2 retained-picture lifecycle: capture during active open/fade,
supersession and client blur regions, then resize/crossfade/release continuity.
P0 custom-decoder/reference fixture audit remains an earlier unfinished gate;
finish that evidence before claiming parity or expanding advanced interactions.

## Retained opening capture and release geometry animation (2026-10-19)

Checkpointed the GPU investigation independently: `2016574` adapts niri's
Dual Kawase pyramid; `5a122bb` restores the winit EGL target and records the
regression/evidence. Neither claims to fix nested NVIDIA presentation. User
confirmed direct-udev TTY runs well. Unrelated output-management edits remain
outside these commits and the current batch.

This batch accepts captures during already active opening scale/fade, including
an inherited fade on a safe, unscaled canvas translation. Drawability follows
the current opening visual. Retained parent/popups/chrome/frost continue the
same animation after an early peer applies its new buffer. Pixel comparisons
exercise nested menus, shadows and 1x/1.25x/1.5x; frost tests also cover a pure
opacity fade. Existing resize/stale-buffer/frame-conversion exclusions remain.

On grouped presentation release, Stage gets the new placements atomically;
render geometry is seeded from each retained picture (or its active chase) and
animates toward the committed live size/position without issuing another size
configure. Structural anchor compensation translates the seed with the camera,
keeping screen-space continuity. Full-speed effects skip this animation.
Opaque old content is baked before any early peer can replace its textures, then
handed to the existing resize crossfade at release. The overlay direction is
resolved against the actual committed size, including client-chosen sizes.
Only compositor opacity 1 plus complete declared opaque-region coverage at both
ends allows that overlay; undeclared/translucent content uses geometry animation
without stacking two pictures and increasing opacity. A general premultiplied
old/new texture blend for transparent content is still missing. Some CSD opaque
regions are deliberately corner-trimmed, so these conservatively skip the
content overlay too. Opening fades retain their progress and skip the overlay.

New controlled regressions check old content and old visual geometry on the
release frame, movement on the next tick, overlay/animation cleanup, and stable
opacity for a translucent release. Capture regressions cover already active
opening/fade. Existing motion tests now explicitly cancel the new release leg
before installing their independently controlled translation setup.

Scope is partial P2/P13: capture during an already active resize/crossfade still
falls back to live rendering, and popup resize behavior, SSD/fractional view
transitions, motion-time remap/disconnect, client-protocol blur regions and full
frame atomicity remain. P0 custom-decoder/reference fixture audit is still an
unfinished gate before broadening interaction or claiming parity.

Validation logs: `/tmp/driftwm-active-captures.log` (five capture tests),
`/tmp/driftwm-release-blur.log` (16 blur pixel tests), and final full gate
`/tmp/driftwm-tiling-release-ci-final.log`. Final release build:
`/tmp/driftwm-tiling-release-build-verified.log`. Do not substitute the earlier CI
log, which predates the final opaque-region safeguards. Manual validation of
this new release transition has not yet been performed; use the existing TTY
launcher after the final build succeeds. Installed binary/session are untouched.

Final validation completed successfully: `dev/scripts/ci-local` exited 0 with
366 library and 1445 binary cases plus integration suites and isolated soak.
Final release build succeeded (43.77 s). Changed popup membership also prevents
release-time resurrection of a snapshot that had already fallen back to live
rendering. No manual test of this transition batch or installed binary update
was performed. Next: direct-udev checks of F2/F5, then capture and continuity
across an already active resize/crossfade; transparent content blending remains
a separate requirement rather than an accepted approximation.

## Direct TTY tiling binding diagnosis (2026-10-19)

User reports only F1 works in the direct preview. The actual log explicitly
rejects F2 because two outputs are active: HDMI-A-1 (GIGA-BYTE GS27FA) and
HDMI-A-2 (Dell S2721HGF). Tiling currently requires exactly one output, so the
F3–F11 layout commands cannot take effect. Do not call this a keyboard parsing
failure or silently remove the multi-output safety guard. F12/quit are independent
of tiling; the multi-output restriction alone does not explain those failing.

Added optional `--drm-connector NAME` for isolated udev runs, filtering both initial
connector enrollment and hotplug. No argument preserves normal multi-output
behavior. Winit rejects the option; an unavailable connector fails startup.
Updated the external `run-tty.sh` to select HDMI-A-1 by default; argument HDMI-A-2
selects the Dell instead. The installed compositor and permanent output configs
remain unchanged. Direct preview must restart to use the backend selection.

Validation logs: `/tmp/driftwm-single-output-ci-verified.log`,
`/tmp/driftwm-single-output-build.log`. CLI reference regenerated through
`UPDATE_CLI_DOCS=1 cargo test docs_cli_md_is_up_to_date` and config validation
passed. Native one-output validation is pending;
this enables the existing single-output tiling, not roadmap P11 ownership.

Single-connector batch validation completed: full ci-local passed, release build
succeeded, launcher shell syntax/config checks passed. Installed binary unchanged.

## Premultiplied resize blending and repeated resize capture (2026-10-19)

The user confirmed the previous direct-udev single-connector preview looked good
and authorized the next batch. Installed compositor/session remain unchanged.

Replaced source-over resize overlays with one mixed content element. Old and
current surface-tree bakes are rendered into a reusable ABGR8888 target with
additive premultiplied weights summing to one; compositor/opening opacity is
applied once on presentation. This supports arbitrary pixel alpha, compositor
opacity and corner-trimmed/undeclared opaque regions; the previous opaque-only
guard is removed. GL blend state is restored to Smithay source-over even when
a draw fails. Failed blends retain live rendering instead of stacking alpha.
The current content bake is cached by mapped tree order, surface IDs, commit
counters, viewports, subsurface offsets, buffer scale/transform, geometry,
flatten scale and clip policy. Client content changes invalidate it; progress
changes reuse both current bake and mixed allocation. Popups and chrome retain
their own z-order/opacity; mixed content replaces only the body tree. Like the
existing resize bake, this crops to geometry even for bare clients' overhangs.

Resolved normal canvas geometry chases (no pending size request, stale buffer,
start hold, waiter, pending view or fullscreen/pin picture) can now be captured
mid-resize/crossfade. Capture uses the currently mixed pixels, avoids applying
corner clipping twice, stores the unrounded current visual geometry, and stops
the old geometry/crossfade after the new picture is safely baked. Any inherited
opening fade keeps its progress. Superseding transactions hold that picture;
release seeds geometry and content from it, even for a peer whose native size
was unchanged. Live translations retain the prior moving-picture behavior.
Unresolved resize holds and screen/fullscreen/pinned conversions still use
live fallback. Popup membership changes keep the prior live fallback policy.

The repeated-resize cleanup test exposed stale window animations/crossfades
surviving tiling exit: viewport cancellation was not window cancellation.
Exit now cancels each tiled client's window animation/crossfade before restoring
canvas size and placement. This also drops the new cached textures promptly.

New/expanded GPU evidence on NVIDIA RTX 3060:
- Pixel alpha 96 -> 160 with compositor opacity 0.6 is checked against explicit
  premultiplied RGBA values at 1x/1.25x/1.5x, both at release and mid-transition.
  A further blue buffer commit verifies current-bake cache invalidation.
- Repeated translucent resize compares capture and release frames (max 2 bytes),
  verifies a stable held frame, a third transaction's timeout, then exit cleanup.
- Rounded frost/shadow/border capture during resize matches the live frame at
  1x/1.25x/1.5x without widening the existing two-byte tolerance.
- All 29 render-pixel tests passed before the final timeout/exit additions;
  strict all-target clippy passed before the final policy assertions.

Final gates: `/tmp/driftwm-premultiplied-resize-ci-verified.log`, release build:
`/tmp/driftwm-premultiplied-resize-build.log`. Targeted logs:
`/tmp/driftwm-alpha-blend-pixels.log`,
`/tmp/driftwm-repeated-translucent-resize.log`, `/tmp/driftwm-repeated-frost.log`.
Manual direct-udev validation of THIS batch is pending; use run-tty.sh with the
single-connector filter, and do not substitute the broken nested NVIDIA preview.

Scope remains partial P2/P13, not niri parity. Next acceptance work: already
active unresolved resize/degrade paths, inherited opening during resize
supersession, popup resizing and live fallback, SSD/release chrome continuity,
changing view/scale/output and motion-time remap/disconnect. The P0 semantic
inventory/custom-decoder audit remains an unfinished gate before expanding
interaction. No push or installed binary replacement is authorized by this log.

All 17 blur pixel tests now pass (`/tmp/driftwm-premultiplied-frost-suite.log`).
Five existing blur assertions expected content to change instantaneously on
release; they now check advancement on the next explicit animation tick, which
is the new intended behavior. Existing byte tolerances are unchanged. Final CI
uses the verified log above; the first CI log contains these stale assertion
failures and must not be reported as a pass.

Final verified gate exited 0: fmt, locked check, strict all-target clippy,
366 library tests, 1447 regular/ignored binary tests plus isolated soak (1448
total), and all integration suites. Release build succeeded (48.85 s).
No installed binary replacement, session restart, push or native validation of
this batch was performed. Existing run-tty.sh selects the updated release build;
close any older preview before restarting it to test repeated F5 and mid-resize
F2. User-owned output-management files remain unstaged.

## Menus and delayed resize responses (2026-10-19)

User confirmed `2f1e5b4` works in the direct-udev preview and authorized this
batch. Do not replace the installed compositor or use nested NVIDIA presentation
as the acceptance environment.

Popup changes previously discarded the whole retained rendering branch,
revealing an early parent's new buffer. Retained popup stamps now include their
mapped surface tree commits/views; membership, geometry, movement or content
changes switch ONLY popup rendering to live at the saved parent origin. Parent
content/chrome/frost stay held. Unchanged popup images retain their original
identity/capture behavior. Removed the release-time skip that had assumed popup
changes meant the parent had resumed live rendering. Menus are never resurrected
when a group releases; current live popup membership is authoritative.

Popup animation is separate from content resize stretching: it maps the live
or captured content origin to its animated location, retaining native popup
size, canvas zoom and opening scale/alpha. This applies to the normal live and
retained branches. Popup updates/dismissal/new menus now have pixel checks that
also assert the early green parent buffer remains hidden. Menu dimensions are
checked through release and the next tick at 1x/1.25x/1.5x. Opening/translation
and fractional popup capture regressions still pass without tolerance changes.

Capture eligibility now includes a normal canvas resize still in its start hold
(no parked view, waiter, fullscreen or pin). The group takes over the safely
baked picture; the old geometry request/crossfade is cancelled while opening
fade progress survives. A real stale reply for the superseded configure is
allowed to commit but remains hidden until the latest group is ready. Degraded
stale/capped content remains excluded: its body and frame need distinct capture
geometry. Policy checks explicitly distinguish held from degraded state.

Added a one-shot late response claim to ConfigureBatch. Only a matching/newer
serial after TimedOut can claim its requested size; it never changes progress,
adds a deadline or reopens a group, and a new request resets the claim. In the
root pre-commit path, a settled tiled window with such a late new buffer arms
the existing bounded normal resize hold before stash_resize_content captures
its outgoing pixels. The existing commit resolver starts the crossfade on
application; a blocked/refused buffer keeps the existing bounded hold/degrade
cleanup. Active animations/opening/crossfades, full-speed effects and rendererless
fixtures keep their original paths. No extra configure is sent. This is not a
new indefinite late-buffer cache. A repaint after completion cannot restart it.

Targeted evidence: `/tmp/driftwm-popup-parent-pixels.log` (31 render pixel tests
before delayed-path additions), `/tmp/driftwm-delayed-held-pixels.log` (held
resize plus stale serial and open menu), `/tmp/driftwm-late-buffer-pixels.log`
(post-deadline commit geometry/content continuity). New pure tests cover
pending/stale serial refusal, single consumption, no deadline reopening and
replacement by a newer request. Strict all-target clippy passed.

Final gate: `/tmp/driftwm-menus-delayed-ci.log`; release build:
`/tmp/driftwm-menus-delayed-build.log`. Updated direct TTY release is selected
by the existing single-connector run-tty.sh. Installed binary/session and
unrelated output-management edits remain untouched. Native checks for this
batch are pending. For a real popup test, Dolphin is installed; launch it from
a terminal INSIDE the isolated TTY preview, open a context menu and use F5.

Remaining P2/P13 acceptance includes degraded capped capture, mixed-scale/view
and output changes, SSD/release chrome continuity and motion-time disconnect
and remap. P0 decoder/fixture semantic audit remains an earlier unfinished gate;
this batch does not claim niri parity or implement multi-output tiling.

Verified final gate exited 0: fmt, locked check, strict all-target clippy,
368 library tests, 1451 binary tests plus isolated soak (1452 total), and all
integration suites. Release build succeeded (1 minute). No installed binary
replacement, native session restart, push or manual validation of this batch
was performed. The user's prior successful TTY report covers `2f1e5b4`, not
these menu/late-response changes. Unrelated output-management files remain
outside this commit.

## Native menu confirmation and preview reference (2026-10-19)

After testing the updated direct TTY preview, the user reported that the menu
stays in place without lag. This manually validates that popup scenario for
`059556e`; it does not validate synthetic delayed-client cases, all chrome
transitions or multi-output behavior. The earlier pending-native statements
above describe the state at commit time.

Added [a local TTY preview cheat sheet](tty-preview.md), verified against the
existing launcher and preview config: startup, output selection, exact keys,
return/exit and a short visual check. This is documentation only; no compositor
code, installed binary, running session or user config was changed. Checked
with `git diff --check`; no runtime tests were needed. Remaining work and the
unfinished P0 gate above are unchanged.

## Documentation reconciliation (2026-10-19)

At the user's request, separated the former 1163-line status into a concise
current handoff and this preserved history. Added a development-document index,
updated roadmap baseline/next batch, corrected outdated mapping/transaction and
popup fallback descriptions, linked the general user guide and local TTY keys,
and marked the initial proposal historical. Corrected current remote ownership
and removed historical progress percentages from the current handoff. Retained
all earlier evidence here, including failed investigations and pending-at-the-time
manual checks; they are not present-day instructions.

Verified local launcher/config and implementation symbols, checked the latest
recorded CI/build logs, reproduced the pinned inventory byte-for-byte, and
checked local Markdown links/anchors and `git diff --check`. No runtime gate
was rerun for these documentation-only changes. No compositor code, generated
config/CLI references, user config, installed binary or running session changed.
The unrelated output-management files remain outside the documentation commit.

## Output-management checkpoint (2026-10-19)

The user explicitly authorized reviewing, committing and pushing the previously
separate output-management changes. Kept them as one focused fix with its three
headless protocol regressions. Refresh-rate changes preserve head/mode objects;
identical snapshots emit no update; mode-list changes replace modes while keeping
the head; a late release of a disconnected head cannot remove its replacement.
Ongoing configurations are cancelled on a real state change, and serial increment
wraps safely. This does not add multi-output tiling ownership or output disabling.

Full `dev/scripts/ci-local` passed (exit 0), including the three output-management
regressions, strict all-target clippy, 368 library tests, 1451 binary cases plus
isolated soak, and integration suites. Log:
`/tmp/driftwm-output-management-ci.log`. The installed compositor and running
session were not changed. Destination authorized by the user:
`origin/nationi/development` in `nationi/driftwm`.

`cargo build --locked` also passed; log: `/tmp/driftwm-output-management-build.log`.

## Native frames at grouped resize release (2026-10-20)

Continued the focused P2/P13 release acceptance gate against the pinned niri
`12de51d6a60baad8896366920278a24f17921f52`. niri's `Tile::render` derives border
bounds from animated window size (`border_window_size`); driftwm's SSD needs its
own conversion because niri has no equivalent compositor title bar.

Normal unblurred tiled geometry chases now compose the border and shadow around
the current visual rectangle with native stroke/radius, and regenerate the SSD
bar at the visual width with native height/text. Content still uses its existing
geometry affine and premultiplied resize blend. Frozen SSD bar height shares
physical-pixel rounding with the live bar; popup animation anchors share the
live root/geometry snapping. The first unblurred resize frame uses the old bake
directly instead of upsampling it to the target and downsampling it again.
That optimization is explicitly opted in: frosted and canvas blends retain
their previous common-sized intermediate, preserving the existing frost gate.

Recapturing an unblurred resize blend retains native border/shadow bounds and a
separate title bitmap. Decoration updates replace their memory buffer, so its
clone safely keeps the previous title pixels. The renderer keeps content and
chrome transforms separate for these snapshots. This does not generalize the
native frame path to frosted windows or inherited opening conversions.

Added two ignored GLES regressions (run by CI with `--include-ignored`):
`tiling_release_preserves_the_complete_ssd_frame` and
`tiling_release_preserves_the_complete_csd_frame`. Both cover shadow, menu and
1x/1.25x/1.5x scale. Complete initial release uses the unchanged one-byte shadow
tolerance; middle-of-chase checks bound title height and ring stroke; a repeated
resize checks the upper chrome band within two bytes. The SSD test synchronizes
with font warmup so asynchronous first-time glyph availability cannot change
between compared frames. Existing progress assertions tick the animation before
requiring changed pixels: release itself intentionally preserves the old frame.

The broader attempted full-frame recapture check exposed fractional-scale corner
resampling (maximum channel deltas 117 in SSD at 1.25x and 89 in CSD at 1.5x).
This is recorded as an unfinished frame-conversion gate, not hidden by a wider
full-frame tolerance. The added supersession check deliberately covers only the
upper chrome band; complete initial release still compares every pixel.

Targeted SSD/CSD tests passed: `/tmp/driftwm-native-recapture.log` (2 cases, six
scale scenes). The preceding graphics sweep passed all 52 pixel cases before
the final separate-title recapture addition; the complete final CI run below is
the authoritative regression gate. Earlier attempts exposed incorrect blur-path
opt-in and recaptured border scaling; both were corrected before the final gate.
No installed binary, user configuration or running desktop was changed. No push
was performed for this batch. P0 audit and the remaining P2/P13 gates stay open.

Final validation: `dev/scripts/ci-local` passed, exit 0;
`/tmp/driftwm-chrome-release-ci.log`. Includes fmt, locked check, strict
all-target clippy, no-run compilation, 368 library tests, 1453 binary tests
including ignored GLES cases, every integration suite and isolated soak (one
additional binary case). Local documentation links and `git diff --check`
also passed. No pure stage/layout/config logic changed, so the extra deep
proptest run was not needed. Next: full fractional-scale resize recapture and
blur/inherited-opening frame conversion, alongside the still-open P0 audit.

## Retain the mixed texel grid across resize supersession (2026-10-20)

Continued the full-frame recapture gate left by `3170ceb`. Replacing the earlier
upper-chrome crop with a complete frame reproduced the recorded corner deltas:
`/tmp/driftwm-recapture-before.log` (SSD 117 at 1.25x, CSD 89 at 1.5x). The held
mixed texture was wrapped in `ClosePixels`, flattened a second time into a
snapshot, and baked again for its next resize leg. These extra sampling passes
changed fractional-scale corner coverage.

Normal unblurred resize supersession now retains the mixed texture and its
original texel extent directly for both the waiting picture and the next resize
leg. The old crossfade is dropped only after the snapshot and next leg own their
texture handles, so it cannot reuse that render target underneath the retained
picture. Failure to capture a mixed texture still skips capture; it must not
silently replace the currently visible mix with the new client's pixels.
Separate native SSD/title/ring/shadow presentation remains as in `3170ceb`.
Blurred and inherited-opening capture retain their existing conversion path.

Extending the test to three successive preset changes exposed a second issue:
the source logical geometry changed on each release, so fractional rounding of
that source moved output edges by a pixel even with the same retained texture.
The normal native-frame live and snapshot paths now derive a texture affine
from its actual rasterized source bounds to the same rounded physical visual
rectangle. Client buffer dimensions no longer determine the retained picture's
output rounding. Source premultiplied pixels and compositor opacity stay separate.

Reference comparison used the unchanged pinned niri revision
`12de51d6a60baad8896366920278a24f17921f52`: `Tile::render` resize handling keeps
snapshot texture/physical geometry separate from animated window size, and
`RenderSnapshot` caches texture plus physical extent. This change addresses
our extra sampling/rounding passes; it is not a claim that driftwm's resize
shader or complete animation policy now matches niri.

Expanded the existing SSD/CSD GLES regressions to compare every pixel across
three successive preset changes (growth and shrink), at 1x/1.25x/1.5x. Each
cycle checks complete recapture within two channel bytes, a byte-identical held
frame after a tick, and complete first release within two bytes. The original
initial release keeps its stricter one-byte shadow tolerance. Test clients reply
only to the newly configured participant, avoiding invalid repeat acknowledgements
on unchanged peers. `/tmp/driftwm-recapture-native-rect.log` passed both expanded
cases before removal of temporary failure diagnostics.

General odd geometry, nonzero geometry offsets, view/scale/output changes,
blurred/inherited-opening conversion and motion-time lifecycle gates remain open.
No installation, session restart, user configuration change or push in this batch.

Final validation passed: full `dev/scripts/ci-local`, exit 0, log
`/tmp/driftwm-recapture-ci.log`. Includes fmt, locked check, strict all-target
clippy, no-run build, 368 library tests, 1453 binary cases (including all ignored
GLES scenarios and the existing frost regressions), integration suites and
isolated soak. Local Markdown links and `git diff --check` passed. No pure
stage/layout/config change required an additional deep proptest pass. Next:
blurred/inherited-opening frame conversion during supersession; P0 semantic
audit and the broader scale/view/output/lifecycle gates remain unfinished.

## Native tiled frames and recapture with window blur (2026-10-20)

Continued the P2/P13 window-blur conversion gate after `78be755`. The previous
repeat-resize frost test held a new transaction without releasing it, and
resized the ordinary peer rather than the frosted window. Expanded it to focus
the frosted column and check initial release plus three successive preset
changes, including client buffer replies and release. Added an SSD variant
beside CSD at 1x/1.25x/1.5x, with font warmup synchronized before text comparisons.
The expanded baseline reproduced a 207-byte channel delta on the first 1x
frost release: `/tmp/driftwm-frost-native-before.log`.

Normal tiled windows with compositor-requested window blur now use the native
frame/mixed-texture path introduced in the last batches. Initial release keeps
the retained pixels; recapture reuses the mixed texel grid for the held image
and following resize leg. Live and snapshot blur requests use one visual-frame
extent constructor. Its mask crop preserves the original pixel-centre rule,
including the logical SSD title band: independently rounding that band changed
the crop at 1.5x and produced a 126-byte initial SSD release delta. Compositor
chrome still shares its separately snapped native title-bar band as before.
The backdrop remains live and no blurred background is baked into the snapshot.

Another release jump came from canvas destination-coverage suppression: a new
full-width target removed shadow/corners while its retained visual was still
mid-resize. Tiled columns now retain configured frames even when filling the
work area. Fullscreen policy remains in its existing path. This aligns live
presentation with the retained frame; normal canvas destination-coverage policy
is unchanged. Pinned niri `12de51d6a60baad8896366920278a24f17921f52`,
`Tile::update_render_elements`, derives border geometry from animated tile size
and handles expanded/fullscreen radius separately from ordinary column width;
it does not use driftwm's canvas work-area coverage suppression.

Client-requested blur regions and inherited opening intentionally remain on the
previous conversion path. Those need separate mask/region/opening-scale gates;
this batch does not remove their guard. Framed canvas-resize entry, odd extents,
nonzero geometry offsets and changing view/output scale are also not accepted
by the new static-scale scenes. P0 audit and other roadmap milestones remain open.

Targeted expanded frost regression passed all six SSD/CSD scale scenes and
three resize cycles each, including initial release, complete recapture within
two channel bytes, byte-identical held frames after a tick and complete release
within two bytes: `/tmp/driftwm-frost-native-final.log`. Existing unblurred
initial-release tolerances were not weakened. Earlier temporary test config and
diagnostic attempts are not validation evidence. Full final CI result follows.
No user configuration, installed binary or running desktop was changed; no push.

The first complete CI attempt (`/tmp/driftwm-frost-frame-ci.log`) caught a
regression in the existing pure-translation frost capture case. A translation
with no resize blend must keep its previous mask-rounding policy. Restricted
the native window-blur path accordingly, without changing the unblurred path;
`/tmp/driftwm-frost-translation.log` passed all existing scale/frame variants.
The final full gate is `/tmp/driftwm-frost-frame-ci-final.log`.

Final full CI passed, exit 0: `/tmp/driftwm-frost-frame-ci-final.log`.
Includes fmt, locked check, strict all-target clippy, no-run build, 368 library
tests, 1453 binary cases including ignored GLES scenes, integration suites and
isolated soak. Local Markdown links and `git diff --check` passed. No pure
stage/layout/config change needed an additional deep proptest pass. Next:
inherited opening during resize supersession, followed by client blur regions;
the remaining view/scale/output, degraded capture and P0 gates are unchanged.

## Preserve inherited opening through grouped resize (2026-10-20)

Continued P2/P13 after `a5a8641`. Group release previously skipped the retained
resize picture whenever `has_open_fade` was true; the fade survived, but the new
client pixels replaced the held picture immediately. The new GLES scenario
reproduced the missing crossfade: `/tmp/driftwm-opening-before.log`.

Group release now creates the retained-content blend while keeping the existing
opening progress. This changes the grouped release path, not the pre-commit
policy that avoids capturing an unshown throwaway first buffer for an ordinary
canvas opening. Native frame bounds follow the unscaled resize geometry, with
an independent opening-only transform around bar, border and shadow. Content
uses the same opening transform applied to rasterized frame bounds, preserving
fractional source-origin rounding rather than deriving it from the new buffer.
Opening snapshots keep a separate title bitmap, avoiding another title-text
sampling pass. Opacity is applied after the premultiplied content blend.

Popup pixels keep their native dimensions under resize. A shared transform maps
the snapped source root onto the snapped visual-frame root, then applies only
opening scale. This fixed a one-pixel popup jump at 1.25x when the new client's
logical geometry changed the source rounding phase. Existing normal resize,
translation and blur paths remain covered by their regression suites.

Pinned niri reference is unchanged:
`12de51d6a60baad8896366920278a24f17921f52`. `Tile::render` wraps
`render_inner` (which handles resize) with the separate opening animation;
opening and resize are independent entries. driftwm now preserves that
separation in its grouped retained-picture path; this does not claim full
shader, physics or lifecycle parity with niri.

Added ignored GLES test
`tiling_resize_release_preserves_an_inherited_opening_picture`: 12 SSD/CSD scenes
at 1x/1.25x/1.5x, opening amplitude 0.9 or opacity-only 1.0, compositor opacity
0.6, premultiplied half-transparent client pixels, shadow and popup. Each runs
three releases, ticks opening while buffers wait, checks progress is unchanged
at release and increases on ticks, compares complete held/released and
resize-recaptured frames within two channel bytes, and checks final cleanup.
Targeted result: `/tmp/driftwm-opening-complete.log`, passed.

An additional probe comparing the original freshly opening rounded window with
its *first* retained capture differed by five channel bytes at 1.5x in the
unblurred CSD scene. This remains a separate sampling gate; it is not declared
fixed and the tolerance was not widened. The committed resize test compares
complete frames starting from the held picture and during subsequent resize
recaptures; it does not assert original-to-first-capture equivalence. Combined
window blur/inherited-opening resize still needs dedicated full-frame scenes,
as do client blur regions and the other scale/view/lifecycle gates. Existing
opening-blur regression scenarios remain part of CI.

No user config, installed binary or running desktop changed; no push. Final CI
result follows; the maintained status is the continuation point.

Final full `dev/scripts/ci-local` passed, exit 0:
`/tmp/driftwm-opening-ci.log`. Includes fmt, locked check, strict all-target
clippy, no-run build, 368 library tests, 1454 binary cases including ignored
GLES scenes, integration suites and isolated soak. Local Markdown links and
`git diff --check` passed. No pure stage/layout/config change required a deep
proptest run. Next: combined window blur/opening resize and initial rounded
opening-capture precision, then client blur regions and remaining P0/P2/P13 gates.

## Shared opening raster and combined blur/resize gate (2026-10-20)

Continued the two sampling gates left by `11619e1`: first fresh rounded-opening
capture and window blur combined with inherited opening during resize.
Restored the complete original-to-first-capture assertion in the opening test;
with its slower fade and opacity 0.6 it reproduced a 7-byte CSD delta at 1x:
`/tmp/driftwm-opening-precision-before.log`. The earlier 5-byte 1.5x probe and this
case share the same cause: a live opening evaluated the analytic corner clip
while drawing the scaled window, whereas a retained picture baked that clip
before scaling. Filtering a baked alpha edge is not the same operation.

Live ordinary rounded opening now uses the same premultiplied content raster
as the retained path. A per-output/per-element cache keys the bake by the
existing full surface-tree commit/view/geometry/scale/chrome stamp. Alpha and
opening scale stay outside the texture; client repaints replace the cached
bake, so a window is not frozen for the duration of opening. The existing
resize blend takes precedence. This also affects ordinary canvas opening for
eligible rounded windows, which is necessary to make the pre-tiling picture
and its first capture agree. Fullscreen, pinned, client-region blur and
coverage-suppressed pictures retain their existing paths.

The cache is pruned when opening ends or the stage ID dies, removed eagerly on
window unmap, and scoped output entries are removed on output teardown. Added
it to debug counters so fixture teardown detects lifetime leaks. A cached
source is sampled without rebuilding on alpha/scale-only animation ticks;
content/geometry/scale/chrome changes rebuild it. Kept the real-client pre-commit
policy for unshown throwaway opening buffers unchanged.

Reference remains niri `12de51d6a60baad8896366920278a24f17921f52`.
`Tile::render` feeds `render_inner` into a distinct opening offscreen; the
principle here is likewise to scale rasterized content consistently, rather
than swapping analytic and prefiltered edges at capture. driftwm caches the
content body and composes chrome/popups separately; this is not identical to
niri's complete opening-render implementation.

Expanded the opening test to assert full original-to-first-capture continuity
within the unchanged two-byte tolerance. It also paints a different client
buffer before capture to verify cache invalidation and checks that cached
content is released after opening settles. Added ignored GLES scenario
`tiling_resize_release_preserves_frost_during_inherited_opening`, sharing the
12 SSD/CSD scale/amplitude/opacity/shadow/popup scenes with window-rule blur
and a striped shader backdrop. The test requires working blur shaders and
checks actual blur-cache participation. Both variants run three resize releases,
check full-frame capture/release/recapture, opening progress while buffers wait,
and cleanup. Combined coverage is 24 scenes; no tolerance was widened.

Targeted results: `/tmp/driftwm-opening-precision.log` (unblurred) and
`/tmp/driftwm-opening-frost.log` (blurred), passed before the final repaint
invalidation assertion was added. Final full CI below is authoritative.
Client blur regions, general odd extents/offsets, capped capture and changing
view/output/scale remain separate gates; the P0 audit remains incomplete.
No user configuration, installed binary, running session or push was changed.

Final full `dev/scripts/ci-local` passed, exit 0:
`/tmp/driftwm-opening-sampling-ci.log`. Includes fmt, locked check, strict
all-target clippy, no-run build, 368 library tests, 1455 binary cases including
ignored GLES scenes, integration suites and isolated soak. The final expanded
repaint/cache-cleanup assertions passed in this gate. Local Markdown links and
`git diff --check` passed. No pure stage/layout/config changes required an
additional deep proptest run. Next: client blur regions and their native-frame /
opening conversion; other roadmap gates and P0 audit remain open.


## 2026-10-20 — committed client blur regions through animated native frames

Baseline: `0163705`. Real `ext-background-effect-v1` clients exposed two distinct
faults: live bounds were transformed before source-local rectangles were clipped,
and fractional SSD bands rounded region origins differently on initial capture.
Transform whole requests after source clipping; native resize/opening frames now
project region endpoints onto their shared raster body before applying opening.
Client effects use the shared clipped animated-content bake during geometry
motion as well as opening. Keep eligibility stable after the first committed
effect, including later unset/empty states; Smithay cached-state presence alone
is insufficient because a pending request creates that entry before commit.
Retained region metadata stays copy-on-write; release now preserves native initial
pixels for these clients as it already does for window-rule blur.

The new ignored GLES scenario uses actual protocol requests and covers 18 scenes:
three motions (opening, move, seeded canvas resize), SSD/CSD, and static
1x/1.25x/1.5x. Content is half-alpha green under opacity 0.6, rounded border and
shadow over a striped backdrop. It verifies first-set and unset double buffering,
visible partial frost bounded independently by the animated client rectangles,
empty/outside region suppression, first grouped capture/release, successive
capture, a region-only commit while held, and real doubled client dimensions
with proportionally doubled regions. Complete transitions retain <=2 byte
comparisons; the held picture is byte-identical after region-only commits.

Development probes: `/tmp/driftwm-client-regions-before.log` showed a 188-byte
initial-capture discrepancy; source-order repair alone left a 127-byte fractional
release discrepancy. Native body sampling closed that gap. The expanded framed
probe caught an SSD 1.5x region origin shifted by one pixel; sharing absolute
raster endpoint projection closed it. The alpha=1 ghost used by older blur tests
quantized away under SSD opening opacity; the new scenario uses half-alpha
content so the enabled/disabled assertion actually measures visible frost.
Final dedicated test: `/tmp/driftwm-client-regions-final.log`, passed. Full
`dev/scripts/ci-local` passed (exit 0, `/tmp/driftwm-client-regions-ci.log`):
368 library cases, 1456 binary cases including ignored GLES scenes, integration
suites and isolated soak. The final expanded precommit/mask-bound assertions
passed in the dedicated run; final fmt and strict all-target clippy passed
(`/tmp/driftwm-client-regions-final-clippy.log`). `git diff --check` and local
Markdown targets passed. No pure stage/layout/config algorithms changed.

Remaining: arbitrary independently changed old/new region shapes need retained
mask blending, rather than merely projecting the latest region onto the mixed
body. Proportional-buffer tests do not close that gate. General geometry offsets,
odd extents, degraded/capped pictures, changing views/scales/output, lifecycle
transitions and P0 semantic audits remain open. No installed binary, user config,
running compositor session or push was changed.


## 2026-10-20 — independently changed client blur masks during resize

Baseline: `5f87ed0`. Old region metadata is retained beside resize content.
Changed masks now threshold each source's alpha independently, restrict it to
that source's normalized client rectangles, and blend coverage additively with
the exact content weights. Thresholding the mixed content first would allow an
opaque old pixel to populate a transparent new client region. The final frost
multiply preserves fractional mask alpha. One reusable body-sized GPU target is
kept with the resize crossfade; a superseding group retains its weighted mask
as one texture, so repeated recapture does not accumulate recursive layers.
Frozen masks are independent of later client region-only commits.

The first release stays on the original binary-mask raster path until the first
tick: pre-thresholding a resampled rounded corner changed two pixels by 84 bytes
in the initial probe. Immediate recapture retains that initial mask policy
separately from the current client's geometry. Later recapture retains already
weighted coverage. Opening transforms wrap the raw native body/frame as before.
A changed-mask request stays alive when the current client disables its region,
so the retained old coverage can fade out before normal no-blur rendering resumes.

`changed_client_blur_shapes_blend_through_resize_and_recapture` covers 12 scenes:
opening/seeded canvas resize, SSD/CSD, scales 1x/1.25x/1.5x, rounded frame/shadow,
opacity 0.6 and half-alpha content. It changes region topology while doubling
client dimensions, checks hold/release and immediate supersession, samples GPU
coverage to require fractional old/new weights summing to one, and checks a
transparent hole in the new source remains unfrosted even though old content
there is opaque. It then captures the mixed mask, commits a region-only disable
while held, releases without a pixel jump, and settles to no residual frost.
Complete capture/release comparisons stay within two bytes; held updates must
be byte-identical. Dedicated log: `/tmp/driftwm-mask-changes.log`, passed.
Full `dev/scripts/ci-local` passed, exit 0:
`/tmp/driftwm-mask-blending-ci.log`. Includes fmt, locked check, strict all-target
clippy, no-run build, 368 library cases, 1457 binary cases with ignored GLES,
integration suites and isolated soak. The final body-mask fallback preserves
whole-window-rule semantics when client rectangles clip out; final strict clippy
(`/tmp/driftwm-mask-final-clippy.log`), fmt and both client-blur regressions
(`/tmp/driftwm-mask-final.log`) passed after that adjustment. Local Markdown
links and `git diff --check` passed. No pure stage/layout/config changes required
a deeper proptest run.

This is a driftwm retained-frost adaptation, not a claim that the pinned niri
resize shader implements these exact masks: `/tmp/niri/src/layout/tile.rs`
combines old/current offscreens independently of the window's background effect.
P0 semantic parity remains incomplete. Next: mixed whole-window-rule/client
policies and SSD title bands, general framed canvas resize entry, then the
remaining degraded/scale/view/output/lifecycle gates. General odd extents and
geometry offsets are still unvalidated. No installed binary, user config,
running session or push changed.


## 2026-10-20 — roadmap and handoff cleanup

Reconciled the active docs with implementation checkpoint `e6eda19`. Removed
obsolete `059556e`/old test totals from the roadmap and source-version claims
from the TTY launcher guide. Condensed the chronological P2 narrative into
current capabilities; detailed batch evidence stays in this journal. The roadmap
now owns one execution queue: B1–B4 stabilization, A0/P0 as the explicit semantic
prerequisite to feature expansion, and a concrete B1 acceptance checklist.
The status is a compact handoff and evidence summary. Corrected the stale history
heading and the unsupported claim that a complete compatibility table existed
in the handoff. Historical entries remain intact. Documentation-only change;
content, local link/anchor and diff checks replace runtime CI for this cleanup.


## 2026-10-20 — reconcile fork branches and integrate upstream

`origin/main` contained IPC cursor/PID commits absent from development; it was
merged at `c54d48f`. Upstream through `7ce05ed` was merged at `327fcf5`, including
viewport sloppy-focus and subsurface pointer-constraint fixes. README kept the
fork's packaging removals; the test client keeps both background-effect and
subcompositor bindings. Removed distribution-specific packaging was not restored.

Full local `dev/scripts/ci-local` passed (`/tmp/driftwm-upstream-sync-ci.log`):
368 library cases, 1467 binary cases including ignored GLES, integrations and
isolated soak. No pure stage/layout/config parsing algorithm changed, so no
additional deep property pass was required. No installation or TTY acceptance
was performed. Tiling gates B1–B4/A0 remain as listed in the roadmap.

Repository-local pull is fast-forward only, push defaults to origin, fetch
prunes deleted remote refs, and local main tracks origin/main. The maintained
Git procedure is in `git-workflow.md`; local Git settings are not committed.


## 2026-10-21 — B1 mixed policies and native title-mask coverage

Baseline `8c17428`. The old retained mask contained body pixels only: when a
partial client region became whole-window rule blur, the SSD title stayed
unfrosted until the resize overlay disappeared. Body-identical client/full-window
regions also looked unchanged to the old comparison although their title policy
differed. Keep a separate scalar title coverage with each retained mask and blend
it with the same old/new weights. Capture/recapture transfers that scalar without
stretching a saved title texture. The final mask samples the actual native title
render element, thresholds its alpha at the capture's byte quantization boundary,
and weights it additively. Restore texture-program, blend function and blend
constant state after drawing; normal/retained composition records the actual
bar index after popup insertion. No extra title GPU allocation is retained.

`mixed_client_and_window_blur_preserves_native_title_coverage` exercises 18 scenes:
SSD/CSD, 1x/1.25x/1.5x, opening/seeded canvas resize; half-alpha content and opacity
0.6, rounded frame/shadow, real 300→600→300 logical buffer sizes. It checks partial
→ unset rule fallback → partial, immediate and mid-animation recapture, exact held
frames, full-frame release comparisons <=2 bytes, fractional GPU title weights,
rounded title cutouts, independent transparent new pixels and body-identical
full-body client/full-window rule regions. Empty and clipped regions must retain
the rule fallback; client-only cases still turn off. Dedicated log:
`/tmp/driftwm-b1-mixed.log`, passed. The earlier two client-mask tests reuse the
same helper through an explicit scenario enum and shared GPU sampling helper.
Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b1-ci.log`):
fmt, locked check, strict all-target clippy, no-run build, 368 library cases,
1468 binary cases including ignored GPU scenarios, integration suites and
isolated soak. Local Markdown targets and `git diff --check` passed. No pure
stage/layout/config parsing changes required an additional deep property pass.

Source audit: pinned niri `src/render_helpers/background_effect.rs`,
`BackgroundEffect::update_render_elements` honors explicit `blur = false` as a
veto; `compute_render_params` returns no effect for an explicit empty client
region. driftwm's documented rule booleans are sticky-on, and
`config::effective_blur` ORs mapped rule/default blur. Existing `SnapshotBlur`
request tests explicitly require whole-window fallback for clipped client regions
when that flag is on. Preserve this compatibility behavior in B1 rather than
silently converting rule semantics. This is compositor integration evidence,
not a claim of exact niri background-effect parity. A0 must account for that
policy difference.

Remaining B1: global decorations-blur reload while held, and cancellation/
lifecycle coverage of partially mixed native titles. Mapped rules apply only
at mapping; config rule edits do not rewrite existing window flags. B2–B4 and
P0/A0 remain open. No installed binary, user config, running compositor session,
release build, push or manual TTY acceptance changed in this batch.

## 2026-10-21 — B1 global blur reload and mixed-title cancellation gates

Baseline `2ee9b26`. Added regression scenarios through the real config-reload
entry point and ext-background-effect protocol; no additional production-render
change was needed. `global_blur_reload_preserves_held_masks_and_blends_release`
covers 12 SSD/CSD scenes at 1x/1.25x/1.5x with opening/seeded resize, opacity 0.6,
rounded frames, transparent new pixels and real buffer growth/shrink. Enable
`[decorations].blur` while holding the old partial client image, release and
recapture mixed coverage, then disable it while held. Both reloads preserve the
held frame exactly; release/recapture retain the two-byte tolerance. GPU samples
check body/title weights and rounded/transparent cutouts; settled disable leaves
no residual frost. Explicit unchanged output mode/scale keeps this gate separate
from B3 output reconfiguration. Mapped window-rule flags are not rematched.

`global_blur_reload_cancellation_releases_mixed_native_masks` adds 12 SSD scenes
at those scales and both motions: exit tiling or disconnect while mixed native
title coverage is held after disabling global blur. Exit retains the live canvas
window and accepts its restore response; disconnect removes its stage identity.
Both clear grouped snapshots and resize crossfades, and all closing/animation
state drains. Disconnect's settled frame equals an independently rendered empty
background. The cancellation test explicitly waits for asynchronous font warmup;
without it, first-use SSD text can arrive between capture and release and produce
an unrelated pixel mismatch. No image tolerance was relaxed. Fixture teardown
also checks resource counters. Shared expectations now name effective fallback
blur separately from mapped-rule blur.

Dedicated gate: `cargo test --bin driftwm global_blur_reload -- --include-ignored`,
passed both tests / 24 scenes (`/tmp/driftwm-b1-reload.log`). Controlled B1 gates
are delivered; B2 odd extents/geometry offsets are next. Broader lifecycle,
view/output transitions and the A0 semantic-policy audit remain open. No release
build, installation, desktop restart, push or manual TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b1-reload-ci.log`):
fmt, locked check, strict all-target clippy, no-run build, 368 library cases,
1470 binary cases including ignored GPU tests, integration suites and isolated
soak. Local Markdown targets and `git diff --check` passed. This batch changes
only fixture scenarios/documentation, so no additional deep property pass was
required.

## 2026-10-21 — B2 general framed canvas-resize entry

Baseline `ced39d2`. A normal canvas resize without a background-effect object
stretched the compositor frame with the content; switching to the held tiling
picture changed frame thickness, shadow footprint and popup placement. The new
odd-offset regression initially found a 135-byte channel difference at 1x CSD.
Eligible normal canvas resizes now use native frame geometry and the same cached,
clipped content grid as capture/release. Snapshot chrome uses that policy for
non-translation geometry even without a client-effect object. Translation-only,
pinned/fullscreen and degraded/ineligible animation paths retain their existing
policy. Cached content is keyed by surface commit/geometry/chrome/scale and uses
the existing animation cache eviction; no new retained collection was added.

`tiling_captures_odd_offset_geometry_during_framed_canvas_resize` covers 12 scenes:
SSD/CSD, 1x/1.25x/1.5x, a 313x223 logical/buffer window with geometry (7,9,299,201),
rounded corners, shadow and opacity 0.6. Seeded canvas grow/shrink enters tiling,
responds with a differently sized buffer with the same nonzero geometry offset,
then recaptures during crossfade and exits. Capture/release/recapture compare the
whole frame at <=2 bytes; release retains the exact visual rectangle. Independent
popup checks require 100x80 logical buffer dimensions at output scale and motion
matching the parent geometry anchor. Output caches are explicitly removed before
renderer teardown, and fixture resource counters must return to baseline.
Dedicated gate `/tmp/driftwm-b2-odd.log`: passed all 12 scenes.

Pinned niri `src/layout/tile.rs` resize rendering keeps current texture physical
geometry and passes it with old snapshot size/current window size to
`ResizeRenderElement::new`; its comments explicitly require reversible rounding
at the same scale. This supports checking frame-space geometry separately from
client textures, but these driftwm canvas-entry scenes are integration evidence,
not recorded niri visual parity. B2 remains open for fractional client-buffer
texel grids, patterned/transparent edges, changed geometry offsets and release of
the recaptured picture in both buffer-size directions. No stage/layout/config
parser algorithm changed. No installation, release build, desktop restart, push
or manual TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b2-ci.log`): fmt,
locked check, strict all-target clippy, no-run build, 368 library cases, 1471 binary
cases including ignored GPU scenarios, integration suites and isolated soak.
Existing client-mask, mixed-title/reload, CSD/SSD release and translation scenarios
remain green. Markdown targets and `git diff --check` passed; no additional deep
property pass was required for this renderer-only behavior change.

## 2026-10-21 — B2 fractional client buffers and changed offsets

Baseline `de35b93`. Extended the shared framed-canvas resize fixture with
`tiling_recapture_handles_fractional_buffers_and_changed_geometry_offsets`:
12 additional SSD/CSD scenes at static 1x/1.25x/1.5x, seeded grow/shrink. Buffer
texel dimensions use ceil(logical × output scale), with alternating opaque and
half-alpha blocks through the content edges and a transparent interior rectangle.
Actual viewport destination remains logical-sized. First response changes the
geometry offset (7,9) → (11,5); recapture response changes it again to (3,13).
The second response explicitly exercises buffer growth/shrink and verifies the
accepted geometry instead of assuming clients always take the offered size.

Capture, release, mid-animation recapture and recapture release keep full-frame
continuity within two bytes. Reuse the independent popup-size/displacement checks.
After the second release settles, an independent surface-local sample verifies
that the transparent hole remains clear; another samples the center of a known
pattern block and verifies premultiplied red and opacity 0.6 (153 or 76), separately
from frame comparisons. Snapshots and resize crossfades clear; fixture counters
return to baseline. No additional production change was required. Original
solid 1x-client scenarios remain distinct and are retained in the full suite.
Dedicated `/tmp/driftwm-b2-fractional.log` passed all 12 scenes.

Controlled B2 gates are delivered. These are driftwm compositor-integration
checks, not recordings of niri or proof of all fractional rounding combinations.
B3 starts with stale/capped capture policy, followed by changing views/scales/
outputs. B4 and the A0 semantic audit remain open. No pure stage/layout/config
parser change required a deep property pass. No installation, release build,
desktop restart, push or manual TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0
(`/tmp/driftwm-b2-fractional-ci.log`): fmt, locked check, strict all-target clippy,
no-run build, 368 library cases, 1472 binary cases including ignored GPU scenarios,
integration suites and isolated soak. Markdown targets and `git diff --check`
passed. This batch adds fixture coverage and documentation only.

## 2026-10-21 — B3 capped stale canvas-resize live fallback

Baseline `ec2a9d0`. Existing `can_capture_canvas_geometry` deliberately rejects
an expired start-hold with stale content: capped pixels can sit inside a larger
animated frame, so flattening and stretching that whole image would invent
magnified content. Preserve this live fallback rather than treating it as an
ordinary retained resize. The renderer nevertheless switched to native tiling
chrome as soon as stage membership changed, while content still used the capped
canvas affine. The new regression found a 255-byte channel jump on entry at 1x
CSD. Exclude `AnimatedVisual::cap_content` from native resize-chrome selection;
keep the existing live chrome/content transform until the capped phase ends.
No additional texture or retained state is allocated by the fix.

`degraded_canvas_resize_keeps_capped_live_content_when_tiling_starts` covers 12
SSD/CSD scenes at 1x/1.25x/1.5x, with real reply or timeout. A 200x100 red buffer
ignores a 600x400 canvas request; injected time expires the 300 ms start hold and
confirms the visual grows while `cap_content` is true. Independent red-pixel
bounds require the content to remain 200x100 at output scale. Entry compares the
whole frame within two bytes and requires zero grouped snapshots. Three width
supersessions keep the current frame exactly and still allocate no snapshots.
A real green-buffer reply or injected group timeout releases the pending state;
injected monotonic animation ticks drain endpoint holds without wall-clock sleeps.
Unanswered settled content still has its original extent. Crossfades clear and
fixture teardown checks resource counters. Dedicated gate:
`/tmp/driftwm-b3-degraded.log`, all 12 scenes passed.

Pinned niri `src/utils/transaction.rs` documents its 300 ms transaction deadline
as protection against clients that do not respond promptly. This supports bounded
waiting, not equivalence with driftwm's canvas cap policy: that is a documented
integration fallback. B3 remains open for view/scale/output changes and additional
degraded/effect/minification/allocation-failure combinations. No pure stage/layout/
config-parser algorithm changed; no additional deep property run is required.
No release build, installation, desktop restart, push or manual TTY acceptance
was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-degraded-ci.log`):
fmt, locked check, strict all-target clippy, no-run build, 368 library cases,
1473 binary cases including ignored GPU scenarios, integration suites and isolated
soak. Existing B1/B2 matrices remain green. Markdown targets and
`git diff --check` passed.

## 2026-10-21 — B3 camera motion and CSD output-scale changes

Baseline `adf9332`. Added
`held_tiling_images_survive_csd_scale_and_ssd_camera_changes`, 12 controlled
scenes: six CSD changes 1→1.25, 1.25→1.5 and 1.5→1 (ordinary/mixed retained image),
and six SSD camera-only cases at fixed 1/1.25/1.5 scales. The actual config-reload
entry point applies an explicit unchanged 1200x800 mode and selected scale;
assert no pending mode intent. Retained images/popups survive reflow. Move the
camera by (16,12), whose physical displacement is integral at these scales, and
independently compare every overlapping pixel with the correspondingly shifted
stationary image (<=2 bytes). Check native popup dimensions against the new scale,
answer the latest size configure, and compare held/released frames <=2 bytes.
Crossfades drain and fixture counters return to baseline after output-cache and
renderer teardown. Dedicated `/tmp/driftwm-b3-view.log`: all 12 scenes passed.
No additional production change is included in this batch.

Unclosed defect: the initial matrix also changed SSD output scale. At 1→1.25,
ordinary held SSD release differed by 174 bytes (`/tmp/driftwm-b3-scale.log`). To
reproduce, in the new test's call to `check_tiling_scale_change`, pass `to` for
SSD too instead of retaining `from`. Keep this failing extension out of the
passing acceptance matrix until fixed, and keep the gap visible in status/docs.
The old retained frame/title raster and newly updated native SSD title do not
share a raster at release; native-title rerasterization/handoff needs explicit
pixel localization and treatment. A larger tolerance is not acceptance.

Suggested bounded next work: localize title/body/border differences independently;
retain the actual old title raster with its source extent, blend it with the live
native title in premultiplied space before applying compositor opacity, and carry
the mixed title through recapture. Preserve native band height, rounded cutouts,
first-frame identity, eventual live-title replacement and cleanup. Avoid drawing
two translucent titles source-over or treating body mask title coverage as saved
title pixels. Existing mask coverage scalars are not the title image itself.
Pinned niri `src/layout/tile.rs` tracks snapshot size and current physical texture
geometry separately in resize rendering; this supports explicit raster metadata,
not a claim that driftwm's native SSD behavior is reference-equivalent.

B3 remains open for this SSD defect, arbitrary camera fractions/zoom, output
mode/transform changes and other degraded/effect/allocation-failure cases.
Second-output enrollment still exits tiling; multi-output ownership is P11.
No pure stage/layout/config-parser algorithm changed, so no extra deep property
pass is required. No installation, release build, desktop restart, push or manual
TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-view-ci.log`): fmt,
locked check, strict all-target clippy, no-run build, 368 library cases, 1474 binary
cases including ignored GPU scenarios, integration suites and isolated soak.
Markdown targets and `git diff --check` passed. This passing suite covers only
the explicitly scoped CSD-scale/SSD-camera matrix, not the SSD scale defect above.

## 2026-10-22 — B3 retained native SSD title pixels across scale changes

Baseline `ddbbe20`. Re-enabled SSD scale transitions in the failing matrix and
localized the 174-byte first-release difference. Native title rerasterization
accounted for most of it; after retaining title pixels, a 28-byte difference
remained around border/shadow. Their held geometry still used the title band's
old-scale rounded height. Recompute native held border/shadow bands at the current
output scale, matching the title and live chrome. Ordinary SSD snapshots now keep
the native title separately rather than baking it into the body.

`render/title.rs` represents title pixels as the original immutable memory buffer
or a retained mixed texture with explicit physical extent. Each resize transition
owns an optional title source and one reusable mixed target. First release uses
the original raster path exactly. Subsequent frames add old/new premultiplied
pixels with the body's crossfade weights before applying window opacity; restore
source-over GL blending even on drawing failure. Capture transfers the mixed
texture into the next source, so recapture does not revert to the live title.
Old title/buffer targets follow the existing resize/snapshot cleanup lifetime;
no separate compositor-wide collection is added. These title pixels are separate
from B1's scalar blur title coverage.

`held_tiling_images_survive_output_scale_and_camera_changes` replaces the former
CSD-scale/SSD-camera-only test and includes all 12 CSD/SSD scale-change scenes:
1→1.25, 1.25→1.5, 1.5→1, ordinary/mixed snapshots, rounded chrome/shadow, opacity
0.6, actual changed buffer dimensions and native popups. Independent camera-shift
and popup-size assertions remain. First release, post-scale mid-animation
recapture and its release stay within two bytes. A title-interior sample requires
alpha 153±2, preventing source-over opacity accumulation. Animations and title
crossfades drain with the parent resize; fixture counters return to baseline.
Dedicated `/tmp/driftwm-b3-ssd.log` passed all 12 scenes. The earlier 174-byte SSD
defect is closed for this matrix; no tolerance was increased.

B3 remains partial: output mode/transform changes and nonintegral physical camera
translations are next; zoom, other degraded/effect/minification/allocation failures
and B4 lifecycles remain open. Multi-output tiling is still P11. This is native
SSD compositor integration, not a claim of an identical niri decoration scheme.
No pure stage/layout/config parsing algorithm changed, so no additional deep
property pass is required. No installation, release build, desktop restart, push
or manual TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-title-ci.log`): fmt,
locked check, strict all-target clippy, no-run build, 368 library cases, 1474 binary
cases including ignored GPU scenarios, integration suites and isolated soak.
Existing B1 mixed-policy/reload/cancellation, B2 framed/fractional and B3 degraded
fallback scenarios remain green. Markdown targets and `git diff --check` passed.

## 2026-10-22 — B3 fractional camera and output mode height changes

Baseline `382bb18`. Extended the shared view-change fixture with an explicit
scenario enum, preserving the original 12 scale-transition scenes.
`held_tiling_images_release_after_fractional_physical_camera_motion` adds 12
SSD/CSD scenes at 1x/1.25x/1.5x with ordinary/mixed snapshots, opacity 0.6,
rounded chrome/shadow and native popup overhang. Camera (16.4,12.2) introduces
fractional physical positions at those scales. Independently compute the popup
anchor's rounded displacement from the parent's held visual origin; require
native popup dimensions, first release/mixed recapture/second release within two
bytes, title alpha 153±2 and eventual crossfade cleanup. Original integral-camera
scenes retain their whole-image translated oracle; fractional motion instead uses
its independent rounded-anchor oracle plus the complete release comparisons.

`held_tiling_images_survive_output_mode_height_changes` adds 12 scenes with
1200x800→1200x1000 at the same static scales/decoration/mixed combinations.
Real config reload must queue a custom mode intent with the requested dimensions.
The fixture models successful backend completion: publish current/preferred mode,
arrange layers, resize fullscreen policy, invalidate output caches and mark output
configuration dirty. Then reflow the pending tiling group and require the offered
content height to equal new logical viewport height minus native bar/borders.
Render the complete taller target, retain the integral camera-translation oracle,
and reuse release/recapture/title/popup/cleanup checks. This is compositor policy
and successful-mode-application coverage; it does not invoke DRM modesetting.

No production fix was required by these 24 new scenes. Dedicated
`/tmp/driftwm-b3-views.log` runs all three view tests (36 scenes). B3 remains open
for actual rotation/reflection, mode width changes/shrink and other listed gaps.
The existing `render_elements_to_rgba` fixture fixes `Transform::Normal`; extend
that helper for actual rotated/reflected rendering before claiming transform
acceptance. Changing output metadata alone would be insufficient. No pure stage,
layout or config-parser algorithm changed; no extra deep property pass is needed.
No release build, installation, desktop restart, push or manual TTY acceptance
was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-views-ci.log`): fmt,
locked check, strict all-target clippy, no-run build, 368 library cases, 1476 binary
cases including ignored GPU scenarios, integration suites and isolated soak.
Dedicated view gate passed all 36 scenes. Markdown targets and
`git diff --check` passed. This batch changes tests/documentation only.

## 2026-10-22 — B3 actual rotated/reflected output rendering

Baseline `ade5ac1`. Parameterized the internal RGBA helper by the actual output transform; tests
export that entry point and the original screenshot/helper API remains a
normal-transform wrapper. The tiling
pixel fixture now renders raw mode-sized GPU buffers with the current transform,
then converts them to upright pixel coordinates with explicit integer permutations
for existing geometry assertions. Quarter-turns swap upright dimensions; frame
indexing and configure-height expectations use those transformed dimensions.
Release and recapture comparisons still inspect the complete rendered frame.

`output_transform_pixels_match_an_independent_upright_render` checks all seven
non-normal transforms with distinct red body/blue popup content. Render the same
composition once with the actual output transform and once upright, then compare
actual pixels through an independent discrete CPU permutation (<=2 bytes).
This catches wrong quarter-turn reflection mappings, not merely metadata changes.
`held_tiling_images_survive_rotated_and_reflected_output_rendering` adds 84 scenes:
90/180/270/flipped/flipped-90/flipped-180/flipped-270 × three scales × SSD/CSD ×
ordinary/mixed snapshots. Real config reload applies the transform; assert native
popup dimensions/parent anchor, reflowed configure height, first release/mixed
recapture/second release <=2 bytes, title alpha 153±2 and complete cleanup.
Dedicated logs: `/tmp/driftwm-b3-orientation.log` (seven-transform oracle passed),
`/tmp/driftwm-b3-transform.log` (84 scenes passed).

Initial diagnostic runs rendered the heavy upright oracle inside a timed commit
group and spent its real 300 ms budget before sending the client response. The
group correctly expired with unchanged client geometry, so no content crossfade
was needed; a later repaint then differed. This was fixture timing, not a lost
retained texture on output rotation. Keep the independent orientation test outside
commit groups, and retain only the essential render/anchor/continuity checks
inside them. No production timeout, pixel tolerance or capture policy changed.
No production rendering fix was required by the final matrix.

B3 remains open for mode width/shrink and other documented degraded/effect/
allocation-failure cases. The GPU path is genuine offscreen output transformation,
not hardware modesetting or DRM scanout. Multi-output tiling remains P11. No pure
stage/layout/config-parser algorithm changed, so no extra deep property pass is
needed. No installation, release build, desktop restart, push or manual TTY
acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-transform-ci.log`):
fmt, locked check, strict all-target clippy, no-run build, 368 library cases,
1478 binary cases including ignored GPU scenarios, integration suites and isolated
soak. The 84-case transformed matrix and orientation oracle pass in the full suite
as well. Markdown targets and `git diff --check` passed.

## 2026-10-22 — B3 output mode width changes and shrink

Baseline `398f889`. Generalized the view fixture's successful mode-application
scenario from a fixed taller output to explicit physical dimensions; the original
1200x1000 height-growth test remains. Added
`held_tiling_images_survive_mode_width_changes_and_shrink`: 48 scenes covering
1600x800 width growth, 900x800 width shrink, 1200x600 height shrink and 900x600
combined shrink, all from 1200x800. Each runs at 1x/1.25x/1.5x, CSD/SSD,
ordinary/mixed retained image, opacity 0.6, rounded frame/shadow and native popup.

Real config reload must queue exactly the requested mode dimensions. Model the
backend's successful apply boundary using that intent; publish mode/preference,
arrange layers, update fullscreen policy, invalidate render caches and mark output
configuration dirty. Independently require content height to equal new logical
viewport minus native chrome; proportional frame width is half the viewport for
the initial column and full width after the fixture's first width cycle, excluding
borders for the client offer. Both are checked before responding, not inferred
from final pixels. Existing whole-frame integral camera translation, popup size/
anchor, release/mixed recapture/second release <=2 bytes, title alpha 153±2 and
resource cleanup checks run unchanged. Dedicated `/tmp/driftwm-b3-mode.log`:
all 48 scenes passed. No production fix or tolerance/timeout change was needed.

Pinned niri `Layout::update_output_size` delegates to its monitor, whose
`update_output_size` refreshes scale, view size and working area before updating
workspaces. This motivates checking dimensions/offers independently of retained
pixels; it is not evidence of driftwm multi-workspace/multi-output parity. This
fixture tests compositor policy after successful backend mode application, not
DRM hardware modesetting. B3's next subset is degraded minification/client effects
and capture allocation failures; other stated limits remain. No pure stage/layout/
config parser algorithm changed, so no additional deep property pass is required.
No release build, installation, desktop restart, push or manual TTY acceptance
was performed.

Full `dev/scripts/ci-local` passed, exit 0 (`/tmp/driftwm-b3-mode-ci.log`): fmt,
locked check, strict all-target clippy, no-run build, 368 library cases, 1479 binary
cases including ignored GPU scenarios, integration suites and isolated soak.
The new mode matrix and existing 84 transformed scenes pass in the full suite.
Markdown targets and `git diff --check` passed. This batch changes tests/docs only.

## 2026-10-22 — B3 stale minification and client-region blur

Baseline `cdf1344`. Added
`degraded_canvas_resize_preserves_minified_content_when_tiling_starts`, 12
SSD/CSD scenes at 1x/1.25x/1.5x with real reply or timeout. Extend the earlier
capped-growth fixture with a stale 200x100 → requested 120x60 case. Inject time
to expire the start hold; require capped-policy/degraded capture ineligibility
and independently count rendered red content bounds at the minified extent.
Entry remains within two bytes, three supersessions preserve the current frame
exactly, and reply/timeout cleanup returns to a nonanimated live window. The
unanswered settled window must have its actual committed 200x100 content size.
Original 12 capped-growth cases remain. Dedicated `/tmp/driftwm-b3-minify.log`:
both tests / 24 scenes passed.

`degraded_client_blur_follows_capped_and_minified_live_content` adds 24 scenes:
SSD/CSD, three scales, capped growth/minification and reply/timeout. A real
ext-background-effect client requests a partial (20,15,90,55) region over a striped
shader backdrop through half-alpha content. After start-hold expiry, compare
enabled and disabled frames: differences must exist and all must lie inside the
independently transformed region bounds (two physical pixels for edge rasterization).
Client-only blur must not leak into title/shadow or unused stale-frame space.
Region-only re-enable restores the exact frame and does not make stale content
capture-eligible. Tiling entry uses zero retained images and stays within two
bytes. Real response or injected timeout drains all animations/crossfades;
settled unset and explicit empty client regions agree, leaving no residual frost.
Fixture counters return to baseline after normal per-output GPU teardown.
Dedicated `/tmp/driftwm-b3-degraded-blur.log`: all 24 scenes passed.

No renderer fix or tolerance/timeout change was required. These are controlled
isotropic stale-content/effect integration gates, not every degraded lifecycle,
anisotropic ratio, client constraint or niri effect-policy combination. B3's next
work is scoped deterministic capture allocation failure: test parent, popup and
resize-content bake separately, preserving transaction bounds and successful
recovery without inducing real GPU/OOM failures. No pure stage/layout/config
parser algorithm changed; no deep property pass is needed. No release build,
installation, desktop restart, push or manual TTY acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0
(`/tmp/driftwm-b3-stale-effects-ci.log`): fmt, locked check, strict all-target clippy,
no-run build, 368 library cases, 1481 binary cases including ignored GPU scenarios,
integration suites and isolated soak. Existing mode/transform/title matrices remain
green. Markdown targets and `git diff --check` passed. This batch adds fixture
coverage and documentation only.

## 2026-10-22 — B3 controlled capture allocation failures and recovery

Baseline `34570f7`. Added a thread-local, RAII-scoped, one-shot allocation fault
hook under `cfg(test)`. The real content/composite capture allocator returns
`None` at the chosen attempt; no enormous allocation, process-wide environment
change or actual driver OOM is involved. Production capture calls use the same
allocator and format as before.

`failed_tiling_capture_allocations_use_bounded_fallback_and_recover` covers 90
real-GLES scenes: parent-content, parent-composite, popup, resize-content and
resize-clip allocation, SSD/CSD, scales 1/1.25/1.5, and reply/timeout/exit. Real
parent/popup surfaces are mapped before arming the hook. The trigger is asserted;
parent/popup failure cannot install a partial snapshot tree. Resize-bake failure
keeps the two held images but cannot retain old resize content at release.
Entry differs by at most two bytes. A reply, injected deadline or exit drains
snapshots and failed crossfades; exit also clears pending configures. The next
successful capture must retain both images and install a real resize crossfade,
with release within two bytes. Settled green parent and blue popup are checked;
a same-size repaint cannot revive the transition. Fixture counters return to
baseline after per-output GPU teardown.

The added recovery-release pixel check exposed a real fractional-scale defect:
at 1.5x, an odd logical height was padded to the chrome alignment grid even for
a content-only bake. The extra transparent row became part of the resize source,
stretched rounded edges and produced a 254-byte maximum difference. Content-only
bakes now use the logical grid; native border/shadow/title composites retain
their physical-grid alignment. All 90 scenes pass without changing the two-byte
tolerance or 300 ms transaction deadline (`/tmp/driftwm-b3-failures.log`).

This closes B3's named controlled gates. It does not cover actual GPU OOM,
bind/import/draw failures, every anisotropic/zoom/effect combination or hardware
scanout. Next is B4: inspect existing motion-time remap, disconnect, supersession
and output-removal coverage, then fill the next missing bounded lifecycle gate.
A0 remains required before feature expansion; multi-output tiling remains P11.
No pure stage/layout/config parsing algorithm changed, so no deep property pass
is required. No release build, installation, desktop restart, push or manual TTY
acceptance was performed.

Full `dev/scripts/ci-local` passed, exit 0
(`/tmp/driftwm-b3-failures-ci.log`): fmt, locked check, strict all-target clippy,
no-run build, 368 library cases, 1482 binary cases including ignored GPU scenarios,
integration suites and isolated soak. Existing odd/fractional-client,
blur/title, mode and transform matrices remain green with the bake-grid fix.
Local Markdown targets and `git diff --check` passed.

## 2026-10-22 — B4 direct remap during held/mixed resize

Baseline `f54ba6d`. Existing `remapping_a_tiled_window_restores_other_windows_before_placing_it`
checked immediate state before replies, without a renderer. The new
`remapping_a_held_or_mixed_tiling_window_clears_retention_and_keeps_placement`
adds 12 GPU scenes: SSD/CSD, 1x/1.25x/1.5x, ordinary held parent/popup or
recaptured mixed resize after a real green buffer and 16 ms animation tick.
Explicit `map_window` exits tiling, drains snapshots/pending configures/crossfades
and animations, and retains the window ID at its requested new position. Submit
an already-produced tiling-sized buffer after cancellation, then acknowledge the
200x100 canvas restore. Both commits must preserve that position. The settled
frame contains the green parent and blue popup, and fixture counters return to
baseline after per-output GPU teardown.

The mixed case reproduced a bug before the fix: a late buffer moved the window
from (210,170) to (-398,-228). `exit_tiling` had registered an owed canvas
recenter after a real tiled-size commit; the subsequent explicit map left it
armed. `map_window` now discards this exit recenter after preparing a tiled
remap, before applying the new position. The change is scoped to windows that
were tiled, preserving existing center-based rule-point placement semantics.
The hermetic `remapping_after_a_tiling_resize_supersedes_the_restore_recenter`
checks the same restore-response regression without GLES.

Dedicated `/tmp/driftwm-b4-remap.log`: all 12 scenes passed.
`/tmp/driftwm-b4-remap-unit.log`: three matching hermetic tests passed; the GPU
case was intentionally ignored in that invocation. This is direct compositor
remap, not xdg protocol unmap/remap, full B4 completion or niri remap equivalence.
Next: retained-motion disconnect with a surviving ready peer; then reconcile
remaining protocol-remap/supersession/output-removal coverage. A0 remains required
before feature expansion, and independent multi-output tiling remains P11.
No pure stage/layout/config parser algorithm changed; no deep property pass is
needed. No release build, install, desktop restart, push or manual TTY acceptance
was performed.

Full `dev/scripts/ci-local` passed, exit 0
(`/tmp/driftwm-b4-remap-ci.log`): fmt, locked check, strict all-target clippy,
no-run build, 368 library cases, 1484 binary cases including ignored GPU scenarios,
integration suites and isolated soak. Local Markdown targets and
`git diff --check` passed.

## 2026-10-22 — B4 disconnect with a ready surviving peer

Added `disconnecting_a_held_or_mixed_tiling_peer_releases_the_survivor` in
`src/tests/render_pixels.rs`: 12 real GLES scenes, SSD/CSD at 1x/1.25x/1.5x,
ordinary held entry and recaptured mixed resize. Two independent clients render
a red/green survivor with a blue popup and a magenta departing window. The
mixed variant releases both initial offers, ticks 16 ms, actually renders the
crossfade, then consumes the adjacent column to request a new grouped resize.
Only the survivor answers. Its committed geometry remains held and its entire
rendered frame is byte-identical until the other client disconnects.

Disconnect releases the queued buffer. The departing stage ID and crossfade
are gone; the survivor keeps its ID and becomes/remains structurally and
keyboard focused. A possible fresh full-height offer after stacked-peer removal
is answered explicitly, without sleeping through the group deadline. After
settling, the actual frame contains green body/blue popup pixels and no magenta
peer pixels; snapshots, pending configures and survivor crossfade are empty.
A subsequent unchanged client commit neither restarts animation nor changes the
frame. The test client's surface dispatcher now records `wl_surface.enter` /
`leave`, so the survivor's single membership is checked against the actual
output name rather than inferred from stage placement. Fixture teardown checks
resource counters; per-output renderer caches are removed before GL uninstall.

No production change was required for this controlled gate. Dedicated evidence:
`/tmp/driftwm-b4-peer.log`, 12 scenes passed. Full validation:
`/tmp/driftwm-b4-peer-ci.log`, `dev/scripts/ci-local` exit 0 (368 library cases,
1485 binary non-soak cases, integrations and isolated soak). Formatting, strict
all-target clippy and locked build checks passed. No pure stage/layout/config
algorithm changed, so no extra deep proptest pass was needed. No installation,
manual TTY acceptance or push was performed. GPU evidence remains NVIDIA
surfaceless rendering, not hardware scanout or exhaustive lifecycle parity.

Next bounded B4 task: protocol null-buffer unmap/remap during retained motion,
including initial configure/enrollment and stale transaction/image cleanup.
Direct compositor remap is a separate delivered scenario. Supersession and output
removal remain; A0/P0 audit still precedes feature expansion.

## 2026-10-22 — B4 protocol unmap/remap during retained motion

Reference inspection: pinned niri `12de51d6a60baad8896366920278a24f17921f52`,
`src/handlers/compositor.rs` mapped-root commit path: a null-buffer unmap removes
the window from layout, stores it as `Unmapped`, and requires a fresh initial
commit/configure sequence before mapping again. Driftwm keeps its existing live
stage record for a hidden toplevel, so this batch removes only tiling membership
while preserving its stage ID; it does not introduce niri's separate unmapped
registry, close animation or claim complete lifecycle equivalence.

New GPU regression
`protocol_remap_during_held_or_mixed_tiling_resize_reenrolls_cleanly` reproduced
an actual phantom column after null-buffer unmap. `Stage::untile` now reuses the
same membership/selection repair as dead-window pruning, leaving the element and
layout lifetime intact. After Smithay's buffer handler has applied the null
buffer, `handle_tiling_unmap` refreshes the window bbox, untiles it, drops its
snapshot/crossfade/animation/animated-content state and SSD, sends output leaves,
clears its focus and arms normal initial placement. The unmap commit returns
before interpreting reset role geometry as a mapped window. A subsequent empty
commit takes the existing initial-size/configure route; its first buffer enrolls
the same stable ID again. Focus recovery now recognizes a selected ID that left
tiling even while its stage record is alive.

Calling `Window::on_commit` on the unmap path is necessary: without refreshing
its cached bbox the no-buffer window still drew a shadow at the previous size.
The empty-frame oracle caught that separately after phantom-column repair.
Existing Smithay notes already document this bbox cache and renderer buffer API.

Dedicated `/tmp/driftwm-b4-protocol.log` passed: 12 real GLES scenes, SSD/CSD,
1x/1.25x/1.5x, ordinary held or actually rendered/recaptured mixed resize. An old
magenta popup is destroyed before the parent unmaps. The unmap frame is exactly
an independently rendered empty frame; all columns, snapshots, pending configures
and resize animations drain, with no keyboard focus/output membership. Fresh
configure precedes re-enrollment. Only the new blue body renders on remap, with
no red/green old body or magenta popup, correct SSD/focus/output membership and
no repeated animation on an unchanged commit. Fixture teardown checks resources.

Hermetic `protocol_unmap_releases_a_ready_peer_and_remap_waits_for_a_buffer`
checks two clients: unmapping the stalled participant releases the ready peer
without waiting for the deadline; a fresh empty commit still reserves no column.
Stage regression `untile_preserves_identity_and_repairs_selection_before_reenrollment`
covers last removal, stable IDs, empty session lifetime and a fresh column ID.
The existing grouped-operation property now includes untile histories.
`PROPTEST_CASES=4000 cargo test --lib stage::` passed
(`/tmp/driftwm-b4-protocol-stage.log`). Full `dev/scripts/ci-local` passed
(`/tmp/driftwm-b4-protocol-ci.log`, exit 0): 369 library cases, 1487 binary
non-soak cases, integrations and isolated soak, including strict all-target
clippy and locked build checks. No push, installation or manual TTY acceptance.

Next B4 gate: group supersession during retained motion with ready/pending peers;
output removal follows. These are bounded compositor integration checks; A0/P0
semantic audit still precedes feature expansion and full multi-output tiling
remains P11. GPU evidence is surfaceless NVIDIA, not hardware scanout.

## 2026-10-22 — B4 group supersession with ready and pending peers

Added real GLES regression
`superseding_held_or_mixed_tiling_groups_preserves_ready_and_pending_peers` in
`src/tests/render_pixels.rs`: 12 SSD/CSD scenes at 1x/1.25x/1.5x, two independent
clients and a blue popup. Existing hermetic
`superseding_a_group_releases_its_queued_buffer_and_keeps_pending_peers` already
checks queued-buffer release and deferred placement, but had no rendered pixels
or interrupted mixed-resize coverage.

Ordinary scenes enter tiling with one ready green buffer and the other client
stalled. Mixed scenes first answer both offers, tick 16 ms and actually render
the crossfade, then consume a neighbor to capture that moving picture into a
new group. In both variants the first client answers and remains blocked by the
second. Cycling column width supersedes the group. The old queued buffer is
released to committed geometry while the held full frame remains byte-identical
and all three parent/popup images remain retained. In mixed scenes the new
first-client width reply is independently queued, still unable to release the
frame until the second client answers. Both latest committed sizes are checked.

The release frame must stay within two bytes of the held frame. Latest buffers
use distinct cyan/yellow colors (ordinary first peer keeps green), so settled
frames independently reject obsolete magenta and mixed old green content while
retaining the blue popup. Stable stage IDs, structural/keyboard focus and actual
client-side output membership are checked for both surviving clients. Snapshots,
pending configures and crossfades drain; unchanged client commits cannot restart
animation or change the settled frame. Output GPU caches are removed before GL
uninstall and fixture teardown checks resource baselines.

Dedicated `/tmp/driftwm-b4-supersede.log` passed: the 12 GPU scenes and existing
hermetic supersession regression. No production change was required for these
controlled cases. Full `dev/scripts/ci-local` passed
(`/tmp/driftwm-b4-supersede-ci.log`, exit 0): 369 library cases, 1488 binary
non-soak cases, integration suites and isolated soak; formatting, locked build
and strict all-target clippy included. No pure stage/layout/config algorithm
changed, so no new deep property pass was required. No push, installation or
manual TTY acceptance. This is controlled compositor integration, not universal
transaction/lifecycle parity or hardware scanout evidence.

Next remaining B4 gate: sole-output removal during held/mixed motion and recovery
after re-adding an output. Independent multi-output tiling remains P11; A0/P0
semantic audit still precedes feature expansion.

## 2026-10-22 — B4 sole-output removal and recovery

`removing_the_only_output_cancels_held_or_mixed_tiling_and_recovers` reproduced
an actual bug: the last removed output remains mapped as a virtual placeholder,
so `refresh_tiling` still saw one active output and retained the tiling session,
images and pending group despite no renderer remaining for that monitor.
The shared `output_disconnected` policy now invokes the existing layout-
invalidation transition before output teardown. It cancels/wakes transactions,
restores canvas and drops tiling images. Both entry and refresh reject outputs
listed in `disconnected_outputs`, so the placeholder cannot admit a new session.
The existing output-leave ordering before disabling the global is preserved.

Dedicated `/tmp/driftwm-b4-output.log` passed: 12 real GLES scenes, SSD/CSD at
1x/1.25x/1.5x, ordinary held or actually rendered/recaptured mixed resize, two
independent clients and a blue popup. One client has a ready buffer queued while
the other stalls. Removal releases the ready buffer to its target geometry;
layout membership, snapshots, pending configures, animations/crossfades and
per-output animated GPU content drain. Actual client output memberships become
empty. Attempting ToggleTiling while only the placeholder exists is rejected.
Clients answer the canvas restore offers while headless. Re-adding an output
with the same name creates a different Output object and retires the placeholder.
Explicit tiling re-entry creates a fresh layout ID and keeps both window IDs.
After replies, focus agrees with selection, both clients enter the replacement
output, and the frame shows latest green/yellow bodies plus blue popup, without
obsolete magenta pixels. Unchanged commits neither restart animation nor alter
the settled frame. Renderer caches are removed before GL uninstall; fixture
teardown checks resource baselines.

Hermetic
`removing_the_last_output_releases_ready_buffers_and_rejects_placeholder_tiling`
independently covers queued-buffer release, original canvas size offers,
placeholder rejection and same-name reconnect/re-entry with stable IDs. Its
initial version accidentally acked a previously consumed configure after
re-entry; it now receives the new configure before acknowledging it, matching
the protocol handshake used by the GPU scenario.

Full `dev/scripts/ci-local` passed (`/tmp/driftwm-b4-output-ci.log`, exit 0):
369 library cases, 1490 binary non-soak cases, integration suites and isolated
soak, including formatting, locked build and strict all-target clippy. No pure
stage/layout/config algorithm changed, so no additional deep property run was
needed. No push, installation or manual TTY acceptance. Output removal is driven
through the actual shared disconnect policy and simulated global removal; these
are surfaceless NVIDIA checks, not actual DRM connector unplug/replug.

All named controlled B1–B4 gates are now delivered. Next is A0/P0 semantic audit
of custom decoders, public layout option names/defaults and ID-addressed actions,
then wider source-derived operation fixtures. Do not infer full P2/P13 acceptance
or niri parity, or restart renderer matrices instead of completing this semantic
prerequisite. Independent multi-output tiling remains P11.

## 2026-10-22 — A0 top-level layout option and decoder audit

Added `dev/docs/niri-layout-config-audit.md` with all 15 public `LayoutPart`
children, `Layout::default` values, compatible/missing/divergent driftwm evidence
and links to pinned source symbols. Audited `DefaultPresetSize`, `PresetSize`,
`Flag`, `FloatOrInt`, `Struts`, column-display parsing and the root layout decoder's
border exception. The reference checkout remains clean at
`12de51d6a60baad8896366920278a24f17921f52`.

Key findings for later implementation: reference presets are 1/3, 1/2, 2/3 rather
than driftwm's 1/3, 1/2, full; default gaps are 16 rather than 12. Omitted default
width inherits, an empty width wrapper explicitly clears it, while empty preset
lists restore built-ins. Flags can explicitly override to false. Struts accept
signed fractional values and replace as a whole, resetting omitted sides to
zero. Empty root `border {}` enables borders through a compatibility exception
that does not apply to includes; empty `shadow {}` does not enable shadows.
Workspace layout rejects insert-hint and empty-workspace-above-first. Public
fixed presets use content units; proportional presets use gap-aware tile units,
unlike driftwm's current frame-width policy. A source-derived 1200-wide/gap-16
half-width probe yields 576 reference tile pixels versus 600 driftwm frame pixels
with zero chrome, exposing a difference absent from the zero-gap trace corpus.

The manual inventory generator now links 30 Layout/LayoutPart rows and four
Struts rows to the audit, with complete top-level field-set checks. Known defaults
are classified conservatively; no Exact status was added. Nested appearance
field rows remain unverified pending their full grammar/merge audit. Removed a
stale generated reminder to add distinct consume/expel actions, which already
exist; their full boundary/ID/tab/view audit is still pending.

Validation: Python source compiles without writing bytecode; generation against
the clean pinned checkout is deterministic and byte-identical to the committed
inventory, still 186 actions / 318 named schema fields. Audited-row counts, local
link targets/anchors, pinned reference source targets and `git diff --check`
passed. This is source review and documentation/tooling validation, not execution
of the niri parser, a differential layout oracle or new Rust parity tests. No
runtime/config change, full CI rerun, push, installation or TTY acceptance. The
last full implementation gate remains `/tmp/driftwm-b4-output-ci.log`.

Next A0 batch: nested appearance decoders/merges (Color, BorderRule, ShadowRule,
TabIndicatorPart, InsertHintPart), then override lifecycle and the remaining
schema/action-addressing audit. Independent reference fixture expansion remains
required. A0/P0 is open; do not expand features or restart delivered B matrices.

## 2026-10-22 — A0 core appearance decoders and local merges

Added `dev/docs/niri-appearance-config-audit.md`, covering 98 named fields in
15 reference structs: Color, Gradient, GradientInterpolation, FocusRing, Border,
BorderRule, Shadow, ShadowOffset, ShadowRule, TabIndicator, TabIndicatorPart,
TabIndicatorLength, TabIndicatorRule, InsertHint and InsertHintPart. Public child
versus property names, ranges, defaults, helper enum spellings and local merging
are linked to pinned appearance/macros/utils sources. The checkout remains clean
at `12de51d6a60baad8896366920278a24f17921f52`.

Findings: Color accepts one delegated CSS string or exactly four u8 components,
with unpremultiplied internal float representation. Gradient uses required string
properties from/to, i16 angle default 180, relative-to window/workspace-view and
public `in` (not Rust `in_`). The custom interpolation parser permits four color
spaces and optional hue methods only for oklch. `Gradient::from(Color)` instead
uses angle 0. Present colors clear inherited gradients; a gradient in the same
part then wins independently of textual node order. A present shadow offset
replaces both axes, so x-only resets y to zero. Tab length and flag omission are
distinct from an empty length or explicit false flag.

Resolved Border/FocusRing/TabIndicator/InsertHint give on precedence when both
presence flags are set. Resolved Shadow gives off precedence; composing two
ShadowRule values instead normalizes flags with on precedence. This is recorded
as separate operations, with application contexts deferred to the override
lifecycle audit. It must not be silently flattened into one generic flag rule.
TabIndicatorRule is color/gradient-only, not a full TabIndicatorPart grammar.
Driftwm's hex-byte colors, two native border colors and shadow boolean remain
partial/narrower counterparts; gradients, urgent colors, separate focus rings,
tab indicators and insertion hints remain missing. No Exact claim was added.

The generator now links these 98 rows plus the prior 34 layout rows to the two
audits, and validates complete field sets for each scoped struct. CornerRadius,
workspace shadows, blur/background effects and other schema rows remain
unverified. Updated the earlier layout audit's stale pending-appearance wording.
Validation passed: Python syntax without bytecode artifacts, deterministic
byte-identical regeneration, 186 actions / 318 fields, 132 scoped audit links,
local targets/anchors, pinned reference source targets and `git diff --check`.
This is source review, not execution of niri or its parser, differential rendering
or a new Rust parity test. No runtime/config change, full CI rerun, push,
installation or manual TTY acceptance; the previous implementation CI gate
remains `/tmp/driftwm-b4-output-ci.log`.

Next: A0 global/output/workspace/window-rule override order, initial-map/reload
lifecycle and which merge path each context uses, then remaining schemas/action
addressing and independent fixture expansion. A0/P0 remains open and continues
to gate feature expansion; delivered B1–B4 renderer matrices are not restarted.

## 2026-10-22 — A0 override order and scoped rule lifecycle

Added `dev/docs/niri-config-lifecycle-audit.md` from clean pinned source
`12de51d6a60baad8896366920278a24f17921f52`. It traces Options global → monitor
output part → workspace part → scale adjustment, then composed per-window
appearance rules into resolved tile appearance. Output lookup selects the first
matching profile; full output/name grammar remains open. Scoped evidence covers
22 WindowRule fields, all nine Match properties and Output.layout/Workspace.layout
(33 additional source rows), rather than claiming all rule/placement decoding.

Rule matching is OR across match blocks, AND inside one, excluded by any matching
exclude. Ordered matching rules fold from a fresh default every recomputation,
with present scalars overriding and nested appearance rules composing. Unmapped
WindowRef has is-floating=false and active-in-column=true, so future floating
opening does not make an initial is-floating matcher true. Metadata, activation,
focus/selection, urgency/cast/floating state, startup completion and rule reload
have distinct recomputation paths. Window-rule shadow parts compose with on
precedence before applying to workspace appearance; layout parts merge directly
to resolved Shadow, locating the separate off-precedence path from the prior audit.

Initial configure stores resolved width/height/target fields separately from the
rules. Updating a configured-but-unmapped window's rules does not replace all
those cached sizing/target fields; first-buffer mapping revalidates output and
workspace and recomputes floating heuristics/focus policy. Opening defaults are
not generic dynamic re-placement/refocus commands. Preset reload clears indices
and converts preset heights to automatic. A further source-derived edge is kept
explicit: un-naming a nonempty workspace clears its name, not saved layout_config;
that path can retain the old override after config removal. These require
independent executable fixtures, not silent policy changes in this fork.

Driftwm comparison records ordered app/title glob-or-explicit-regex rules, stored
mapped appearance versus niri dynamic recomputation, global reload/cache refresh,
and the deliberate live pass_mouse rule exception. Per-output/workspace tiling
layout overrides and rule-level layout-native opening/sizing remain missing.
The generator links these 33 rows in addition to the earlier 132, guards Match
property completeness and selected WindowRule field existence, and retains
unscoped rows as unverified. The earlier audits link to the scoped lifecycle
trace without declaring every override/placement permutation resolved.

Validation passed: Python syntax without bytecode artifacts, deterministic
byte-identical generation, 186 actions / 318 fields, 34 layout + 98 appearance +
33 lifecycle audit links, local targets/anchors, pinned reference source targets
and `git diff --check`. No execution of niri/parser, differential rendering or
new Rust parity tests. No runtime/config change, full CI rerun, push, installation
or TTY acceptance; implementation CI remains `/tmp/driftwm-b4-output-ci.log`.
Next A0: remaining appearance/effect decoders and consumers (CornerRadius,
workspace shadows, blur/background effects), then remaining schemas/actions and
independent fixture expansion. A0/P0 stays open and gates feature expansion.

## 2026-10-22 — A0 radii, workspace shadows and effect selection

Added `dev/docs/niri-effects-config-audit.md` against clean pinned reference
`12de51d6a60baad8896366920278a24f17921f52`. It covers 34 fields in the seven
remaining named appearance structs and seven window/popup consumer fields:
41 new source-index links, 206 scoped links total. All 132 named public
appearance fields now have scoped decoder/default/local-merge evidence across
the core and effects audits, without claiming complete renderer/config parity.

CornerRadius accepts one or four positional numbers, clockwise TL/TR/BR/BL;
integers go through i16 while decimals use f32, negatives emit errors. Recorded
fit/expansion and windowed-fullscreen/geometry-clip consumers. WorkspaceShadow is
under overview.workspace-shadow, enabled by default; offset/softness/spread
normalize to logical workspace view height / 1080. Blur passes decode as u8 but
rendering clamps to 1–31; global tuning defaults differ from driftwm and do not
request every window's blur.

BackgroundEffect preserves optional blur/xray/noise/saturation overrides.
Nonempty client regions request blur unless explicitly vetoed; a rule can force
blur with no client region, but an explicitly empty region suppresses the effect
render. Visible normal-surface effects default xray on, while popups default it
off. Global blur.off gates blur execution rather than every other selected
effect; noise/saturation defaults depend on actual blur execution. Region dirty
state is committed before recomputation/damage. Capture block-out and clipping
branches are recorded without claiming every capture/transition path audited.
Driftwm's sticky rule OR global blur and empty/clipped-region fallback are
explicit divergences, not modified by this source audit.

The generator validates the seven complete struct field sets and seven scoped
consumer names, linking them to the effects audit. Validation passed: syntax
without bytecode artifacts, deterministic byte-identical regeneration, unchanged
186 actions / 318 indexed fields, 34+98+33+41 audit links, local targets/anchors,
pinned source paths and clean checkout, `git diff --check`. No execution of niri,
new Rust parity tests, runtime/config change or full CI rerun; implementation
validation remains `/tmp/driftwm-b4-output-ci.log`. No push, installation or TTY
acceptance.

Coverage finding: the generator indexes seven selected modules, not the entire
public config. Overview/OverviewPart live in misc.rs (though workspace-shadow
value types are indexed), and root Config/layer-rule surfaces are absent. Next
A0 task is schema coverage reconciliation with explicit exclusions and missing
layout-visible nodes before the remaining decoder/action audit and independent
fixture expansion. A0/P0 remains open and gates feature expansion.

## 2026-10-22 — A0 entire source/root schema reconciliation

Added `dev/docs/niri-config-schema-coverage.md` and expanded the generator from
seven selected modules / 318 named fields to 14 indexed modules / 576 fields.
Added root Config, misc, layer_rule, input, debug, binds and recent_windows:
258 additional rows. All previous 318 schema rows and 186 Action variants remain.
The manifest accounts for all 18 niri-config/src Rust paths, explicitly handling
error.rs, macros.rs, utils.rs and utils/merge_with.rs as infrastructure. error.rs's
four result/diagnostic fields are not configurable nodes and are excluded from
the schema row count. Custom scalar/merge helpers remain audited through their
consumers, rather than invented as independent public option trees.

The actual ConfigPart root dispatch has 25 public nodes mapped to 24 Config
fields; include is a custom directive without a backing field. The generator
extracts root arms and validates them against curated aliases/scopes, plus all
Config fields and the complete source-file set. Root output/window-rule/workspace
aliases are recorded as singular public nodes, not guessed from plural storage.
A generated module coverage table and root-node table expose every decision.

Mixed files cannot be blanket-excluded: input contains focus/warp/workspace
policy, debug contains transaction/resize/focus switches, misc contains Overview,
layer rules affect scene/backdrop/effects, binds has public action/switch-event
surfaces and recent_windows is relevant MRU focus UI. New rows remain Unverified
unless scoped evidence or explicit exclusions justify a label. Process launch /
environment, selection clipboard policy, screenshot destination naming, help UI
and config-error notification presentation are explicitly Outside, with related
window map/capture/overview behavior kept separately in scope. Overview/Part is
now visible as six Missing container settings; canvas zoom is not reference
overview equivalence. Its runtime consumers and layer semantics are next.

Validation: Python syntax without bytecode files; deterministic byte-identical
regeneration; 18 source coverage entries / 14 indexed modules; 25 root nodes / 24
storage fields; 576 unique module-qualified schema rows; preservation of previous
318 rows and Action table; unchanged 206 scoped semantic links; local targets /
anchors, pinned source paths, clean reference SHA and `git diff --check`. Source
reference remains `12de51d6a60baad8896366920278a24f17921f52`. This closes the known
source-index coverage gap, not complete public decoder/lifecycle or parity work.
No runtime/config change, full CI rerun, niri execution, push, installation or
TTY acceptance; implementation CI remains `/tmp/driftwm-b4-output-ci.log`.

Next A0: LayerRule/Match and Overview/OverviewPart defaults and actual consumers,
then input focus/warp/workspace policy, debug layout flags, animation/gesture /
remaining naming/rule decoders, public binding/action addressing and independent
fixture expansion. A0/P0 stays open and gates feature expansion.

## 2026-10-22 — A0 LayerRule and Overview consumers

Added `dev/docs/niri-layer-overview-audit.md` from clean pinned reference
`12de51d6a60baad8896366920278a24f17921f52`. The generator now links 13 complete
LayerRule/Match fields and six Overview/OverviewPart fields to scoped semantic
source evidence, raising the total from 206 to 225 links. Field completeness is
checked. The 576-field / 186-action inventory and full source/root coverage stay
unchanged; remaining input/debug/gesture/animation/rule/naming/action semantics
are still open.

Layer rules match namespace regex, startup flag and current protocol layer, with
OR match blocks, AND properties and exclude veto. Rules resolve on first mapped
buffer, committed layer change, rule reload and startup completion. Layer shadows
use global layout.shadow parameters but explicitly reset on=false before merging
layer rules, so enabled window shadows do not enable layer shadows. Backdrop
placement requires Background plus ExclusiveZone::DontCare; on-demand focus is
cleaned when the computed predicate becomes true. f32 opacity is clamped only
at rendering. Geometry radius affects shadow/effect geometry without forcing the
ordinary layer surface tree to be corner-clipped in the inspected render path.
Target block-out replaces body with a black solid and omits popups; full capture
pipeline acceptance is not claimed. Popup/IME distinction and bob motion are
recorded, not mistaken for tiling floating mode.

Overview's decoded [0,1] zoom is runtime-clamped to [0.0001,0.75], interpolated
from progress separately from its desired-open flag. Toggle/open/close/gesture
and completion paths are traced; closing can have a false desired flag while
still scaled. Monitor workspace geometry/render and selected pointer/focus
branches are source evidence, not full click/drag/DnD parity. Config propagates
options without that path toggling overview; backdrop reload chooses per-output
override then global color and forces buffer alpha to one. Workspace shadows
keep the previously audited view-height normalization.

Driftwm comparison explicitly distinguishes live namespace-based layer rule
lookup at render time from sticky mapped-toplevel appearance. Its opt-in native
layer chrome/canvas positioning is not the niri layer matching/backdrop grammar,
and free canvas zoom is not workspace overview. No runtime policy changed.
Validation passed: Python syntax, deterministic byte-identical generation, 576
unique schema rows, 186 actions, all source/root coverage sets, previous row keys
and scoped links, 19 new layer/overview links, local targets/anchors, pinned source
paths and clean checkout, `git diff --check`. No niri/parser execution, Rust parity
tests, full CI rerun, push, installation or TTY acceptance. Implementation CI
remains `/tmp/driftwm-b4-output-ci.log`.

Next A0: input focus/warp/workspace policy and layout-visible debug flags, then
remaining decoder/action addressing and independent fixture expansion. Complete
layer/overview interaction/capture permutations remain separate acceptance work;
A0/P0 stays open and gates feature expansion.


## A0 input focus and layout debug source audit

Audited pinned niri input defaults/optional-node replacement, Percent/mode shapes,
hover target/transition eligibility and working-area-normalized scroll threshold,
visual-rectangle cursor warp modes and per-output workspace back-and-forth.
Traced five debug flags through mapping, token validation, refresh and commit
blocking. The later individual configure-intent branch overrides scrolling's
earlier throttle bypass; this edge is recorded for executable reference fixtures.
Rule/token/strict precedence, ten-second activation expiry, urgency-only tokens
and activated-hint versus keyboard focus are distinguished. Driftwm's boolean
hover, camera pulls, activation adoption and bounded groups are not assumed
reference-compatible. No runtime or public config change.

The inventory adds 18 source-semantic links (243 total), retaining 576 field keys,
186 actions, 18 source paths / 14 indexed modules and 25 root nodes / 24 Config
fields. Python syntax, deterministic generation, old keys/action table, audited
field existence, local anchors/pinned source paths and clean reference checks
passed, as did git diff --check. No niri execution, Rust tests or full CI rerun
for this docs/tooling-only change; implementation evidence remains B4. No push,
installation or TTY acceptance. Next A0: animation/gesture consumers, remaining
mixed input/debug scope, names/rules/actions and independent executable fixtures.


## A0 animation and gesture source audit

Audited all 40 named animation and 23 gesture fields at the pinned revision,
including tuple-wrapper defaults/custom decoders that add no named field rows.
Whole-event replacement contrasts with fieldwise DnD merges; root on wins over
off but does not clear individual off. Spring/easing selection/default fallbacks,
curve/spring validation and custom shader source/file/reload failure behavior are
recorded. Shared clock slowdown/completion differs from unadjusted DnD time.
Selected edge view/workspace consumers clamp and normalize penetration, delay
nonzero entry, integrate speed and discard excess. Hot corners use output-wide
replacement, implicit top-left, one-pixel boxes, latch/focus/grab checks and
Overview toggles. Six notification/exit/screenshot UI-motion rows are explicitly
Outside; workspace overview, MRU, window transitions and capture privacy remain
in scope. Driftwm frame factors/actions/edge pan are not assumed equivalent.

Validation: Python syntax, deterministic generation, 576 preserved field keys,
186 preserved actions, 63 new review links (306 total), exact scoped field sets,
local targets/anchors, pinned source paths/revision/clean checkout and diff checks
passed. No runtime/config change; no niri/parser/shader execution or full Rust CI
rerun. Implementation evidence remains the B4 gate. No push/install/TTY claim.
Next A0: remaining mixed input/debug scope and output/workspace naming, then
rules/action addressing and independent executable reference fixtures. Physics,
reload and interaction acceptance still remain open.


## A0 client input, debug scope and output identity review

Audited six scroll/pinch fields and five output identity fields; triaged the
remaining 34 debug storage/Part fields against located consumers. Niri forwards
source-selected per-axis factors multiplied by a focused-window factor, and
pinch scale raised to combined sensitivity. This is distinct from canvas speed.
Output matching uses ASCII-insensitive connector or full metadata string, first
matching entry, with connector-only fallback when all metadata is absent.
Workspace duplicate detection/named lookup and preference-order creation were
traced without double-counting already reviewed lifecycle rows. Full action
parsing, persistence and hotplug permutations remain open.

Only the two DBus-export switch rows are newly Outside. Scanout/timing/format/
capture rows retain Unverified status, while missing topology/name/preview knobs
are explicit. Debug merge/default behavior is separated from actual reload
application. A debug label does not waive layout-visible integration acceptance.

Validation passed: Python syntax, deterministic generator output, all 576 previous
field keys and 186 actions, 45 new review links (351 total), field existence,
local links/anchors, pinned source paths and clean reference revision, diff check.
No runtime/config change, niri/parser execution or full Rust CI rerun; latest
implementation evidence remains B4. No push, installation or TTY acceptance.
Next A0: remaining keyboard/device grammar/consumers, unscoped output/rule fields,
public binding/action addressing and independent executable reference fixtures;
retained integration/physics/interaction acceptance gates remain open.


## A0 remaining keyboard and device input source audit

Reviewed the remaining 89 named input fields, completing source links for all
103 input rows (440 scoped review links overall). Traced keyboard field merge
versus whole Xkb/device replacement, defaults/numeric/enum shapes, repeat zero,
per-surface layout restoration, keymap file/locale1 reload and keyboard state,
backend Mod defaults and power fallback. Device classification and setter/default
paths, OnButtonDown-only button application, six-value calibration runtime
fallback and tablet/touch geometry/target precedence remain separate from parser
acceptance and actual libinput support. Most device rows stay Unverified rather
than asserting equivalence. Driftwm repeat-delay default differs (200 vs 600 ms);
per-surface track-layout policy remains missing. No runtime/public config change.

Validation passed: Python syntax, byte-identical deterministic generation, exact
combined input field coverage, 576 preserved field keys / 186 actions, 89 new
review links, local anchors/pinned source paths, clean pinned checkout and diff
checks. No niri/parser/hardware execution or full Rust CI rerun for docs/tooling;
implementation evidence remains B4. No push/install/TTY claim. Next A0: unscoped
output/window-rule fields, public action/binding addressing and independent
reference fixtures; retained input/backend/capture/interaction gates remain open.
