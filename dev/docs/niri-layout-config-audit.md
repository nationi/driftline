# Pinned niri layout configuration audit

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 batch audits
all 15 top-level `Layout` / `LayoutPart` fields, their public KDL names, defaults
and the shared size/flag/strut decoders. Core appearance decoding and merging is
covered by the [appearance audit](niri-appearance-config-audit.md); remaining
appearance types, window rules, output/workspace overrides and action addressing
still need auditing. No driftwm
configuration or runtime behavior changes in this batch.

Evidence is source inspection of the pinned checkout, not execution of niri's
parser or recordings of its layout. Examples below are source-derived acceptance
requirements for later fixtures. A checked reference decoder does not make the
driftwm implementation Exact. The [generated inventory](niri-tiling-inventory.md)
links to this audit; regenerate it rather than editing its rows by hand.

## Reference symbols

All links target the pinned revision:

- [layout.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/layout.rs): `Layout::default`, `LayoutPart`, `Layout::merge_with`, `PresetSize`, `DefaultPresetSize::decode_node`, `CenterFocusedColumn`, `Struts`.
- [utils.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils.rs): `Flag`, `FloatOrInt::raw_decode`, `expect_only_children`.
- [config lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs): `ConfigPart::decode_children`, the `layout` branch and root/include recursion distinction.
- [appearance.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/appearance.rs): defaults for `FocusRing`, `Border`, `Shadow`, `TabIndicator`, `InsertHint`, `DEFAULT_BACKGROUND_COLOR`.
- [IPC lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs): `ColumnDisplay::from_str`.
- [workspace config](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/workspace.rs): `WorkspaceLayoutPart::decode_node`; [output config](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/output.rs): `Output.layout`.
- [scrolling layout](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs): `Column::resolve_column_width`, `resolve_preset_size`, `compute_working_area`; [workspace layout](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs): `resolve_default_width`.

## Public layout children

KDL names below are decoded child-node names, not TOML keys. `Layout` is the
resolved value; `LayoutPart` carries optional overrides. An omitted child in a
part preserves the inherited value. Defaults describe `Layout::default`, not
values explicitly set in the shipped example config. Nested appearance defaults
here are summaries; their public child/property grammars and local merges are
recorded in the appearance audit.

