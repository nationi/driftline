# Niri tiling parity: execution plan

Status (2026-10-22): integration checkpoint `8c17428`; named controlled B1–B4
stabilization gates are delivered. A0 top-level layout and core appearance
decoder/local-merge, effects and scoped lifecycle audits are recorded. Source/root
coverage is reconciled; scoped layer/overview and input focus/layout debug
consumers and animation/gesture configuration are audited. Named input fields
and output/window-rule fields have source-review coverage; input/backend
acceptance, remaining misc/root/action-family semantics and executable fixtures
stay open.
Single-container topology and bounded commit groups are established; reference
semantics and rendering acceptance remain partial.
This document supersedes the remaining delivery sequence in
[the initial proposal](scrolling-tiling-plan.md). It does not claim parity today.
For current handoff context and the next unfinished task, read
[tiling development status](tiling-status.md).

## Goal and reference

Reproduce the observable behavior of niri's scrolling layout, including its
keyboard operations, size policy, gestures, interactive manipulation, tabbed
columns, floating integration, fullscreen, output handling and layout-facing
workspace/overview behavior. Keep driftwm's canvas available as a separate mode.
A horizontal strip with similar-looking windows is insufficient for completion.

Reference: niri commit `12de51d6a60baad8896366920278a24f17921f52`, checked out at
`/tmp/niri`. Pin fixtures and expected behavior to this revision. Do not silently
change the target whenever upstream changes. Features labeled unreleased in that
revision must be identified explicitly in the inventory. Later upgrades require
an explicit baseline update and a delta report.

Primary source map, relative to that checkout:

| Source | What to verify there |
| --- | --- |
| `src/layout/scrolling.rs` | Column operations, view offsets, sizing, gestures, resize and invariants |
| `src/layout/tile.rs` | Per-window target state, content/frame conversion and rendering |
| `src/layout/workspace.rs` | Floating/tiling routing, output scale, activation and layout transitions |
| `src/layout/floating.rs` | Floating placement, geometry and focus |
| `src/layout/monitor.rs`, `src/layout/mod.rs` | Output/workspace ownership and transfers |
| `niri-config/src/binds.rs`, `layout.rs`, window rules | Complete action and option inventory |
| `src/utils/transaction.rs`, layout tests | Bounded asynchronous updates and test cases |
| `docs/wiki/` | User-observable semantics and examples at the pinned revision |

