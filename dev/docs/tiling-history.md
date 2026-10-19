# Tiling development history through 2026-10-19

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