| Public child | Shape and reference default | driftwm evidence / compatibility |
| --- | --- | --- |
| `focus-ring` | `BorderRule` subtree; enabled, width 4; active RGBA8 `(127,200,255,255)`, inactive `(80,80,80,255)`, urgent `(155,0,0,255)` | Missing as a separate niri focus-ring facility. Native decoration borders are not a verified equivalent. |
| `border` | `BorderRule` subtree; off, width 4; active `(255,200,127,255)`, inactive/urgent as above. Root empty-subtree exception below. | Partial: `[decorations]` border width/colors exist, but appearance grammar, defaults, gradients and placement semantics differ. |
| `shadow` | `ShadowRule` subtree; off; offset `(0,5)`, softness 30, spread 5, draw-behind-window false, color RGBA8 `(0,0,0,119)`, inactive override absent | Partial: driftwm has native shadows; no equivalence for these options or defaults has been established. |
| `tab-indicator` | `TabIndicatorPart` subtree; enabled, left, width 4, gap 5, length total-proportion 0.5; hide-when-single-tab/place-within-column false; gaps-between-tabs/corner-radius 0 | Missing: no tabbed column/indicator implementation. Nested grammar is recorded in the appearance audit. |
| `insert-hint` | `InsertHintPart` subtree; enabled, RGBA8 `(127,200,255,128)`, gradient absent | Missing: no layout-native insertion hint. Workspace override restrictions below. |
| `preset-column-widths` | Children `proportion <f64>` or `fixed <i32>`; default proportions `[1/3, 1/2, 2/3]`; empty list resets to these defaults after merge | Divergent: `Stage::cycle_column_width` hardcodes `[1/3, 1/2, 1]`, with no public list, fixed presets or matching-index behavior. |
| `default-column-width` | Optional single `PresetSize` child; default `proportion 0.5`. Empty subtree explicitly resolves to `None`. | Partial: `ColumnWidth::default()` is 0.5, but no public setting, client-chosen default or nested override semantics. |
| `preset-window-heights` | Same size-child grammar; default proportions `[1/3, 1/2, 2/3]`; empty list resets after merge | Missing: no configurable height presets or preset-height cycling. Current equal-height arrangement does not implement this option. |
| `center-focused-column` | One scalar: `"never"`, `"always"`, `"on-overflow"`; default `never` | Partial: existing reveal/fit policy and zero-gap reference traces cover a subset; no public centering policy or full on-overflow semantics. |
| `always-center-single-column` | `Flag`: bare node or boolean argument; default false | Missing public setting and dedicated policy. A centered result in one scene is not evidence for this option. |
| `empty-workspace-above-first` | `Flag`; default false | Missing: no niri workspace lifecycle. Disallowed inside `workspace.layout`. |
| `default-column-display` | One string, exactly `"normal"` or `"tabbed"`; default normal | Missing configurable display mode; current normal columns do not establish parity for tabs or rule overrides. |
| `gaps` | One integer/decimal in inclusive `[0,65535]`; resolved `f64`, default 16 | Divergent: shared `[snap].gap` defaults to 12 and is rounded to an integer in `refresh_tiling`; geometry/gap accounting differs. |
| `struts` | Subtree with `left`, `right`, `top`, `bottom`; each integer/decimal in inclusive `[-65535,65535]`, omitted sides 0 | Missing public struts. Layer-shell usable area already exists, but does not supply this additional signed/fractional adjustment. |
| `background-color` | `Color` child; default unpremultiplied RGBA `(0.25,0.25,0.25,1)` | Partial: driftwm's configurable canvas background defaults to a dot grid; it is not a per-layout/workspace solid-color equivalent. Color decoding is recorded in the appearance audit. |

Driftwm sources: [column policy](../../src/stage/tiling.rs),
[pure width/geometry](../../src/layout/scrolling/mod.rs),
[compositor policy](../../src/state/tiling.rs),
[config parsing/defaults](../../src/config/mod.rs),
[canonical option reference](../../config.reference.toml), and
[current reference trace limits](niri-reference-traces.md). These evidence paths
support the compatibility labels; no new runtime parity test is claimed.

## Size decoders

`PresetSize` has child variants `proportion` with an `f64` argument and `fixed`
with an `i32` argument. This declaration does not impose a positive-value range.
Do not invent parser rejection for zero/negative sizes from driftwm's pure
layout validation: runtime resolution/clamping and client constraints are a
separate audit. `From<PresetSize> for SizeChange` multiplies a proportion by 100
when constructing `SetProportion`; `proportion 0.5` is not `0.5%`.

`DefaultPresetSize` uses `expect_only_children`: node arguments, properties and
type annotations emit decoding errors. Zero children means `None`; one child
means `Some(PresetSize)`; a second child emits an error. The outer
`Option<DefaultPresetSize>` in `LayoutPart` is a distinct inheritance layer:
omitted = preserve, empty = clear, one child = replace. An empty preset list has
a different meaning: `Layout::merge_with` restores the built-in list.

`Flag` defaults its optional boolean argument to true. Bare and explicit true
are equivalent; explicit false remains a present override and can undo an
inherited true value. `FloatOrInt` accepts integer or decimal literals and checks
its inclusive type-specific bounds. Other literal types, conversion failures or
out-of-range values emit errors; the decoder's recovery value is zero. That
recovery value must not be presented as successful acceptance of invalid input.

Source-derived decoder probes, to turn into independent reference fixtures:

