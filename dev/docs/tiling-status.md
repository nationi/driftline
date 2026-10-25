# Tiling development status

Updated 2026-10-25. User behavior and bindings are documented in the
[tiling guide](../../docs/tiling.md); persistence is described in the
[session guide](../../docs/session.md).

## Scope

The initial multi-output delivery plan is complete. Each output owns an
independent scrolling-column layout. Navigation, resizing, window transfers,
hotplug, fullscreen isolation, mode-exclusive controls and durable topology
restore are implemented. Canvas remains the startup default.

Niri is a design reference, not a parity target. Workspaces, tabs and a general
floating-window policy are outside the current scope. Prioritize concrete user
needs and verified bugs over extending a completed feature checklist.

## Latest implementation and validation

Canvas/tiling switching now uses staggered window motion along a shallow arc,
with a small temporary shrink. Entry preserves the zoomed canvas picture;
reversing the transition starts from the visible geometry. Setting
`animation_speed = 1` keeps switching immediate.

For this change, the locked check, all-target Clippy and formatting passed.
All 1613 non-ignored compositor tests passed, including 77 tiling fixtures.
Focused GPU checks covered zoomed entry, retained pixels, multi-output snapshots
and protocol unmap/remap. The complete CI sequence was not rerun as a single
final pass for this change. Consult Git history for earlier validation details.

The user previously confirmed multi-monitor operation, app relaunch and topology
restore across a real session restart. Live review of the latest mode animation
and mouse transfer/drop feedback remains; do not infer hardware acceptance from
headless or surfaceless rendering tests.

## Continuing work

- Read the affected implementation and [architecture caveats](caveats.md).
- Use the [testing guide](testing.md) for focused checks and hardware acceptance.
- Update user-facing documentation when behavior changes, and replace stale
  validation notes here after delivery. Keep detailed session history in Git.
