# Tiling development status and handoff

Updated 2026-10-22. Integration checkpoint: `8c17428`; upstream `7ce05ed` is
included. Named controlled B1–B4 stabilization gates are delivered; this does
not establish complete P2/P13 acceptance or niri parity.
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
| Decoded layout options, defaults and merge semantics | [Configuration audit](niri-layout-config-audit.md) |
| Color/gradient, border/shadow, tab/insert-hint decoding and merges | [Appearance audit](niri-appearance-config-audit.md) |
| Override precedence, rule matching, initial map and reload | [Lifecycle audit](niri-config-lifecycle-audit.md) |
| Radius, workspace shadow, blur and background-effect policy | [Effects audit](niri-effects-config-audit.md) |
| Entire source-file/root-node coverage and exclusions | [Schema coverage](niri-config-schema-coverage.md) |
| Layer matching/reload/render and overview consumers | [Layer/overview audit](niri-layer-overview-audit.md) |
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

Latest A0 documentation/tooling validation: the
[recent-windows audit](niri-recent-windows-audit.md) adds all 29 named MRU fields
(525/576 schema review links). It traces defaults/merge, first-custom-bind clearing
across includes, keyboard-only modifier bindings, omitted versus skipped All
filter, model selection/cache/scope, debounce/open delay, preview minimum floor
and reload/open-state behavior. One help-title row is Outside; MRU facility rows
are Missing rather than an implementation claim. All 186 action links remain;
source exposure is 136 derived /50 skipped. Syntax, deterministic generation,
exact MRU field coverage, old keys/actions/statuses and local/pinned links were
checked. No runtime/config change or executable parity claim; implementation CI
remains B4.

Latest validation gate: `/tmp/driftwm-b4-output-ci.log`, exit 0: full
`dev/scripts/ci-local`, 369 library cases, 1490 binary cases including ignored
GPU scenarios, integration suites and isolated soak. Dedicated B4 output gate:
`/tmp/driftwm-b4-output.log`, 12 SSD/CSD scenes at 1x/1.25x/1.5x, held/mixed
resize with a ready peer, stalled peer and popup, plus a hermetic regression.
Sole-output removal now explicitly leaves tiling before teardown: queued buffers
are released, retained images/animations and output GPU state drain, and clients
leave the removed output. A disconnected placeholder cannot start tiling.
Re-adding an output with the same name retires the placeholder; manual tiling
re-entry uses a fresh layout ID, preserves window IDs, restores focus/membership
and shows the latest body/popup pixels without a repeated animation. This uses
the shared backend policy and simulated globals, not actual DRM unplug/replug.
Dedicated B4 supersession gate: `/tmp/driftwm-b4-supersede.log`, 12 SSD/CSD scenes at 1x/1.25x/1.5x,
ordinary held or recaptured mixed resize, two independent clients and a popup.
A new width request releases the old queued buffer while retaining the pending
peer. The held frame remains exact; a newly ready buffer still waits for the
other peer. Release continuity stays within two bytes and settled frames contain
only the latest body colors. Stable IDs, focus/output membership, zero retained
snapshots/pending configures/crossfades and no repeated animation are checked.
The existing hermetic queued-buffer supersession scenario also passed. No
production fix was needed for these controlled supersession cases.
Dedicated B4 protocol gate:
`/tmp/driftwm-b4-protocol.log`, 12 SSD/CSD scenes at 1x/1.25x/1.5x with held
or recaptured mixed content and a popup, plus a hermetic ready-peer scenario.
Null-buffer unmap removes layout membership and retained animation state without
ending the tiling session or replacing the stage ID. A new empty commit starts
a fresh initial configure; only the first buffer enrolls the window again.
The hidden frame equals an independent empty render; remap shows only the new
blue body, restores SSD/focus/output membership and does not animate twice.
Stage untile/re-enrollment and 4000-case stage properties also passed
(`/tmp/driftwm-b4-protocol-stage.log`). This preserves driftwm's existing live
stage record for a hidden surface; it is not complete niri lifecycle parity.
Dedicated B4 peer gate:
`/tmp/driftwm-b4-peer.log`, 12 SSD/CSD scenes at 1x/1.25x/1.5x: an ordinary
held group or a recaptured mixed resize waits for a stalled client while the
survivor has a ready buffer. Disconnect releases that buffer; any full-height
reflow is answered, retained images/crossfades/transactions settle, and a later
repaint does not restart animation. Exact held-frame comparison, final green
body/blue popup and absent magenta departed content, stable survivor ID, focus
and actual client-side output membership are checked. This is controlled
two-client disconnect coverage, not all lifecycle combinations.
Dedicated B4 remap gate:
`/tmp/driftwm-b4-remap.log`, 12 SSD/CSD scenes at 1x/1.25x/1.5x with ordinary
held images or a recaptured mixed resize. Explicit remap cancels the group,
images and crossfades; late tiling buffers and the canvas restore response must
preserve the new placement and stable window ID. A deferred exit recenter used
to override that placement after a committed tiling resize; remap now discards
it. A hermetic regression covers the restore response without GLES
(`/tmp/driftwm-b4-remap-unit.log`). This is direct compositor remap, not protocol
unmap/remap or a complete lifecycle gate.
Dedicated allocation gate:
`/tmp/driftwm-b3-failures.log`, 90 scenes passed across five capture allocation
sites, three scales, SSD/CSD and reply/timeout/exit. Parent/popup failure installs
no partial image tree; resize-bake failure retains the held image but releases
through live content. A later successful capture restores content retention,
with entry and recovered release within two bytes. Content-only bakes no longer
pad to the chrome alignment grid, fixing a fractional-scale edge stretch found
by this recovery check. Failures are scoped one-shot test injections, not actual
GPU OOM, bind/import/draw failures or every resource-failure combination.
Dedicated stale-effect gates:
`/tmp/driftwm-b3-minify.log` (24 grow/minify scenes),
`/tmp/driftwm-b3-degraded-blur.log` (24 client-region scenes), both passed. Dedicated mode gate
`/tmp/driftwm-b3-mode.log` passed 48 width-growth/shrink scenes. Transform gates:
`/tmp/driftwm-b3-transform.log` (84 tiling scenes),
`/tmp/driftwm-b3-orientation.log` (seven-transform pixel oracle), both passed. Dedicated view gate
`/tmp/driftwm-b3-views.log` passed all three tests / 36 scenes, including 24 new
fractional-camera/mode-height cases. Dedicated title gate
`/tmp/driftwm-b3-ssd.log` passed all 12 CSD/SSD scale-change scenes with opacity
0.6, title-alpha checks and repeated mixed recapture/release. Dedicated degraded gate
`/tmp/driftwm-b3-degraded.log` passed 12 SSD/CSD capped live-fallback scenes. Dedicated fractional-client
gate `/tmp/driftwm-b2-fractional.log` passed 12 scenes with patterned/transparent
content, changed offsets and both recapture response size directions.
Earlier B2 dedicated gate:
`/tmp/driftwm-b2-odd.log`, 12 SSD/CSD grow/shrink scenes passed with odd geometry
and offset (7,9). Whole-frame capture/release/recapture tolerance remains two
bytes; independent popup bounds and parent displacement checks passed.
Earlier global-reload gate: `/tmp/driftwm-b1-reload.log`, 24 scenes passed.
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
was performed for these B1–B4 batches. Nested winit NVIDIA blur artifacts/lag remain
unresolved; use direct TTY for visual acceptance.