| Input inside `layout` | Expected effect / diagnostic |
| --- | --- |
| no `default-column-width` child | Keep inherited width; fresh resolved default is proportion 0.5. |
| `default-column-width {}` | Explicitly clear inherited width to `None`; workspace sizing may use client choice. |
| `default-column-width { fixed 800; }` | Replace with `Some(Fixed(800))`. Fixed units are discussed below. |
| `default-column-width 800` | Error: arguments are forbidden on the wrapper. |
| `default-column-width { fixed 800; proportion 0.5; }` | Error: more than one child. |
| `preset-column-widths {}` / `preset-window-heights {}` | Replace with empty list, then restore that list's built-in defaults. |
| `always-center-single-column` | Present true override. |
| `always-center-single-column false` | Present false override, not omission. |
| `gaps 0.5` / `gaps 65535` | Within decoder range; preserve the numeric value. |
| `gaps -1` / `gaps 65536` | Decoder error; not a clamp-to-boundary setting. |
| `struts { left -0.5; top 1.25; }` | Signed fractional values; right/bottom remain zero in this Struts value. |
| `default-column-display "tabs"` | Error; accepted spelling is `"tabbed"`. |

## Merge and lifecycle boundaries

`Layout::merge_with` replaces preset vectors rather than concatenating them,
merges nested appearance parts through their own `MergeWith` implementations,
and assigns a present `Struts` value as a whole. Thus a later `struts { top 10; }`
does not retain a previous left strut: omitted sides decode to zero. Scalar/flag
omission preserves the previous resolved value. This is not a universal
"empty subtree does nothing" rule.

`ConfigPart::decode_children` has a root-file compatibility exception: at
recursion 0 an explicitly present `border {}` without `on`/`off` is treated as
`on`. Omitted border keeps the disabled default. This exception is not applied
inside included files. `shadow {}` does not enable shadows. Do not infer any
of these results merely from `Border::default` or Rust field optionality.

Output config carries `Option<LayoutPart>`; workspace config wraps it in
`WorkspaceLayoutPart`. That custom decoder rejects `empty-workspace-above-first`
and `insert-hint` within `workspace.layout`: they are monitor-level concepts.
The complete global → output → workspace override precedence, hot reload and
window-rule lifecycle still need a dedicated audit. Default width rule resolution
already shows another distinction: a present rule can explicitly clear width,
whereas an absent floating-window default does not inherit the tiling width
(`Workspace::resolve_default_width`).

## Geometry implications for later fixtures

A public fixed preset is a window-content size: `resolve_preset_size` returns
`ResolvedSize::Window`. A proportional preset resolves to a tile size:
`(working_extent - gaps) * proportion - gaps - extra_size`, where the extra size
can come from a tab indicator. `Column::resolve_column_width` uses the same
proportional formula, while its internal fixed width is already resolved. Do
not conflate public `PresetSize::Fixed` with internal `ColumnWidth::Fixed`, or
with driftwm's fixed *visual frame* width.

For a 1200-wide working area, gaps 16, zero decoration/extra size and proportion
0.5, the reference formula yields 576 logical tile pixels. Driftwm's current
pure width calculation yields 600 frame pixels before client constraints. The
existing zero-gap trace corpus cannot expose this difference. Later geometry
fixtures must independently cover nonzero gaps, borders/titlebars, struts,
fractional scale, preset content/frame units and centering policy. Parser
acceptance alone is not a geometry acceptance gate.

## Next audit

Audit global/output/workspace/window-rule override precedence and lifecycle.
The appearance audit records Color, Gradient, BorderRule, ShadowRule,
TabIndicatorPart and InsertHintPart grammar and local merges, including
context-dependent shadow flag precedence; their application contexts still
need tracing. Remaining appearance schemas (CornerRadius, workspace shadows,
blur/background effects), other indexed modules and ID/index/previous/next
action addressing are open. A0/P0 remains unfinished; no new public setting or
action should be added merely on the strength of these source tables.
