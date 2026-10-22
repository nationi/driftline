# Pinned niri layer rules and overview consumers

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source audit
covers 13 LayerRule/Match fields and six Overview/OverviewPart fields from the
expanded inventory. It traces defaults, matching, map/commit/reload and selected
render/focus/interaction consumers. It is not execution of niri, a pixel oracle
or complete layer/capture/overview interaction acceptance. No driftwm behavior
or config changes are made.

## Pinned sources

- [layer_rule.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/layer_rule.rs): public LayerRule/Match grammar; [IPC Layer parser](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs): accepted layer names.
- [layer/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layer/mod.rs): ResolvedLayerRules::compute and surface_matches.
- [layer/mapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layer/mapped.rs): shadow setup/update, layer placement predicate, body/popup rendering, opacity and bob motion.
- [handlers/layer_shell.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/layer_shell.rs): initial mapping, pending/current layer comparison hook, commit recomputation and null-buffer unmap.
- [misc.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/misc.rs): Overview/OverviewPart defaults and merge.
- [layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs): compute_overview_zoom, overview flag/progress, open/close/toggle, gesture update/end and option propagation.
- [monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs): overview_zoom, scaled workspace rendering and active-window visual rectangle.
- [workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs): workspace shadow/size updates; [input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs): overview-specific workspace hit/interaction branches.
- [niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs): config/rule/startup refresh, backdrop buffers, layer render partition and keyboard focus.
- [render targets](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/render_helpers/mod.rs): RenderTarget::should_block_out; [utils](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/mod.rs): baba_is_float_offset.

Shared Color/ShadowRule/CornerRadius/BackgroundEffect values retain the
[appearance](niri-appearance-config-audit.md) and [effects](niri-effects-config-audit.md)
audits. Layer contexts are traced here rather than inferred from WindowRule.

## Layer matching, defaults and local merge

Root `layer-rule` contains repeated `match`/`exclude` children. An empty match
list matches all surfaces; otherwise any matching block suffices. All properties
within one block must agree. Any matching exclude rejects the rule. Applicable
rules fold in source order from a fresh ResolvedLayerRules default: present scalar
values override, absence preserves, nested shadow/effect/popup values compose.
Explicit false can override earlier true.

Match has only three optional properties:

| Property | Decoder / predicate |
| --- | --- |
| `namespace` | RegexEq string; regex match against layer namespace, without implicit whole-string anchoring. |
| `at-startup` | Boolean compared to compositor startup state. |
| `layer` | String parsed as exactly background, bottom, top or overlay; compares the current protocol layer. |

It has no app-ID, output-name, focus or floating match property. Layer matching
uses current committed state; the update hook deliberately waits for the commit.

| LayerRule child | Shape, default and consumer |
| --- | --- |
| `opacity` | Optional f32 argument; no explicit 0–1 bound in this decoder. Default absent becomes 1; rendering clamps to [0,1]. |
| `block-out-from` | Optional scalar screencast or screen-capture. Screencast hides only on RenderTarget::Screencast; screen-capture hides on all non-output targets. Default absent does not block. |
| `shadow` | ShadowRule, default empty. MappedLayer starts with **global config.layout.shadow parameters**, forcibly disables its on flag, then merges composed layer rules. A global enabled window shadow does not automatically enable layer shadows. |
| `geometry-corner-radius` | Optional CornerRadius. Used by shadow/background effect geometry. The inspected body render passes the ordinary surface tree; this setting alone is not a forced corner clip of the layer's content. Block-out solid rendering explicitly has a radius TODO. |
| `place-within-backdrop` | Optional bool, resolved false. True is effective only when the layer is Background **and** its exclusive zone is DontCare. This is not permission to move any Top/Overlay panel into the backdrop. |
| `baba-is-float` | Optional bool, resolved false. Enables continuous bob motion, not tiling floating mode. Offset Y is view_height/96 × (sin(2π×time/3.6) − 1), rounded in physical space. |
| `background-effect` | BackgroundEffectRule, empty/default optional values. Consumes global blur tuning through the shared effect helper, with previously audited veto/region/xray rules. |
| `popups` | PopupsRule (opacity/radius/background effect), not all parent settings copied into children. Regular popup alpha multiplies clamped parent and popup opacities; IME popups use default popup rules. Unspecified popup xray is explicitly false. |

Resolved defaults otherwise have absent opacity/block/radius, empty shadow/
effect/popup rules and false placement/bob flags. Blocked-out layer body becomes
a solid black rectangle at the layer's opacity; its popups are omitted for that
render target. This traces the selected target predicate/body branch, not every
screencopy, cast or capture pipeline.

## Layer lifecycle, redraw and focus

The layer commit handler arranges layers before initial configure. It creates
MappedLayer and resolves rules when the first mapped buffer appears. A pre-commit
hook compares pending/current protocol layer and marks rule recomputation for
the subsequent commit; the commit recomputes rules and updates configuration if
they changed. Null-buffer unmap removes mapped state and requires a fresh initial
commit/configure sequence.

Config application updates existing mapped-layer global shadow parameters and
blur tuning. A changed layer-rule list triggers recompute_layer_rules from fresh
defaults, updates changed mapped state and queues redraw. Startup completion
also recomputes layer rules. Layer changes do not rely on an app-ID/title
callback, since those are not LayerRule match inputs.

