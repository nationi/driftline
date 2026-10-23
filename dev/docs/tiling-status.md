# Tiling development status

Updated 2026-10-23. This is the fork's 1.0.0 baseline on
`nationi/development`; the integration pull request targets `nationi/driftwm`
`main`. Canvas remains the default. Niri is a design reference, not a parity
target.

## Current behavior

- Each connected output owns an independent scrolling strip. Windows can be
  focused, reordered, stacked, widened, transferred between outputs and
  restored to their canvas geometry.
- Canvas and tiling keyboard actions are mode-exclusive. `Mod+Ctrl+T` toggles
  modes; `Mod+R` cycles column widths; `Mod+Shift+R` toggles half/full width.
- Alt+left-drag swaps tiled windows on the same output. Fullscreen temporarily
  suspends tiling and resumes the prior layout when fullscreen ends or closes.
- Toggling back into tiling during the same compositor session restores output
  ownership, column order, vertical stacks, widths and selected tiles. Closed
  or ineligible windows are skipped; new windows get new columns. This topology
  is not persisted across compositor restarts.
- Removing one output transfers its columns to a surviving output. Removing the
  last output restores canvas mode. Canvas geometry and view are retained.

## Scope and known limits

The fork focuses on dependable scrolling columns alongside the infinite canvas.
It does not add workspaces, tabs, a general floating-window policy, or layout
restore across compositor restarts. Fullscreen uses the existing canvas
transition path, then resumes tiling. Trackpad column snapping and layout-native
drag resizing are not implemented.

The automated multi-output fixtures cover independent layouts, mapping,
cross-output transfer, disconnect/reconnect policy and tiling re-entry. A
recorded physical TTY acceptance of the full multi-monitor workflow remains a
useful final release check; automated tests do not replace that observation.

## Latest validation

`dev/scripts/ci-local` passed in full: format, locked check, all-target clippy,
test compilation, 1668 library/compositor tests including ignored GPU cases,
all integration targets and the isolated soak test. `git diff --check` also
passes. A recorded physical TTY acceptance remains useful validation for the
multi-monitor journey; `.github/workflows/ci.yml` is authoritative for CI.

## Start here

- User behavior and bindings: [tiling guide](../../docs/tiling.md)
- Scope, sequence and acceptance: [roadmap](scrolling-tiling-roadmap.md)
- Architecture and testing rules: [caveats](caveats.md),
  [testing](testing.md)
- Local hardware preview: [TTY guide](tty-preview.md)
