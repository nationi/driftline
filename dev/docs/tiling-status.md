# Tiling development status and handoff

Updated 2026-10-20. Tiling checkpoint: `e6eda19`; upstream `7ce05ed` was
integrated at `327fcf5` without restoring removed packaging. The persistent
development branch is `nationi/development`. This handoff was reconciled with Git;
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
  controlled tests. Late responses can animate once without reopening the group.
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

Latest integration gate: `/tmp/driftwm-upstream-sync-ci.log`, exit 0:
`dev/scripts/ci-local`, 368 library cases, 1467 binary cases including ignored
GLES scenarios, integration suites and isolated soak. Upstream IPC cursor/PID,
viewport sloppy-focus and subsurface constraint fixes are included; client-blur
and subsurface additions coexist in the test client. Earlier mask-specific
validation is recorded in history. This integration does not close a new tiling
parity milestone.
Logs in `/tmp` are disposable; persistent test names and details are in history.

Current client-mask evidence covers 18 fixed/proportional-region scenes and
12 changed-mask scenes: SSD/CSD, static 1x/1.25x/1.5x, opening/motion/canvas resize,
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
was performed for `e6eda19`. Nested winit NVIDIA blur artifacts/lag remain
unresolved; use direct TTY for visual acceptance.

## Next work

Next implementation: roadmap **B1**, mixed whole-window-rule/client-region masks,
including SSD title bands. Client-only changed-shape blending is already delivered;
do not restart it. Follow B1's explicit protocol/policy, pixel and cleanup checks.

Then B2–B4 cover general framed canvas entry/odd extents/geometry offsets,
degraded captures, changing views/scales/outputs and motion-time lifecycle.
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
