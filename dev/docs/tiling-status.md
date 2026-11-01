# Tiling development status and next steps

Updated 2026-11-02 for the `v1.1.7` upstream integration. User behavior and
bindings are documented in the
[tiling guide](../../docs/tiling.md); persistence is described in the
[session guide](../../docs/session.md).

## Scope and completed work

The initial multi-output delivery plan is complete. Canvas remains the startup
default; each output owns an independent scrolling-column layout.

Implemented:

- Mode-exclusive keyboard controls, directional focus, Alt-Tab and centering
  the tile under the pointer with `Mod+X`.
- Column width presets and Alt+right-drag resize, client minimum widths,
  pointer confinement to the source output and continuous resize edge scrolling.
- Alt+left-drag swap, stack insertion, extraction and column placement, with
  a moving window picture, destination previews and compact action labels.
  Holding a dragged window at a horizontal edge scrolls toward neighboring columns.
- Keyboard and mouse transfer between outputs, hotplug handling and fullscreen
  isolation without leaving tiling on other outputs.
- Durable column/stack order, widths, selection and output ownership; eligible
  app relaunch and topology reconstruction after restarting the session.
- Animated layout changes and canvas/tiling transitions. The latest transition
  uses staggered motion along a shallow arc with a temporary shrink, preserves
  the zoomed canvas picture and can reverse from the visible geometry.
- Repository cleanup and a refreshed README covering the fork, current controls,
  build/install steps and compatibility with existing configurations.

Niri is a design reference, not a parity target. Workspaces, tabs and a general
floating-window policy are outside the current scope. No further feature is
required merely to finish the old checklist.

## Upstream integration

Merged 104 upstream commits through `e97a9d0` (v0.21.0 plus the fractional-scale
follow-up), preserving the fork's tiling implementation, Split D artwork,
Driftline package identity and legacy config/session/IPC compatibility.
Nix support, the documentation site and tracked AI assistant files stay removed.
Canvas default binding changes are listed in [the changelog](../../CHANGELOG.md).
User config files are not migrated or overwritten.

The scaled-output transfer fixture now checks preferred scale in both directions
(1.0 → 1.5 → 1.0), no duplicate scale events and preserved tiling membership.
All upstream functions in changed files remain available after rebranding,
except the old session round-trip fixture superseded by the fork's existing
legacy-session restore coverage. The removed `which` subprocess is replaced by
upstream's in-process PATH lookup.

## Validation

The complete `dev/scripts/ci-local` sequence passed for `1.1.7`: formatting,
locked check, all-target Clippy, test compilation, tests including ignored
rendering cases, and isolated soak. Results: 428 library tests, 1817 compositor
tests, 354 integration tests and one separate soak test passed.
`PROPTEST_CASES=4000` runs of the library and config robustness suite also passed.
The locked release build, staged install/uninstall and both reference/personal
config validation passed. The staging directory was removed.
The real-client test self-skips locally without `foot` or `weston-terminal`;
GitHub CI installs `foot` for that coverage.

The user accepted multi-monitor operation, app relaunch, topology restore and
the interaction/animation checks in TTY before this upstream integration.
That acceptance is for the previous build; no live session was restarted during
integration. Headless/GLES checks do not establish hardware acceptance of a new
build. Release publication and CI outcomes are tracked by Git tags and GitHub.

## Continuing work

There is no remaining feature checklist for the current tiling scope. Prioritize
a reproduced issue or concrete user need; do not recreate parity roadmaps or
canceled demo work. Preserve the user's running session during binary updates.

## Maintenance rules

Read the affected implementation and [architecture caveats](caveats.md) before
changing behavior. Use the [testing guide](testing.md) for focused checks and
hardware acceptance. Keep this file as the single active tiling plan; completed
roadmaps need not be recreated. Replace stale validation notes after delivery
and keep detailed history in Git.
