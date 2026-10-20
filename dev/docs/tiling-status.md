# Tiling development status and handoff

Updated 2026-10-20. Implementation includes client-region conversion (`5f87ed0`)
and the changed-mask blending batch recorded below. This is the current
continuation point. Earlier batch
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
- Normal tiled resize chases, including compositor-requested window blur,
  keep native border strokes and SSD
  title-bar height/text around the animated visual bounds. Release preserves
  the complete SSD/CSD frame, including shadow and menu, at 1x/1.25x/1.5x.
  Superseding captures retain separate native title/ring/shadow presentation.
  Resize recapture keeps the mixed texture's texel grid; repeated grow/shrink
  releases share stable rasterized visual bounds and window-blur mask extents.
  Tiled columns keep configured corners/shadows when filling the work area.
  Retained opening progress can now survive group release and successive
  resize captures: opening wraps native chrome while content blends separately.
  Scaled-opening and opacity-only scenes cover unblurred and window-rule-blurred
  SSD/CSD, including first rounded capture. Live rounded opening uses a bounded,
  commit/geometry/scale-keyed content bake so capture samples the same grid.
  Client blur regions now follow opening/motion/native resize frames, including
  grouped capture and release with fixed or proportionally resized regions.
  Protocol state remains double-buffered; region-only commits cannot mutate a
  retained mask. Independently changed client regions now blend the old/new
  source masks with the content weights; each source contributes its own alpha
  shape. Superseding groups retain the current fractional coverage, including
  capture before the first tick and during an inherited opening. Disabling
  client blur can fade out without exposing the pending mask while held.
- Optional udev `--drm-connector NAME` restricts preview output enrollment.
  It does not implement independent multi-output tiling.

P1's single-container foundation is in place. P0 semantic audit and P2/P13
acceptance are incomplete. Sizing/topology/view behavior has partial P3/P4/P5
coverage. The main P6–P12/P14–P15 features/audits remain outstanding. No weighted
completion metric exists; historical percentage estimates are not a current
measurement. See roadmap acceptance criteria instead.

## Verified evidence and its limits

Latest implementation gate: `/tmp/driftwm-mask-blending-ci.log`, exit 0:
full `dev/scripts/ci-local`, 368 library cases, 1457 binary cases including ignored
GLES scenarios, integration suites and isolated soak. Final fmt and strict
all-target clippy passed (`/tmp/driftwm-mask-final-clippy.log`); the final client
blur regressions passed in `/tmp/driftwm-mask-final.log`. Dedicated changed-mask GLES gate:
`/tmp/driftwm-mask-changes.log`, 12 scenes (SSD/CSD, opening/seeded canvas resize,
static 1x/1.25x/1.5x). It checks changed shape with a doubled buffer, first release,
immediate recapture before a tick, intermediate fractional old/new GPU-mask
weights, independent transparent pixels in the new source, recapture of the
mixed mask, region-only disable while held, release and complete fade-out.
Complete capture/release comparisons retain the two-byte tolerance; held frames
remain byte-identical. Earlier 18 fixed/proportional-region scenes remain in CI.
This closes these controlled client-only mask transitions, not every blur policy
combination or universal frame atomicity. Whole-window-rule/client-mask switches
with SSD title bands, odd extents/geometry offsets, changing views/scales and
lifecycle transitions still require dedicated gates.
No release build, installation or manual TTY acceptance was performed in this batch.

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
2. Continue P2/P13 with mixed whole-window-rule/client-region policies (including
   SSD title bands) and general framed canvas resize entry. Changed client-only
   mask shapes, disable, immediate recapture and inherited opening now have
   controlled gates. The first rounded opening capture and
   combined window blur/opening resize gates are delivered for the scenes above.
   Normal unblurred and window-rule-blurred SSD/CSD recapture/release is covered
   for three consecutive preset changes at static 1x/1.25x/1.5x. Client-effect
   framed canvas resize entry is covered in the controlled scenes above; general
   framed canvas resize entry, odd extents, geometry offsets, other animation
   phases and changing views/scales still need their own gates.
3. Cover degraded stale/capped capture (body/frame geometry differ), changing
   view/output scale and output transitions, then motion-time remap/disconnect
   and opening through these lifecycle changes. Existing static-scale
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
No installation, desktop restart or push was performed in this implementation batch.

The previously separate output-management changes were reviewed and authorized
for their own commit/push on 2026-10-19: preserving monitor/mode resources on
refresh changes, suppressing identical notifications, and protecting a reconnected
head from a late release of its predecessor. They remain separate from tiling
ownership work and do not implement P11. Check the current tree before edits;
never assume historical uncommitted-file lists are still current.

Current `origin` is `https://github.com/nationi/driftwm.git`; `upstream` is
`https://github.com/malbiruk/driftwm.git`. The user authorized pushing the current
work to `origin/nationi/development`; recheck destination before later pushes.
Local implementation authorization does not authorize publishing or messaging.

Read AGENTS.md, CONTRIBUTING.md, caveats and testing rules before code changes.
Run meaningful targeted checks and the required CI gate for implementation;
documentation-only edits need link/content checks, not a repeated runtime gate.
Keep this status concise and current; append detailed batch evidence to history.