layers_in_render_order partitions surfaces by the computed place_within_backdrop
predicate. Keyboard-focus cleanup rejects an on-demand layer that moved into the
backdrop. The predicate itself does not depend on the overview-open flag. Newly
mapped OnDemand surfaces otherwise get the tracked on-demand focus candidate;
exclusive focus ordering remains in keyboard-focus policy. Full focus/input
integration still requires independent fixtures.

## Overview defaults, runtime zoom and state

Public `overview` children:

| Child | Decode / resolved default |
| --- | --- |
| `zoom` | FloatOrInt [0,1]; default 0.5. |
| `backdrop-color` | Color; default unpremultiplied RGBA (0.15,0.15,0.15,1). |
| `workspace-shadow` | WorkspaceShadowPart; enabled default and logical view-height normalization from the effects audit. |

Parts preserve omitted zoom/color values and merge the nested shadow. Parse
acceptance is not the runtime zoom range: compute_overview_zoom clamps configured
zoom to **[0.0001,0.75]**. No progress state means 1. Otherwise the result is
max(0.0001, 1 − progress × (1 − clamped_zoom)). At configured 0.5, progress 0,
0.5,1 gives 1,0.75,0.5. At configured 1, fully open zoom is 0.75; at configured
0, it is 0.0001. Gesture rubber-band overshoot and animation are separate from
parse-time validation.

Overview has a desired open flag **and** optional progress state (animation,
gesture or fully open). Toggle reverses the desired state from the current
progress using overview_open_close animation. Open/close are idempotent checks
against the desired flag. During closing the flag can be false while progress
still scales rendering; on completion closed progress is removed, while opening
becomes the Open state. The state is propagated to monitors.

Gesture begin takes the current progress, update uses tracked displacement plus
rubber-banding, end projects velocity and snaps the desired target to 0 or 1
before animating. This audits the state/consumer path, not a complete gesture
threshold/physics or action-binding parity specification.

## Overview rendering, reload and interactions

Monitor derives overview_zoom from current options/progress, uses it for
workspace size/render geometry and rescales/relocates workspace elements.
active_window_visual_rectangle returns None when overview's desired open flag
is true. Pointer handling explicitly uses workspace_under_cursor in overview;
for example a right-button workspace interaction has a distinct grab branch,
and selecting an overview workspace can focus/activate it. These are separate
workspace interactions, not driftwm's generic canvas pan/zoom. This audit does
not close every click/drag/DnD/keyboard path or workspace-transfer milestone.

Config updates propagate Overview options through layout/monitor/workspace
updates; the inspected update_options path does not toggle or reset the overview
state just to apply a new zoom value. Workspace shadow is recalculated on config
and view-size updates. Changed global backdrop color requests output-config
reload. Per-output backdrop-color overrides the global overview backdrop; the
output backdrop buffer forces alpha to **1** before rendering. This is distinct
from layout.background-color / workspace background.

Driftwm [layer rendering](../../src/render/layers.rs) resolves generic window
rules against namespace as app-ID and empty title at render time. It has opt-in
layer shadow/border/scalar corner clipping and opacity, but no separate niri
match/exclude/startup/layer rule grammar. These live layer lookups are distinct
from sticky mapped-toplevel rules. Its [layer handler](../../src/handlers/layer_shell.rs)
also supports canvas-positioned layer instances through window rules. Neither
canvas layers nor global camera zoom implement niri overview/workspace backdrop,
MRU, zoom interpolation or workspace interaction semantics. Preserve canvas
behavior; all six overview fields remain Missing as a reference facility.

## Source-derived probes and remaining work

These probes require later independent fixtures; none is a claim of niri execution:

| Trigger | Required distinction |
| --- | --- |
| Global window shadow on; no layer shadow-on rule | Layer shadow remains off, while shared global shadow parameters are still available. |
| Two matching layer rules, later opacity absent | Earlier opacity persists; recompute from defaults removes stale values when matching changes. |
| Commit layer change Background → Top | Rule selection observes the committed layer; pending changes do not apply prematurely. |
| place-within-backdrop=true on Top or exclusive-zone != DontCare | Predicate false; no unconditional backdrop relocation. |
| Qualifying backdrop placement on an on-demand layer | Focus cleanup rejects it as on-demand focus. |
| opacity 2 or −1 | Decoder f32 is not the rendering clamp; render uses 1 or 0. |
| screen-capture versus screencast block-out | First includes both capture targets, second only cast; output remains visible. |
| Configured overview zoom 1 or 0 | Decode succeeds within bounds; fully open runtime zoom becomes 0.75 or 0.0001. |
| Close overview mid-animation | Desired flag changes immediately; rendering may remain scaled until progress settles. |
| Global translucent backdrop color, with/without output override | Select output override first; backdrop buffer alpha is forced opaque. |

Next A0: input focus/warping/workspace policy and debug transaction/resize/focus
flags, then remaining animation/gesture/naming/rule decoders and public binding /
ID/index/previous/next actions. Independent decoder/operation/geometry fixtures
beyond zero gaps remain required. These scoped consumers do not implement the
missing facilities or complete P0, P11/P12/P14, capture or visual acceptance.
