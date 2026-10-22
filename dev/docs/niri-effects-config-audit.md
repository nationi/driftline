# Pinned niri radius, workspace-shadow and background-effect audit

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 batch covers
34 fields in seven remaining appearance structs (CornerRadius, WorkspaceShadow,
WorkspaceShadowPart, Blur, BlurPart, BackgroundEffectRule, BackgroundEffect) and
seven scoped window/popup consumer fields. Their public grammar, defaults,
local merges and effect-selection paths are source-reviewed, not executed.
No driftwm runtime/config policy changes or Exact compatibility claims are made.

See the [core appearance audit](niri-appearance-config-audit.md) for Color,
ShadowOffset and shared numeric/flag rules, and [lifecycle audit](niri-config-lifecycle-audit.md)
for dynamic window-rule composition. Complete renderer, capture, layer-rule and
fullscreen transition equivalence is outside this source-only batch.

## Pinned source evidence

- [appearance.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/appearance.rs): the seven types, custom CornerRadius decoder/fit/expansion, defaults and merges.
- [misc.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/misc.rs): Overview/OverviewPart owns workspace-shadow; [config lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs): top-level blur part.
- [window_rule.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/window_rule.rs): geometry-corner-radius, clip-to-geometry, background-effect, PopupsRule/ResolvedPopupsRules merging.
- [workspace layout](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs): compute_workspace_shadow_config and size/config updates.
- [layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs): LayoutElement::geometry_corner_radius; [tile.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tile.rs): fit/scale and clipping during sizing transitions.
- [background-effect rendering](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/render_helpers/background_effect.rs): option resolution, global blur gate, subregions/empty suppression and clipping.
- [blur rendering](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/render_helpers/blur.rs): BlurOptions and runtime pass clamp.
- [protocol region cache](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/background_effect.rs): committed region dirty tracking, None versus empty and damage.
- [mapped window](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mapped.rs) and [mapped layer](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layer/mapped.rs): effect consumer arguments and popup xray default.

## Corner radius decoding and consumers

`CornerRadius` has four internal f32 fields, in order top-left, top-right,
bottom-right, bottom-left. The public node takes **one or four positional numeric
arguments**, not per-corner properties or CSS two/three-value shorthand. One
argument is copied to all four corners; two/three arguments fail with missing
arguments, and a fifth reports an extra argument. Node/argument type annotations,
properties and children report errors. The default is all zero.

Integer arguments first convert to **i16**, then f32. Decimal arguments convert
to f32 directly; the decoder does not impose the same i16 bound on that branch.
Negative radii report a conversion error after decoding; they are not silently
clamped into successful zero settings. A conversion/type failure can produce a
recovery value, which is not evidence of successful config acceptance.

`fit_to(width,height)` applies one shared reduction factor: the minimum of 1
and each edge extent divided by the sum of its two corner radii. This preserves
relative radii while avoiding overlapping corners. `expanded_by` preserves
exact zero corners, grows nonzero ones, and clamps at zero when shrinking.
These geometry helpers are distinct from parser validation.

`WindowRule.geometry-corner-radius` and popup geometry-corner-radius use this
value; clip-to-geometry is a separate optional boolean rule. LayoutElement
returns zero for windowed fullscreen unless clip-to-geometry is explicitly true.
Tile rendering scales radius by `1 - expanded_progress` and gates geometry
clipping by fullscreen transition progress; this is not equivalent to one
unconditional radius for all sizing modes. The effect renderer expands clip
radius by 1 before fitting it to reduce edge bleed. Full transition pixel
behavior remains an independent fixture gate.

Driftwm has a scalar integer corner_radius per decoration/rule, with decoration
mode none vetoing clipping. It does not expose four independent fractional
radii or the niri clip-to-geometry rule; mapped-rule lifecycle differs too.

## Workspace shadows

