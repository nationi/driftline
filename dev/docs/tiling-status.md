# Tiling development status and handoff

Updated 2026-10-19. Implementation baseline: `059556e`; subsequent commits
update documentation. This is the current continuation point. Earlier batch
notes, failed experiments and source derivations are preserved in
[the historical journal](tiling-history.md); their pending/next statements are
historical, not instructions to repeat completed work.

## Where to look

| Need | Reference |
| --- | --- |
| Run the local preview and recall its keys | [TTY cheat sheet](tty-preview.md) |
| Available behavior and binding examples | [User guide](../../docs/tiling.md) |
| Full objective, dependencies and acceptance criteria | [Parity roadmap](niri-tiling-parity-plan.md) |
| Reference symbols and conservative compatibility labels | [Generated inventory](niri-tiling-inventory.md) |
| Source-derived fixture assumptions and limitations | [Reference traces](niri-reference-traces.md) |
| Older implementation evidence and investigation logs | [History](tiling-history.md) |

## Goal and current implementation

The goal remains full niri tiling behavior while preserving driftwm's default
canvas. Reference: `12de51d6a60baad8896366920278a24f17921f52` in `/tmp/niri`.
Recover that exact revision if the temporary checkout disappears. Do not update
the baseline implicitly. Workspaces/overview and independent output ownership
still need architectural decisions at their milestones.

Implemented, with partial reference coverage:

- Single-output columns with multiple windows, stable column/layout IDs,
  separate selection, structural reorder, merge/split and reference
  first-consume/last-expel topology actions.
- Canvas geometry/view restoration, close-return focus and horizontal anchoring;
  a central policy gate restores canvas for unsupported operations.
- Ordinary initial size offers before the first buffer. Enrollment and neighbor
  choice happen at mapping time; there are no unbuffered phantom columns.
- Generation/serial tracking, bounded 300 ms commit groups and deferred placement
  and view updates. Timeout, supersession, removal and exit release waiters.
- Retained parent images, separate popup trees, live backdrop blur, opening and
  translation transforms, release geometry animation and premultiplied resize
  content blending. Resolved resize and normal start holds can transfer into
  a newer group. Changed popups update live without exposing the held parent.
- Popup dimensions stay native during geometry resize, following parent motion
  and opening scale. A settled timed-out window can animate its late response
  once without another configure or reopening the group.
- Optional udev `--drm-connector NAME` restricts preview output enrollment.
  It does not implement independent multi-output tiling.

P1's single-container foundation is in place. P0 semantic audit and P2/P13
acceptance are incomplete. Sizing/topology/view behavior has partial P3/P4/P5
coverage. The main P6–P12/P14–P15 features/audits remain outstanding. No weighted
completion metric exists; historical percentage estimates are not a current
measurement. See roadmap acceptance criteria instead.

## Verified evidence and its limits

Latest implementation gate: `/tmp/driftwm-menus-delayed-ci.log`, exit 0:
fmt, locked check, strict all-target clippy, 368 library tests, 1451 binary
regular/ignored tests plus isolated soak (1452 total), and integration suites.
Release build succeeded: `/tmp/driftwm-menus-delayed-build.log`.
These are recorded checks for the implementation baseline, not newly rerun by
this documentation update. Full working-tree checks included the unrelated
output-management changes listed below.

GPU regressions ran on NVIDIA RTX 3060, including controlled 1x/1.25x/1.5x
retained-content, alpha, popup and delayed-response scenes. Static scale cases
do not validate scale transitions or universal frame atomicity. Reference JSON
fixtures are hand-reviewed source-derived expectations, not recordings of niri.

The user confirmed direct-udev TTY runs well, the single-output bindings work,
and the latest menu test stays in place without lag. This is manual acceptance
of those scenarios, not synthetic delayed clients or every decoration/scale case.
Nested winit NVIDIA blur still has artifacts/lag; its precise presentation defect
is unresolved. Use direct TTY for visual acceptance. Do not report the nested
issue fixed or disable blur to claim success.

## Next work and remaining gates

1. Finish P0 custom-decoder/public-option/default/ID-action audit and extend
   source-derived fixtures beyond the controlled zero-gap geometry subset.
   This remains a prerequisite to expanding advanced interactions.
2. Continue the existing P2/P13 acceptance work with a focused full-frame
   regression for SSD, borders and shadows at resize-group release. Preserve
   body and popup behavior and existing pixel tolerances.
3. Cover degraded stale/capped capture (body/frame geometry differ), changing
   view/output scale and output transitions, then motion-time remap/disconnect
   and inherited opening during resize supersession. Existing static-scale
   and timeout tests do not close these cases.
4. After these gates, follow P3 sizing/settings and P5 view physics dependencies;
   extend structural reference operations with their own fixtures. Tabs,
   layout-native drag/resize, floating, retained fullscreen, workspaces and
   multi-output ownership are still future milestones.

Fixed presets remain 1/3, 1/2 and full width; reference presets/settings and
weighted frame-space heights are missing. Many canvas actions and a second
output leave tiling. These are safety fallbacks, not niri-compatible behavior.
Do not reinterpret existing focused move/split commands as the distinct
reference consume/expel commands. Input keeps its existing destination contract
through visual animation; Stage remains the sole window authority.

## Local preview and working-tree precautions

Use [the TTY guide](tty-preview.md). The external launcher selects the current
`target/release/driftwm`, not the commit in its directory name. Main desktop is
on TTY2 in this setup; verify before relying on that in a later session. Do not
replace the installed binary or restart the main desktop for ordinary testing.
No installation or push was performed by this documentation update.

Pre-existing unrelated changes currently remain in:

- `src/protocols/output_management.rs`
- `src/tests/output_management.rs`
- `src/tests/mod.rs` (`mod output_management;`)

Recheck the tree and preserve these files; stage only owned paths. Current
`origin` is `https://github.com/nationi/driftwm.git`; `upstream` is
`https://github.com/malbiruk/driftwm.git`. Recheck before any authorized push.
Local implementation authorization does not authorize publishing or messaging.

Read AGENTS.md, CONTRIBUTING.md, caveats and testing rules before code changes.
Run meaningful targeted checks and the required CI gate for implementation;
documentation-only edits need link/content checks, not a repeated runtime gate.
Keep this status concise and current; append detailed batch evidence to history.