## Next work

Next work: **A0/P0 remaining misc/root integration decoders**, pinned niri
`12de51d6a60baad8896366920278a24f17921f52`: cursor, prefer-no-csd, xwayland-satellite,
root assembly/include/reload and explicit remaining exclusions. MRU named source
review is delivered; model/input/render lifecycle acceptance remains open.
All 186 action variants have source-scope links and 525/576 named fields have
scoped review links. Independent executable parser/geometry/operation fixtures
beyond zero gaps and retained input/backend/capture/numerical interaction gates
remain required; source review is not feature completion or a gate waiver.
Do not repeat delivered audits or restart B matrices. A0/P0 gates feature
expansion; source coverage and scoped review do not prove complete niri parity.
Controlled SSD scale-change release is
fixed: retain the native title raster separately, blend old/current premultiplied
pixels before window opacity, and carry mixed title pixels through recapture.
At the current output scale, title/border/shadow must share the same rounded
band height; body mask title coverage remains a separate quantity.

Delivered B3 subsets: 12 stale/capped live-fallback scenes and 12 CSD/SSD output
scale + camera scenes (ordinary/mixed snapshot, opacity 0.6, repeated recapture/
release, independent native popup bounds, translated pixels and title-alpha
sample). Scales cover 1→1.25, 1.25→1.5, 1.5→1. Another 12 scenes cover camera
(16.4,12.2), including fractional physical displacement, and 12 scenes cover mode
height growth 1200x800→1200x1000. They retain popup anchors, release/recapture
continuity and independent configure-height checks. Mode application is simulated
at the backend-success boundary; real DRM modesetting is not exercised. These
are controlled integration gates, not all output transitions. Another 84 scenes
cover all seven non-normal transforms, SSD/CSD, 1x/1.25x/1.5x and ordinary/mixed
snapshots, including release/recapture, native popup anchors/title alpha and
configure-height reflow. Another 48 mode scenes cover width growth, width shrink,
height shrink and combined shrink (1600x800/900x800/1200x600/900x600), with exact
proportional-width/native-height offers and complete release/recapture checks.
A separate seven-transform scene verifies actual GPU orientation against an
independent upright render. Real DRM scanout is not tested. Zoom, every degraded/effect/minification path,
other GPU failure APIs and multi-output ownership remain unproven. The controlled
B4 lifecycle gates are delivered; broader combinations remain unproven. Full
tiling multi-output support remains P11; enrolling a second output still leaves tiling.
Degraded coverage also adds 12 stale-minification scenes and 24 client-region blur
scenes (grow/shrink, SSD/CSD, 1x/1.25x/1.5x, reply/timeout). Rendered blur differences
stay within the independently transformed client region, region-only restoration
is exact, and settled disable/empty regions agree. These do not cover every
constraint, anisotropic ratio, effect policy or degraded lifecycle combination.

Controlled B2 gates now include 12 solid 1x-client scenes and 12 patterned
fractional-client scenes: odd extents, SSD/CSD, canvas grow/shrink, changed
geometry offsets, capture/release/recapture/release, native popup size/anchor and
independent settled transparency/opacity samples. Scales stay fixed per scene;
this does not establish behavior during scale/output transitions or every
client texture/extent combination. Controlled B1 blur/reload/cancellation gates
remain in CI. Mapped rule flags stay sticky until remapping; the existing effect
policy still differs from pinned niri's tri-state rules. Do not restart the
already delivered controlled B1/B2 matrices.

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
