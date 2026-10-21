# Tiling development status and handoff

Updated 2026-10-21. Integration checkpoint: `8c17428`; upstream `7ce05ed` is
included. Controlled B1 native-title mask gates are delivered; B2 is next.
The persistent development branch is `nationi/development`. This handoff was reconciled with Git;
see the [roadmap queue](niri-tiling-parity-plan.md#execution-queue-and-acceptance-gates)
for task order and acceptance. Older “next” statements in the
[history](tiling-history.md) are historical, not the current task queue.

## Where to look

| Need | Reference |
| --- | --- |
| Available behavior and configuration | [User guide](../../docs/tiling.md) |
| Objective, dependencies and acceptance gates | [Parity roadmap](niri-tiling-parity-plan.md) |
| Pinned symbols and conservative compatibility labels | [Generated inventory](niri-tiling-inventory.md) |
| Source-derived fixture assumptions | [Reference traces](niri-reference-traces.md) |
| Local preview launch, keys and return path | [TTY guide](tty-preview.md) |
| Per-batch results and investigation logs | [History](tiling-history.md) |

## Goal and current implementation

The goal is full behavioral parity with niri tiling, retaining canvas as the
default. The reference remains `12de51d6a60baad8896366920278a24f17921f52` in
`/tmp/niri`; recover that exact revision if missing. Do not silently upgrade it.
Workspace/overview design and independent output ownership remain unresolved.

Implemented, with partial reference coverage:

- Single-output columns, stable layout/column IDs, separate selection, reorder,
  focused merge/split, and distinct reference first-consume/last-expel actions.
- Initial size offers before the first buffer; mapping-time enrollment and
  neighbor selection. No unbuffered phantom columns.
- Generation/serial tracking, bounded 300 ms commit groups, deferred placement
  and view updates, timeout/supersession/removal/exit cleanup.
- Canvas geometry/view restoration, horizontal anchoring and close-return focus.
- Retained parent images, separate popup updates, live backdrop blur, native
  SSD/CSD frames, premultiplied resize blending and inherited opening.
- Repeated capture/release retains mixed texels and fractional client-mask
  coverage. Changed client-only regions blend independent source alpha; region-only
  commits stay hidden while held. Immediate recapture and disabling blur have
  controlled tests. Mixed client/window-rule blur now retains native SSD title
  coverage separately from the body, including title-only policy changes.
  Late responses can animate once without reopening the group.
- Optional udev `--drm-connector NAME` restricts preview output enrollment.

P1's single-container foundation exists; this does not close the full milestone.
P0 semantics and P2/P13 acceptance are incomplete. P3/P4/P5 have partial coverage;
P6–P12/P14–P15 remain feature/audit milestones. There is no weighted percentage.

Known product limits: presets remain 1/3, 1/2 and full width; reference settings,
weighted frame-space heights, view snap physics, tabs, layout-native manipulation,
floating/fullscreen integration, workspaces and independent multi-output tiling
are unfinished. A second output and many conflicting canvas actions leave the
mode. These are documented fallbacks, not reference-compatible behavior.

## Verified evidence and its limits

Latest validation gate: `/tmp/driftwm-b1-reload-ci.log`, exit 0: full
`dev/scripts/ci-local`, 368 library cases, 1470 binary cases including ignored
GPU scenarios, integration suites and isolated soak. Dedicated global-reload
gate: `/tmp/driftwm-b1-reload.log`, both tests / 24 scenes passed.
Dedicated mixed-policy GPU gate: `/tmp/driftwm-b1-mixed.log`, passed 18 scenes:
SSD/CSD, static 1x/1.25x/1.5x, opening/seeded canvas resize, actual client-buffer
growth and shrink, opacity 0.6, rounded frames and transparent new-source pixels.
It checks partial → rule fallback → partial, native title weights/corner cutouts,
body-identical but title-different policies, held region-only changes and immediate/
mid-animation recapture. Earlier integration evidence remains in history.
Logs in `/tmp` are disposable; persistent test names and details are in history.

Global reload evidence adds 12 SSD/CSD scenes and 12 SSD exit/disconnect scenes
at the same static scales, with opening/seeded resize. Held images remain exact
after both enable and disable reload; release stays within two bytes. Disconnect
returns to an independently rendered empty background after closing completes.
These tests exercise real config reload with unchanged output mode/scale; output
changes and broader lifecycle permutations remain B3/B4.

Current client-mask evidence covers 18 fixed/proportional-region scenes and
12 changed-mask scenes, plus the 18 mixed-policy scenes above: SSD/CSD, static 1x/1.25x/1.5x, opening/motion/canvas resize,
opacity and rounded frames. Changed-mask tests check GPU weights, transparent
new-source pixels, immediate and mid-animation recapture, held region-only
commits, disable and settled cleanup. Full-frame transition tolerance is two
bytes; held-frame comparisons are exact. Previous window-rule/opening/frame
regressions remain in CI. These controlled scenes do not establish universal
atomicity or parity across every policy, extent, scale transition and lifecycle.

GPU evidence is from NVIDIA RTX 3060. Reference fixtures are hand-reviewed,
source-derived expectations, not recordings of niri. Older direct-udev feedback
confirmed the single-output bindings/menu scenario; it does not validate later
commits automatically. No release build, installation or manual TTY acceptance
was performed for this B1 batch. Nested winit NVIDIA blur artifacts/lag remain
unresolved; use direct TTY for visual acceptance.

## Next work

Next implementation: roadmap **B2**, general framed canvas resize entry with
odd extents and nonzero window-geometry offsets. Exercise SSD/CSD grow/shrink,
capture/hold/release/recapture, popup anchors and fractional rounding. Controlled
B1 gates now include global `[decorations].blur` hot-reload during a held group
and exit/disconnect with a partially mixed native title. Mapped window-rule flags
stay sticky; changing their TOML rules does not reapply them to existing windows. Native title/body blending for
fixed mapped rules is delivered in the controlled scenes above. Keep driftwm's
existing policy distinct from niri's tri-state effect configuration; see the
source audit in history. Do not restart the delivered client-only mask work.

Then B3–B4 cover degraded captures, changing views/scales/outputs and the broader
motion-time remap/supersession/output-removal lifecycle.
**A0/P0 semantic audit must finish before expanding features** into P3/P4/P5 or
advanced interactions. The roadmap is the single task/acceptance queue.

## Working-tree and session precautions

- Read `AGENTS.md`, contribution, caveat and testing rules before implementation.
  Stage is the sole window authority; input retains its destination contract.
- Inspect Git status/history before editing. Update this handoff after delivery;
  append detailed evidence to history, not to `AGENTS.md` or the roadmap.
- Follow the TTY guide. The launcher uses the last local `target/release/driftwm`,
  not a binary pinned by the preview directory name. Do not replace the installed
  compositor or restart the main desktop for routine tests.
- Follow [the Git workflow](git-workflow.md) for branch tracking and integration PRs.
- `origin`: `https://github.com/nationi/driftwm.git`;
  `upstream`: `https://github.com/malbiruk/driftwm.git`.
  This plan does not grant push, publish, installation or messaging permission.
- Output-management fixes already landed separately; they do not implement P11.
  Never reuse historical uncommitted-file lists as current tree state.
- Documentation-only cleanup needs content/link/diff checks. Implementation
  changes require the meaningful targeted and full CI gates in the testing guide.
