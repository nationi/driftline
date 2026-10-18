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
Cardinal `center-nearest` also follows the tiling structure while in this mode.

## Next work: finish P0/P1 before expanding interaction

1. Audit inventory fields against custom decoders and public option names,
   nested values, defaults and ID-targeted action availability. Do not infer
   exact compatibility from the generated schema index.
2. Add a hermetic reference operation-trace corpus for map/focus/close/group/
   split/reorder/resize. Include geometry, focus and view outcomes, not only
   driftwm structural invariants.
3. Finish P1: stable layout-container identity, clean tile/column sizing inputs,
   and extend the centralized transition policy as floating and retained
   fullscreen become supported. The shared gate exists; broad mode-exit
   fallbacks still exist.
4. Complete P2 generation/serial and bounded configure-lifecycle handling before
   layout-native interactive resizing. Then P3 settings and P5 gesture physics.
5. Preserve current command semantics while adding distinct niri-compatible
   consume/expel variants. First/last/index, swap, animation and fullscreen/tab
   contexts of structural operations remain outstanding.

Important differences to preserve explicitly while adding reference actions:

- Our merge action moves the focused window into its neighbor. Niri's consume
  action instead pulls a window into the current column.
- Our expel action splits the focused window. The pinned niri operation and its
  directional consume-or-expel variants have different selection semantics.
- Current presets are hardcoded 1/3, 1/2, full width. Reference defaults and a
  separate full-width toggle still need implementation.
- Current height allocation equalizes content heights; reference automatic
  weights and frame-space allocation are not implemented.

Do not break documented existing actions merely to reuse reference names.

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
- `PROPTEST_CASES=4000 cargo test --lib --quiet` passed all 350 library tests.
- The headless tiling/session coverage includes 18 scenarios, including direct
  operation restoration, reorder and
  preservation of camera/pointer coordinates and camera targets.
- The reference inventory regenerates byte-for-byte from the pinned checkout.
- The user tried the initial single-window-column version nested and reported
  that it worked. Multi-window/reorder behavior has headless coverage; real
  trackpad feel and current graphical behavior still need manual validation.

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
