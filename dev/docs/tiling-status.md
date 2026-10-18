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

Implementation locations:

- `src/layout/scrolling/` — pure single/grouped geometry and reveal calculations.
- `src/stage/tiling.rs` — membership, ordering, selection and original canvas rects.
- `src/state/tiling.rs` — mode lifecycle, reflow, configures and view integration.
- `src/config/{types,parse}.rs`, `src/input/actions.rs` — public binding actions.
- `src/tests/tiling.rs`, session-restore tests — real headless Wayland scenarios.
- `config.reference.toml`, generated `docs/config.md`, `docs/tiling.md` — user interface.

The current implementation supports one output. It exits tiling for many
incompatible canvas actions and when another output appears. Tabs, snapping
physics, layout-native drag/resize, stationary floating and retained tiled
fullscreen are missing. These exits are temporary fallbacks, not parity.

Current action names: `toggle-tiling`, `focus-column left/right/up/down`,
`cycle-column-width`, `move-into-column left/right`, `expel-from-column`.
Cardinal `center-nearest` also follows the tiling structure while in this mode.

## Next work: P0 and the first P1 slice

No parity inventory has been implemented yet. Do these next:

1. Create a complete action/option inventory from pinned niri sources. For each
   entry record its reference symbol, driftwm equivalent, exact/partial/missing
   status, behavioral differences and test evidence. Include ID-targeted forms.
2. Add stable `ColumnId` identity independent of selected-window `ElementId`;
   separate column identity from per-column selection. Currently `TiledColumn.id`
   is the selected window, so it must not become a drag/animation/persistence ID.
3. Add model traces for identity and selection through merge, split, reorder,
   close/crash and replacement. Preserve saved canvas geometry through all of them.
4. Preserve the visible anchor when columns to the left change. Then implement
   column left/right reorder and window up/down reorder as focused changes.
5. Continue P2 configure-generation/serial handling, followed by sizing settings
   and the P5 gesture view model. Follow dependencies in the roadmap.

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

## Validation evidence

For the implementation at `0fc1309`:

- `dev/scripts/ci-local` passed every stage, including ignored tests and isolated soak.
- `PROPTEST_CASES=4000 cargo test --lib --quiet` passed all 348 library tests.
- The headless tiling/session filter passed 14 scenarios.
- The user tried the initial single-window-column version nested and reported
  that it worked. Multi-window-column behavior has headless coverage; real
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
The two tiling commits above excluded them; the full working-tree CI included
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