WorkspaceShadow belongs to **overview.workspace-shadow**, not layout.shadow or
a per-window ShadowRule. It defaults enabled (`off=false`), offset `(0,10)`,
softness 40, spread 10, RGBA8 color `(0,0,0,80)`. Public WorkspaceShadowPart:

| Child | Shape / bound |
| --- | --- |
| `off`, `on` | Presence flags; resolved merge processes off then on, so on wins if both are present. |
| `offset` | ShadowOffset properties x/y in `[-65535,65535]`; a present value replaces both axes and missing axes decode to zero. |
| `softness` | One integer/decimal in `[0,1024]`. |
| `spread` | One integer/decimal in `[-1024,1024]`. |
| `color` | Color node through the previously audited grammar. |

Conversion to window-style Shadow sets on=!off, draw-behind-window=false and no
inactive-color override. `compute_workspace_shadow_config` then scales offset
axes, softness and spread by **logical view height / 1080**; color/enabled state
are unchanged. It runs on workspace construction, config and size updates. At
height 540, defaults therefore become offset `(0,5)`, softness 20 and spread 5.
This is a workspace-view normalization, not a public parse-time scaling rule.
Driftwm has no niri overview/workspace shadows.

## Blur tuning and background-effect rules

Top-level `blur` controls shared blur tuning. It does not request blur on every
window merely because its resolved `off` flag defaults false.

| BlurPart child | Grammar / resolved default |
| --- | --- |
| `off`, `on` | Presence flags; on wins the resolved merge. Default off=false. |
| `passes` | Optional u8 argument (0–255 at decoding); default 3. Renderer clamps to **1–31** at allocation/rendering, so decoded 0 is not a zero-pass effect. |
| `offset` | Integer/decimal `[0,100]`; default 3. |
| `noise` | Integer/decimal `[0,1000]`; default 0.02. |
| `saturation` | Integer/decimal `[0,1000]`; default 1.5. |

Present tuning values replace inherited ones. BackgroundEffectRule under window
or popup `background-effect` has separate optional children:

| Child | Meaning / grammar |
| --- | --- |
| `xray` | Explicit boolean argument; None means default according to effect visibility, false disables xray, true forces xray. |
| `blur` | Explicit boolean argument; None follows client blur request, false vetoes blur, true requests it even without a client region. |
| `noise`, `saturation` | Integer/decimal `[0,1000]`, stored as optional f64 overrides. Explicit zero/one differ from absence. |

Resolved BackgroundEffect defaults all fields to None. Merging replaces only
present values; explicit false is retained. When rendering, a nonempty client
region makes blur default true unless the rule is Some(false). Without a
nonempty region, only Some(true) requests blur. Effect visibility includes
xray=true, requested blur, positive noise or saturation different from 1. If an
effect is visible and xray is unspecified, xray becomes true. This rule is
applied before the global blur.off gate.

Actual blur execution is `requested_blur && !global_blur.off`. Noise/saturation
fall back to global tuning **only when blur actually executes**, otherwise to
0/1, then explicit effect overrides replace those values. Thus global blur off
is not a blanket veto of xray/noise/saturation; an auto/explicit xray path can
still be selected. The xray render path also requires an available xray context.
Driftwm's shared backdrop optimization is not a verified xray policy equivalent.

## Client regions, popup rules and policy differences

The protocol cache distinguishes no region (None) from an explicitly present
empty region (Some(empty)). Region changes become dirty after their surface
commit and damage the effect. `render_for_tile` uses nonempty state for blur
option selection, then `render_params_for_tile` applies another rule:

**Some(empty) suppresses the entire background-effect render**, even if a rule
requested blur or xray. None instead permits geometry-shaped forced effects.
A nonempty region is transformed from surface coordinates and uses surface
extent; clip-to-geometry controls geometry clipping, except the blocked-out
capture branch forces geometry-shaped clipping. Rule-only effects default to
geometry clipping. Do not equate a nonempty off-geometry region with the empty
protocol value: clipping and transformed extent are separate steps and need
independent fixtures.