[Source tree](https://github.com/niri-wm/niri/tree/12de51d6a60baad8896366920278a24f17921f52/src/layout),
[action definitions](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs),
[pinned user documentation](https://github.com/niri-wm/niri/tree/12de51d6a60baad8896366920278a24f17921f52/docs/wiki).
Live documentation is useful for navigation but cannot override the pinned code:
[layout](https://niri-wm.github.io/niri/Configuration:-Layout.html),
[gestures](https://niri-wm.github.io/niri/Gestures.html),
[tabs](https://niri-wm.github.io/niri/Tabs.html).

## Scope decisions

The goal covers layout behavior, not every feature of the niri compositor.
Screen capture backends, launchers, lock screens and unrelated protocols remain
separate projects. Layout-visible protocol effects, input routing, capture
visibility of tabs and workspace events are in scope.

For planning, include niri-like workspaces and workspace overview inside tiling
mode. This is a proposed extension of driftwm's original no-workspaces policy,
not an already approved architectural design. Resolve it before the workspace
implementation milestone. Omitting it would mean reporting scrolling-layout
parity rather than full niri tiling UX parity. Do not quietly relabel bookmarks,
canvas regions or zoom-to-fit as equivalent workspaces/overview.

Preserve canvas semantics outside tiling. Inside tiling, floating windows need a
real layout policy rather than merely being excluded from column membership.
Likewise, unsupported operations must eventually stop exiting tiling as a
fallback. Keep an explicit compatibility table for driftwm-only actions such as
fit/fill, pin, snap clusters, bookmarks and suspended stand-ins.

## Current baseline and known differences

Integration checkpoint: `8c17428`. Current automated/manual evidence and its
limits live only in [the maintained status](tiling-status.md#verified-evidence-and-its-limits).
A passed CI suite is not a parity verdict. Release-build and TTY observations
from older entries do not validate later commits automatically.

| Area | Current state | Remaining parity work |
| --- | --- | --- |
| Topology | Stable column/layout IDs, separate selection, column/tile reorder | Swap and complete reference actions; multi-container ownership |
| Consume/expel | Existing focused move/split plus distinct reference first-consume/last-expel topology and focus | Reference sizing/animation, directional combined semantics and context coverage |
| Width | Hardcoded 1/3, 1/2, 1 presets; integer frame widths | Configurable forward/back presets, reference defaults, sizing units and gaps |
| Height | Equal content-space allocation with min/max | Automatic weights, fixed/preset heights, frame-space weighting and reset |
| View | Reveal focused rectangle; horizontal generic momentum | Column-relative anchoring, centering modes, snap physics and interruption |
| Mapping | Initial tiling size offer; enrollment after first buffer using mapping-time focus | Broader rule/context coverage; preserve no-phantom-column behavior |
| Client response | Serial/generation tracking, 300 ms commit groups, deferred placement and retained images | Full rendering/transition acceptance and reference response policy |
| Floating | Dialogs and special windows excluded from tiling | Stationary floating layer, explicit toggles and focus routing |
| Fullscreen/fit/pin | Most conflicting operations leave the mode | Fullscreen/full-width/maximize transitions retaining topology |
| Input | Focus four directions, width cycle, merge/split | Full action surface, interactive resize, drag insertion, touch and tablet |
| Tabs | Missing | Tabbed columns, indicator, visibility and fullscreen behavior |
| Outputs | One output; a second output exits tiling | Independent owners, transfers, scale changes and disconnect/reconnect |
| Workspaces/overview | Missing in tiling | Explicit model/design decision and then full interaction coverage |
| Configuration/IPC | Bindings documented; no complete tiling API | Layout settings, rules, selectors, inspection and events |
| Session | Original canvas geometry/view saved | Versioned topology, owner and sizing restoration |

Two concrete semantic traps must be captured before adding aliases:

- In the pinned implementation, `consume_into_column` pulls the first window
  from the next column into the current column. Our move action moves the
  focused window in the opposite ownership direction.
- The pinned `expel_from_column` operates on the last window in the column;
  directional consume-or-expel variants have their own branching behavior.
  Our existing action expels the focused window. Preserve its documented
  behavior, adding distinct reference-compatible actions where necessary.

## Architectural requirements

1. `Stage` remains the owner of membership, order, focus selection and saved
   geometry. No second authoritative window registry; no `Space` window APIs.
2. Give columns and layout containers stable IDs independent of their selected
   `ElementId`. Stable `ColumnId` and separate per-column selection are now
   implemented. `TilingLayoutId` now survives an empty strip and changes only
   when a layout ends and a new one begins; multi-container ownership is future work.
3. Pure layout consumes topology, constraints, settings and work area, producing
   geometry and view decisions. `state/tiling.rs` applies protocol effects.
4. Distinguish desired layout, configured size, acknowledged configure and
   committed geometry. An acknowledgement does not mean the new buffer exists.
5. One sizing authority per window. Seeing a window through another viewport
   never authorizes that viewport to resize it.
6. Keep coordinates explicit: strip-local, canvas, output-local logical and
   physical. Establish a fractional-position/rounding policy before implementing
   exact proportions and scale transitions; current integer geometry is a limit.
7. Replace the broad action allowlist and scattered exit hooks with documented
   transition policy as supported operations grow. Do not leave two size owners
   active or let a generic canvas reflow reposition a tiled window.
8. Reflow must preserve z-order and focus; animations must preserve the existing
   input contract unless a separately reviewed design change replaces it.
9. No unbounded waits, configure storms, event-loop blocking or retained state
   after a client dies. Explicitly cancel stale transactions and grab targets.
10. Preserve popup-inclusive visibility/hit-testing and route output enter/leave
    through `refresh_window_outputs`.

## Milestones, in dependency order

### P0 — Freeze the behavioral contract

The [source inventory](niri-tiling-inventory.md) indexes all 186 reference Action
variants and 576 named schema fields in 14 option-bearing modules. The
[schema coverage audit](niri-config-schema-coverage.md) accounts for all 18 source
files and 25 root decoder nodes / 24 Config fields, with explicit infrastructure
and non-layout exclusions. This is source indexing, not completed public decoding.
Its statuses are conservative; a complete custom-decoder audit and reference operation fixtures are still pending.
The [configuration audit](niri-layout-config-audit.md) covers all 15 top-level
layout names/defaults and shared size/flag/strut decoding. The
[appearance audit](niri-appearance-config-audit.md) covers core color/gradient,
border/shadow and tab/insert-hint grammars and local merges. The
[lifecycle audit](niri-config-lifecycle-audit.md) traces scoped override order,
matching and initial-map/reload consumers; complete permutations, remaining
schemas and action addressing remain open. The [effects audit](niri-effects-config-audit.md)
covers remaining named appearance structs and selected window/popup consumers;
the [layer/overview audit](niri-layer-overview-audit.md) traces those scoped
matching/reload/render/state consumers. Remaining indexed semantics and full
interaction/capture permutations stay open.

Tasks:

- Inventory every layout-related action and option at the pinned revision.
  Include ID-targeted variants, previous/next/index addressing and boundary
  behavior. Record reference symbol, driftwm equivalent, status and test ID.
- Write operation traces covering map, focus, close, group, split and resize.
  Record topology, focus, target sizes and view position at every step.
- Record deliberate driftwm-specific deviations separately from missing work.
- Establish reference fixture generation outside the regular test suite; commit
  hermetic expected data. CI must not download or launch niri to test driftwm.
- Keep existing action names working. Do not change an action's meaning merely
  to make its name resemble niri.

Done when: the inventory accounts for the entire pinned action/option surface;
existing implemented behavior is labeled exact, partial or divergent with evidence.

### P1 — Stable topology and centralized transition policy

Stable column IDs, separate selection, horizontal column/vertical tile reorder
and horizontal view-anchor preservation are implemented with stage and headless
tests. A shared transition gate now covers actions, direct operations, IPC and
layout invalidation, with the current fallback behavior documented in the user guide. A complete
action-by-action compatibility table still belongs to the semantic audit.
Stable layout-container identity and separate `Column`/`Tile` sizing inputs
are now implemented. The single-container P1 foundation is in place. Retained
floating/fullscreen transitions remain with their feature milestones; this does
not close the P0 semantic audit or imply support for multiple containers.

Depends on P0.

- Add `ColumnId` and a stable layout-container ID; separate active column from
  each column's selected tile. Provide ID-safe lookup and mutation primitives.
- Preserve selection, saved canvas geometry and IDs through remove, replace,
  suspend/adopt, reorder, merge and split. Specify which identity survives merge.
- Separate single-window sizing inputs from column inputs in the pure API.
- Centralize transitions among canvas, tiled, floating and temporary fullscreen
  state. Add an explicit action compatibility table instead of implicit exits.
- Capture viewport anchors before structural changes; avoid shifting the user's
  view when a column to the left disappears or changes width.

Done when: operation histories retain valid IDs and selection, reflow does not
raise windows, and deleting/replacing any tile cannot strand an anchor or grab.

### P2 — Configure lifecycle and bounded layout transactions

Implemented foundation (partial acceptance):

- Layout generations, configure serials, pre-commit barriers and one bounded
  300 ms deadline coordinate new-buffer replies. Early acknowledgements do not
  count as applied buffers. Supersession preserves the earliest deadline;
  timeout, participant removal and mode exit release waiters.
- Ordinary initial size offers precede the first buffer. Enrollment and neighbor
  selection happen at mapping time; no phantom columns preserve configure order.
- Placement and view changes wait for buffer application or timeout. Retained
  parent content, separate live popup updates, native frames and live backdrop
  blur preserve supported transitions. Failed/degraded captures retain fallback.
- Old/new content blends in premultiplied RGBA. Native SSD/CSD chrome, inherited
  opening, repeated recapture and client-only mask blending have controlled
  static-scale pixel gates. Late responses can start one bounded transition.

Remaining acceptance includes global-default reload and mixed-title lifecycle,
general frame conversions, stale/capped images, changing views/output scales and
motion-time lifecycle changes. The actionable checks are in the queue below;
[test evidence](tiling-status.md#verified-evidence-and-its-limits) and
[historical investigations](tiling-history.md) stay outside the roadmap.
No unconditional frame-atomicity or complete reference response-policy claim is
made by these implementation tests.

Depends on P1; required before advanced resize/drag/animations.

- Apply tiling rules before the first configure. Avoid briefly opening at a
  canvas position and immediately sending another independent placement request.
  Keep initially configured windows outside column membership until their first
  buffer, as the reference does. Recheck late parent/floating hints and choose
  the ordinary insertion neighbor from mapping-time focus; do not reserve blank
  columns merely to preserve configure order.
- Track layout generation and relevant configure serials; ignore stale completion
  effects after a newer resize, transfer, fullscreen transition or mode exit.
- Group related window geometry changes without blocking the event loop. Give
  transactions a deadline and deterministic timeout/cancellation behavior.
- Test clients acknowledging early, delaying buffers, rounding to cell sizes,
  refusing sizes, remapping and disconnecting mid-transaction.
- Compare reference behavior against our current live-frame reservation policy.
  Keep safety guarantees, but do not assume indefinite reservation equals niri.

Done when: rapid interleaved operations settle to the latest request, a stalled
client cannot freeze others, and repeated unchanged input sends no new configure.

### P3 — Exact sizing model and configuration

Depends on P0–P2.

- Add a dedicated tiling settings section: initial width, width/height presets,
  gaps, struts, default column display and centering policies.
- Match reference defaults, including 1/3, 1/2 and 2/3 width presets; the current
  hardcoded full-width preset is not a substitute for a separate full-width toggle.
- Implement fixed and proportional sizes, relative/absolute changes and forward/
  reverse preset cycling. Verify the reference distinction between window size
  and frame size; never blindly reuse our current frame-width units.
- Account for gaps in proportional widths so fitting multiple fractions matches
  the reference, rather than placing two half-width frames plus an extra gap.
- Add automatic weighted tile heights, fixed heights, height presets and reset
  to automatic. Weights apply in the correct frame/content space.
- Implement full-width toggle, expand-to-available-width, window width/height
  actions and maximize-to-edges where relevant to the selected layout.
- Define fractional logical placement and physical-grid rounding across borders,
  title bars, gaps, usable area and output transforms. Do not round repeatedly
  from previously rounded results when scale changes.
- Resolve min/max conflicts and impossible allocations against pinned cases.
  Recheck oversized-window behavior rather than blessing the current fallback.
- Regenerate `docs/config.md` from `config.reference.toml` after every option change.

Done when: sizing fixtures match the reference within documented physical-pixel
rounding, including mixed chrome, terminal increments and fractional scale.

### P4 — Complete topology and keyboard operations

Depends on P1; sizing-related operations also require P3.

- Move columns left/right, to first/last and by index; reorder windows up/down.
- Add exact consume, expel, directional consume-or-expel and swap operations.
- Add first/last/index column focus, top/bottom/index window focus, previous
  window and explicit wrap/fallback variants from the inventory.
- Add centering of a column, window and visible columns; under-pointer and
  monitor-fallback focus variants as their dependencies become available.
- Preserve focus/selection and viewport anchoring at boundaries and when removing
  the last window. Support explicit ID targets without requiring focus theft.

Done when: each inventory action has traces for normal, boundary, empty, dead-ID,
fullscreen and tabbed contexts. Context-dependent rows may finish in P7/P8/P11.

### P5 — View state, gesture physics and centering

Depends on P1–P3; interactive source support continues in P9.

- Introduce explicit static, gesture-controlled and animated view states anchored
  to a stable column. Reuse driftwm's camera application path, not another render
  transform or independent conflicting camera animation.
- Implement reference centering modes and single-column centering.
- Track gesture velocity/time, projected stopping position, snap candidates,
  overscroll resistance and spring settling. Respect touchpad versus mouse units.
- Define interruption by a new gesture, keyboard focus, application activation,
  opening/closing windows, fullscreen, resize, screen lock and output removal.
- Support mouse view dragging and configurable scroll factors; preserve intended
  focus selection while panning. Avoid a continuous focus/scroll feedback loop.

Done when: deterministic time-driven tests settle at reference alignment points,
rapid reversals remain continuous and structural changes do not jump the view.
Manual validation must use real trackpad input as well as a nested session.

### P6 — Layout-native interactive resizing

Depends on P2/P3/P5.

- Route tiled resize to column width or tile height instead of exiting the mode.
- Handle compositor resize borders, client `xdg_toplevel.resize`, touch and the
  relevant configured gesture paths through one policy.
- Preserve the anchored edge, update auto weights/fixed sizing appropriately and
  propagate column-width changes to its windows.
- Implement reference double-click edge actions and height reset behavior.
- Specify cancellation and ownership when a client dies, a new configure arrives,
  focus changes, fullscreen starts or the output changes during a resize.

Done when: dragging every edge/corner is continuous, respects constraints and
settles without reviving stale canvas resize compensation or leaking grabs.

### P7 — Fullscreen, full-width and maximized states

Depends on P2–P5; tabbed cases complete with P8.

- Fullscreen a tiled window without abandoning its column membership.
- Retain column width, tile heights, selection, view anchor and restore state.
- Distinguish full-width column, protocol fullscreen, maximized and windowed
  fullscreen behavior from the reference; do not equate them with canvas fit.
- Handle application-initiated requests, repeated toggles, close, transfer,
  output change and stale commits during entry/exit.
- Resolve driftwm fit/fill/pin explicitly: supported mapping, documented separate
  operation or rejected request. Do not silently exit tiling for normal niri actions.

Done when: all transitions preserve topology and round-trip to the correct target
geometry/view under delayed commits and client refusal.

### P8 — Tabbed columns

Depends on P2/P3/P7.

- Add normal/tabbed display state and default/rule configuration per column.
- Size and select tabs according to the reference; preserve merge/split, focus,
  reorder and height metadata across display changes.
- Render a configurable tab indicator with pointer selection, inside/outside
  placement, single-tab policy and correct damage/scale handling.
- Audit hidden tabs across rendering, input, frame callbacks, output membership,
  foreign-toplevel state, screenshots and capture. Geometry overlap is not a
  sufficient visibility policy.
- Implement tabbed fullscreen and close/crash of the selected tab.

Done when: hidden tabs cannot receive stray input or appear in capture, and all
column operations preserve tab identity and restore state.

### P9 — Interactive move, insertion and pointing devices

Depends on P2/P4–P8.

- Drag tiled windows within/across columns with insertion previews and stable
  insertion targets. Include empty space, column edges and vertical slots.
- Drag complete columns where the reference supports it, with correct view follow.
- Scroll near edges during move and client DnD; distinguish those protocols and
  their cancellation rules. Implement hold-to-activate behavior.
- Support changing tiled/floating destination during a drag.
- Cover mouse, touchpad bindings, touchscreen and tablet paths present in the
  baseline; label unreleased reference behavior explicitly.
- Re-pick targets after scale, panel, topology and output changes. Preserve
  destination-based hit-testing policy or resolve its conflict before changing it.

Done when: previews agree with final insertion, no drop duplicates/loses a tile,
and cancelling any grab leaves topology, focus and constraints consistent.

### P10 — Floating integration and window rules

Depends on P1–P4; drag integration depends on P9.

- Provide tiled-to-floating, floating-to-tiled and focus-between-layout actions.
- Implement stationary floating geometry above the scrolling layer in tiling
  mode. Canvas-floating behavior outside that mode remains unchanged.
- Add reference automatic placement rules for dialogs/fixed-size windows and
  explicit rules overriding the heuristic.
- Support floating positioning/sizing, saved tiling slot and reattachment policy.
- Implement default width/height/display, opening destination and activation
  rules at the appropriate lifecycle boundary, including application retitles.
- Keep modal chains, widgets, pinning and suspended stand-ins explicit; do not
  accidentally enroll a window because an unrelated action triggered refresh.

Done when: panning does not move a tiling-mode floating dialog, transitions affect
only the requested window, and rules/activation do not steal another window's slot.

### P11 — Multiple outputs and layout ownership

Depends on P1–P3/P7/P10. Resolve ownership design before implementing it.

- Choose explicit strip owners and a mapping between strip-local geometry and
  the shared canvas. Secondary views do not become additional sizing authorities.
- Decide whether other viewports observe a strip as canvas content or isolate it
  like a niri output. Record the difference if observation remains supported.
- Add independent output layouts and focus/transfer actions for windows and
  whole columns, including by name and next/previous/directional variants.
- Preserve original output preference across disconnect/reconnect, including
  removal of the last real output and virtual-placeholder cleanup.
- Reflow for mode, transform, scale, usable-area and panel changes without losing
  identity, focus, pending transactions, fullscreen or floating restore state.
- Audit output enter/leave, preferred scale, presentation and popup placement.

Done when: two unequal-scale monitors can operate independently, transfers are
atomic, and repeated hotplug never produces two size owners or orphaned tiles.

### P12 — Workspaces and overview in tiling mode

Depends on the scope/design decision, P5/P9/P11 and stable container IDs.

- Design dynamic tiling containers with independent strips/floating state and
  per-output selection; decide their relation to the infinite canvas explicitly.
- Add workspace creation/removal, naming, index/previous focus, movement/reordering
  and window/column/container transfers from the reference inventory.
- Preserve output return preferences and fullscreen/selection while moving a
  workspace. Cover empty-workspace lifecycle and named empty containers.
- Implement overview transforms, selection, drag insertion and workspace gestures.
  Keep visual transforms, popup/input coordinates and focus recovery consistent.
- Map only relevant layout state into exposed workspace protocols/IPC.

Done when: reference workspace/overview traces pass on one and several outputs.
If this milestone is intentionally excluded, report the narrowed parity scope;
do not mark the full UX goal complete.

### P13 — Animation and visual behavior

Depends on P2/P5; feature-specific animations accompany P6–P12.

- Match transitions for column/tile movement, consume/expel, resizing, open/close,
  tab switches, fullscreen, drag insertion, overview and output/workspace transfer.
- Preserve continuity when an animation is interrupted/reversed or its target
  changes. Use bounded transactions without starving unrelated windows.
- Align rendering and damage with the physical grid; verify border/focus/tab
  indicators, rounded corners, shadows, popups and clipping at output edges.
- Audit interaction with driftwm's existing destination hit-testing, pointer pull,
  client locks/confines and crossfade captures. No second focus-refresh mechanism.

Done when: deterministic animation tests and exported pixel fixtures cover the
transitions, with manual comparison on actual hardware for motion quality.

### P14 — IPC, persistence and integration completeness

Incremental throughout; close after P4–P13.

- Expose stable output/container/column/window identity, topology, selected tile,
  display mode, sizing policy, floating state and current/target view.
- Provide action selectors, structured errors and change events for the complete
  layout inventory. Do not require keyboard focus for ID-targeted operations.
- Version persistent topology. Store stable persisted identities, not runtime
  `ElementId` values. Restore widths, height policies, tabs, ownership and view.
- Reconcile missing apps, duplicate identities, relaunch/adoption, disabled
  outputs and changed configuration without losing the saved canvas fallback.
- Document bindings, precedence, rules, migration and any intentional differences.
  Keep the config reference and generated documentation synchronized.

Done when: restart and automation reproduce the layout, and all inventory rows
have a public interface and documented behavior or an explicit scope decision.

### P15 — Parity audit and release gate

- Run the reference trace corpus against the entire supported action/option
  inventory. Review every partial/deviation row; do not hide gaps behind tests
  that merely mirror driftwm's implementation.
- Exercise mixed operations under randomized topology, client response ordering,
  hotplug and config reload. Replay every regression seed in CI.
- Run complete local CI and the deeper layout/stage/config property pass. Keep
  ignored tests and the isolated soak test in the gate.
- Run manual nested tests, real trackpad tests on udev, multiple monitors with
  mixed scaling and real applications including cell-snapping terminals, GTK,
  Chromium, games, dialogs and X11 clients through satellite.
- Verify idle behavior and reflow/input cost under a larger window population;
  no wakeup loops, unbounded caches or process-wide resource growth.
- Publish the final parity matrix and remaining intentional differences.

Done when: every in-scope baseline item passes its evidence gate, canvas behavior
still passes regression tests and the remaining differences are explicitly accepted.
Only then describe the implementation as complete niri tiling parity.

## Execution queue and acceptance gates

The [preset/topology action audit](niri-preset-topology-action-audit.md) traces
23 preset/reset/maximize/consume/expel/swap consumers. Directional/centering and
remaining action families plus executable acceptance stay open.

The [size/transfer action audit](niri-size-transfer-action-audit.md) traces change
scalar parsing, context-specific units and 16 selected transfer/size consumers.
Presets/topology/directional families and executable reference acceptance stay open.

The [binding/address audit](niri-binding-address-audit.md) covers named bind
fields, config exposure and 12 scoped ID/ordinal/previous/workspace consumers.
Remaining action families/arguments and complete acceptance contexts stay open.

The [output/window rule audit](niri-output-window-rule-audit.md) completes review
links for those named module fields. Dynamic consumer, decoder diagnostic,
protocol and DRM/render acceptance remains open despite source coverage.

The [device input audit](niri-device-input-audit.md) completes named input source
coverage. Hardware support, keymap/protocol behavior and remaining Unverified
integration rows still require independent acceptance.

The [input/output integration audit](niri-input-output-integration-audit.md)
traces client scroll/pinch delivery and output/workspace identity. Remaining debug
rows have field-specific scope triage; retained Unverified backend gates stay open.

The [animation/gesture audit](niri-animation-gesture-audit.md) records defaults,
replacement versus merge, custom decoder rules, clock and scoped DnD/hot-corner
consumers. Numerical/interaction execution and remaining schemas stay open.

The [input focus audit](niri-input-focus-audit.md) traces optional-node replacement,
hover scroll eligibility, cursor modes, workspace back-and-forth and five debug
flags. Remaining device/backend fields and executable acceptance stay open.

Existing P2/P13 stabilization may continue while P0 is incomplete. P0 is the
semantic prerequisite before expanding sizing, interactions or ownership; the
implemented P1/P2 subsets do not waive milestone dependencies. Do not keep adding
renderer work without closing a named remaining gate.

| Order | Task | Completion evidence |
| --- | --- | --- |
| B1 — controlled gates delivered | Mixed window-rule/client blur and native SSD title bands; global-default reload while held, mixed-title exit/disconnect | Pixel matrices below and full local CI; broader lifecycle remains B4 |
| B2 — controlled gates delivered | General framed canvas resize entry, odd extents/nonzero geometry; 12 solid and 12 patterned fractional-client scenes | SSD/CSD grow/shrink, changed offsets, recapture release, popup size/anchor and independent settled transparency/opacity; static scales only |
| B3 — controlled gates delivered | Capped/minified stale fallback, client-region blur, view/output matrices and five scoped capture allocation failures | 90 allocation scenes cover reply/timeout/exit and subsequent recovery; content-only fractional bake padding fixed; GPU-offscreen/mode evidence remains distinct from hardware scanout |
| B4 — controlled gates delivered | Direct remap, ready-peer disconnect, protocol unmap/remap, group supersession and sole-output removal/recovery during held/mixed resize | 12 GPU scenes per lifecycle matrix plus hermetic restore/peer/output and stage re-enrollment regressions; output loss leaves tiling and rejects placeholder enrollment; these bounded gates do not prove every lifecycle combination or real DRM hotplug |
| A0 — in progress, semantic prerequisite | Layout/appearance/effects/scoped lifecycle, layer/overview, all named input, layout debug, animation/gesture and output identity consumers reviewed; remaining decoders/actions/fixtures and backend acceptance stay open | 496 scoped reviewed rows; 51 scoped action consumers (including explicit exclusions and retained Unverified integration gates); inventory covers 576 fields in 14 modules, all 18 source paths and 25 root nodes / 24 Config fields; source coverage is not an Exact claim or executable oracle |
| Feature expansion | P3 sizing/settings, then dependency-ready P4/P5 work | Source-derived size/topology/view fixtures and documented public settings; retain canvas regressions |

A0 can be taken before any stabilization batch, and must be completed before
feature expansion. Keep each B task bounded; if an uncovered case needs a new
design decision, record that blocker rather than declaring the whole gate done.
Later P6–P15 retain their dependencies and acceptance criteria above.

### B1: delivered controlled gates

Delivered subset: 18 controlled SSD/CSD cases now retain title coverage separately
from body texels through grow/shrink, opening, immediate/mid-animation recapture
and client partial ↔ mapped-rule fallback transitions. Another 12 SSD/CSD scenes
cover global-default enable/disable reload while held, and 12 SSD scenes cover
exit/disconnect while retaining a mixed native title. Broader remap, supersession
and output-removal lifecycle coverage remains B4.
Mapped rules remain sticky and apply at mapping time; changing their TOML does
not reapply flags to existing windows. Pinned niri's tri-state/empty-region policy
differs from driftwm's compatibility policy; this batch does not resolve A0.

- Compare the pinned reference's background-effect/resize behavior with driftwm's
  retained-frost policy. Separate compositor integration from proven niri parity.
- Exercise client partial → unset/empty/outside regions while a window rule still
  requests blur, whole-window → partial, and rule changes during a held group.
  Confirm which policy wins from code/config semantics before changing it.
- Check SSD title-band coverage separately from the body. Native title height,
  border and shadow must survive; client-local regions must not leak into chrome.
- Cover SSD/CSD at 1x/1.25x/1.5x with rounded frames, opacity, transparent source
  pixels, opening and both resize directions. Use independently expected coverage
  in addition to image comparisons.
- Compare first capture, held region-only commits, first release, intermediate
  mask weights, immediate/mid-animation recapture and settled state. Keep current
  two-byte transition tolerances and exact held-frame comparisons.
- Verify mask/cache cleanup on completion and cancellation. Run the required
  CI gate, then update status/history and affected user documentation.

Use [current status](tiling-status.md) for the continuation point and
[the TTY guide](tty-preview.md) for local keys. Direct TTY visual checks and
surfaceless GPU regressions do not close the unresolved nested NVIDIA issue.
No weighted completion percentage exists. A partial gate does not close a
milestone, and this queue does not authorize publishing, pushing or installation.

## Working rules for continuing agents

- Read this plan, `tiling-status.md`, `AGENTS.md`, caveats and
  testing rules. Inspect the working tree before staging or editing shared files.
- Update status and evidence after each delivered milestone. Mark partial work
  honestly; a safety fallback is not a reference-compatible implementation.
- Test pure algorithms, stage policy, headless protocol sequences and relevant
  pixels at their own layers. Use injected clocks; no sleeps in deterministic tests.
- Complete meaningful checks before committing; do not include unrelated changes.
  Follow upstream contribution/issue conventions before proposing upstream work.
- Do not create external issues, publish or push solely because this plan lists
  them. Use the user's authorization for those external actions.
