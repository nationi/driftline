# Tiling development status and next steps

Updated 2026-11-02 for the `v1.1.6` release preparation. User behavior and bindings are documented in the
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

## Validation already recorded

The user confirmed multi-monitor operation, app relaunch and topology restore
across a real session restart. The user also accepted drag edge scrolling before
the subsequent visual restyling. On 2026-11-02, the user confirmed completing
the remaining TTY hardware acceptance checks for the latest interactions.
No further manual acceptance is pending for this delivery.

For the `1.1.6` release preparation, the complete `dev/scripts/ci-local`
sequence passed: formatting, locked check, all-target Clippy, test compilation,
all tests including ignored rendering cases, and the isolated soak stage.
The run reported 1695 compositor tests plus one isolated soak test, 415 library
tests and 354 integration tests passing. The real-client test can self-skip
when neither `foot` nor `weston-terminal` is available; neither is installed on
the local validation machine. CI installs `foot` for that coverage.

The locked release build and a staged
`make install` also passed; the installed test binary reports `driftline 1.1.6`
and accepts `config.reference.toml`. The staging directory was removed.

The release updates the version and changelog, refreshes installation guidance,
adds the user-selected Split D logo and session icon, and removes local AI
assistant instructions from version control. License and upstream attribution are preserved. Release publication is tracked by Git tags
and GitHub releases rather than a pending checklist here.

## Continuing work

There are no outstanding implementation or hardware-acceptance tasks in the
current tiling plan. Choose the next change from a reproduced issue or a concrete
user need. Do not recreate completed parity roadmaps or canceled demo work.

## Maintenance rules

Read the affected implementation and [architecture caveats](caveats.md) before
changing behavior. Use the [testing guide](testing.md) for focused checks and
hardware acceptance. Keep this file as the single active tiling plan; completed
roadmaps need not be recreated. Replace stale validation notes after delivery
and keep detailed history in Git.