Toplevels consume their own composed BackgroundEffect. Popup consumers use
PopupsRule/ResolvedPopupsRules, whose present radius replaces and background-
effect values merge through the same optional-value rule. Popup xray defaults
explicitly to **false** when unspecified, unlike the normal surface's automatic
xray default. Both mapped-window and mapped-layer popup paths do this. These
popup rules do not inherit every parent effect merely by being a popup.

For blur-only cases below, assume global blur enabled and no independent
xray/noise/saturation request:

| Client region | Rule blur | Pinned niri selection / region gate |
| --- | --- | --- |
| None | absent / false | No blur. |
| None | true | Geometry-shaped blur. |
| Nonempty | absent / true | Client-requested blur with transformed subregion. |
| Nonempty | false | No blur; explicit rule veto. |
| Explicitly empty | any | No effect render; empty-region gate suppresses it. |

Driftwm [effective_blur](../../src/config/types.rs) is stored rule blur OR the
global decoration blur default (with fullscreen gating of that default), not
niri's optional rule veto. A false rule does not veto a nonempty client request
or an enabled global decoration default. [BlurMaskSource](../../src/render/blur.rs)
uses nonempty client regions when available; empty/unset state or regions fully
clipped out of geometry can fall back to whole-window blur when its window_blur
policy is true. Without window blur, empty/clipped client-only requests produce
no frost. This policy is explicitly documented in [the tiling guide](../../docs/tiling.md)
and covered by [controlled blur pixel checks](../../src/tests/blur_pixels.rs).
Mapped rule flags remain sticky while global defaults can reload. No policy is
changed by this audit.

Driftwm defaults blur_radius=2 and blur_strength=1.1 with global decoration blur
false. These are its Kawase pass/spread controls, not an exact niri tuning map;
there are no matching configurable noise/saturation, independent xray or popup
background-effect rule trees. Retained blur/SSD masks being well-tested does not
make the differing rule selection compatible.

## Source-derived probes and remaining work

These are requirements for later independent fixtures, not parser executions:

| Probe | Expected distinction |
| --- | --- |
| Radius `5` vs `1 2 3 4` | Four equal radii vs TL/TR/BR/BL order. |
| Radius with 2, 3 or 5 arguments | Missing/extra argument error; not CSS shorthand. |
| Integer radius 32768 vs decimal 32768.0 | Integer i16 conversion fails; decimal branch has no i16 bound. |
| Radius -1 | Negative-value diagnostic, not silent successful clamp. |
| Uniform radius 10 fitted to a 10×20 rectangle | All corners reduce to 5. |
| Expand `(0,2,0,2)` by 1 | `(0,3,0,3)`; square corners remain square. |
| Workspace default shadow at view heights 1080 vs 540 | Unscaled defaults vs half offset/softness/spread. |
| Blur passes 0 / 255 / 256 | First two decode as u8 then renderer uses 1/31; 256 fails decoding. |
| Client nonempty region plus rule blur=false | Reference veto differs from driftwm's policy. |
| Client explicit empty region plus blur=true | Reference suppresses render; driftwm window-blur policy can keep whole-window frost. |
| Popup with unspecified xray plus blur request | Defaults to non-xray, unlike normal surface auto xray. |
| Global blur off with explicit noise/saturation or xray | Blur computation is disabled; other selected effects are not universally disabled. |

All named public appearance struct fields now have scoped decoder/default/local-
merge evidence across the core and effects audits. This is not all niri config:
the current inventory indexes seven modules and omits root/misc fields such as
Overview/OverviewPart themselves and layer-rule schemas. The next A0 task must
reconcile schema coverage and justified exclusions before auditing remaining
modules and ID/index/previous/next actions. Full output/placement/capture semantics,
BlockOutFrom application, derived structural diagnostics, geometry fixtures beyond
zero gaps and executable differential evidence remain open. A0/P0 still gates
feature expansion.
