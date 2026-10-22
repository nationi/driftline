# Pinned niri appearance decoder and merge audit

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 batch covers
Color, Gradient/interpolation, BorderRule/FocusRing/Border, ShadowRule/Shadow/
ShadowOffset, TabIndicatorPart/TabIndicator/Length, TabIndicatorRule and
InsertHintPart/InsertHint: 98 named schema fields across 15 structs. The enum
spellings they use are recorded too. CornerRadius, workspace shadows, blur and
background-effect rules remain separate audit work. Runtime override precedence
is traced separately in the [lifecycle audit](niri-config-lifecycle-audit.md); it
is not inferred from these local merges.

This is pinned source review, not execution of niri's parser or a rendering
oracle. The cases below are source-derived fixture requirements. No driftwm
runtime/config change or Exact compatibility claim is made. See the
[top-level layout audit](niri-layout-config-audit.md) for layout defaults and the
root-file empty-border exception; [the inventory](niri-tiling-inventory.md) links
audited rows here.

## Pinned sources and driftwm evidence

- [appearance.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/appearance.rs): the named types, custom `Color::decode_node`, `Color::from_str`, `GradientInterpolation::from_str` and their `MergeWith` implementations.
- [macros.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/macros.rs): `merge`, `merge_clone`, `merge_clone_opt`, `merge_color_gradient`, `merge_color_gradient_opt`, `merge_on_off`.
- [utils.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils.rs): numeric bounds/recovery and `Flag`; [layout.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/layout.rs): layout parts using these types.
- Driftwm [color parsing](../../src/config/parse_helpers.rs), [decoration defaults and applied rules](../../src/config/types.rs), [TOML schema](../../src/config/toml.rs), [native rendering](../../src/render/mod.rs) and [canonical config](../../config.reference.toml).

Driftwm config colors use `#RRGGBB` or `#RRGGBBAA` strings and byte RGBA values;
its parser does not expose niri's CSS-color grammar or numeric KDL form. Native
focused/unfocused borders and shadow enablement are related features, not proof
of niri geometry, defaults or rule equivalence. There is no separate focus ring,
urgent border color, border gradient, tabbed indicator or insertion-hint setting.
For example, driftwm decoration defaults have border width 0 and shadows enabled;
niri's resolved border defaults to off/width 4 and its shadow defaults to off.

## Color and gradient decoding

`Color.r/g/b/a` are internal unpremultiplied `f32` components, not named KDL
properties. The custom node decoder accepts either:

- One string, delegated to `Color::from_str` → `csscolorparser::parse(...).clamp().to_array()`.
  Do not invent a CSS feature/version matrix from that delegation alone.
- Four integer arguments R, G, B, A, each decoded as `u8` and divided by 255.
  Alpha is required; this is not a three-channel form with implicit opacity and
  not a 0–1 floating-point argument form.

A missing argument, invalid string or failed u8 conversion is a decoding error.
The string branch also reports extra arguments, properties, children and type
annotations; the numeric branch delegates structural validation to the derived
`ColorRgba` decoder. `Color::default()` is transparent zero, but feature-specific
resolved defaults explicitly set their own colors. `to_array_premul` multiplies
RGB by alpha at rendering conversion; multiplying `Color` itself changes alpha.
Do not multiply alpha into stored RGB twice when adapting this representation.

Gradient node **properties**, not children:

| Public property | Required/default and validation |
| --- | --- |
| `from`, `to` | Required string colors through `Color::from_str`; no four-argument color form in these properties. |
| `angle` | `i16`, default 180; no explicit angle-range restriction beyond that type in this decoder. |
| `relative-to` | Scalar `"window"` or `"workspace-view"`; default window. |
| `in` | String parsed by `GradientInterpolation::from_str`; default color space sRGB and shorter hue. Rust field name `in_` is not the public property. |

The `in` grammar is a whitespace-separated color space, optionally followed by
`<hue-method> hue`. Color spaces are exactly `srgb`, `srgb-linear`, `oklab`,
`oklch`; hue methods are `shorter`, `longer`, `increasing`, `decreasing` and may
only follow `oklch`. Extra trailing tokens, missing `hue` or an unknown token
are errors. `oklch` alone uses shorter hue. `Gradient::from(Color)` constructs a
solid gradient with angle **0**, unlike the decoded gradient's default **180**;
these are different constructors, not conflicting defaults.

## Border and focus-ring rules

`BorderRule` is the public subtree grammar used by both `border` and `focus-ring`.
Its derived default has both presence flags false and all optional values absent.
The resolved `FocusRing` and `Border` structs are not additional public subtree
grammars: their eight fields map through this same rule type.

| Public child | Shape / range |
| --- | --- |
| `off`, `on` | Presence children stored as bool flags, not the optional `Flag` wrapper. |
| `width` | One integer/decimal, inclusive `[0,65535]`. |
| `active-color`, `inactive-color`, `urgent-color` | Color nodes, either one string or four u8 arguments. |
| `active-gradient`, `inactive-gradient`, `urgent-gradient` | Gradient nodes with the properties above. |

Applying a BorderRule to a resolved Border first ORs `off`, then clears it if
`on` is present: **on wins when both are set**. FocusRing converts to Border,
uses the same merge and converts back. Width omission preserves the inherited
width. Each state has an independent color/gradient pair, with precedence below.
No public gradient or urgent-state equivalent exists in driftwm's decoration
schema, even though width and two solid border colors have partial counterparts.

## Shadow rules

`ShadowRule` public children:

| Public child | Shape / range |
| --- | --- |
| `off`, `on` | Presence bool flags. |
| `offset` | `ShadowOffset` node with optional **properties** `x`, `y`, each integer/decimal in `[-65535,65535]`; omitted axes decode to 0. |
| `softness` | One integer/decimal in `[0,1024]`. |
| `spread` | One integer/decimal in `[-1024,1024]`. |
| `draw-behind-window` | One explicit boolean argument (`Option<bool>` with unwrap(argument)); not the optional-argument Flag form. |
| `color`, `inactive-color` | Color nodes. Inactive color is an optional override, absent by default. |

The resolved Shadow defaults are listed in the top-level audit. A present offset
replaces both axes: `offset x=3` produces `(3,0)`, not `(3,5)` by inheriting the
resolved default Y. A present numeric/boolean/color value replaces its inherited
value. An omitted inactive color preserves the previous optional override; no
clear-to-None color spelling is provided by this decoder.

Applying ShadowRule **directly to resolved Shadow** ORs `on`, then clears it
when `off` is present: **off wins when both are set**. This differs from the
Border/FocusRing/TabIndicator/InsertHint resolved merges. Separately, merging
one ShadowRule into another uses `merge_on_off`, which processes off then on,
so **on wins at that rule-composition stage**. Preserve the distinction; the
[lifecycle audit](niri-config-lifecycle-audit.md) identifies the scoped config contexts using each path.
Do not summarize both operations as an unconditional last/on/off-wins rule.

## Tab indicator and insertion hint

`TabIndicatorPart` public children (16 fields):

| Public child | Shape / range |
| --- | --- |
| `off`, `on` | Presence bool flags; resolved merge gives on precedence. |
| `hide-when-single-tab`, `place-within-column` | `Option<Flag>`: bare true, or explicit boolean argument, including false override. |
| `gap` | One integer/decimal in `[-65535,65535]`. |
| `width`, `gaps-between-tabs`, `corner-radius` | One integer/decimal in `[0,65535]`. This corner radius is scalar, not the separate four-corner CornerRadius decoder. |
| `length` | Node with optional **property** `total-proportion: f64`; no explicit range bound in this decoder. Empty length sets it to None, unlike omission of the entire length child. |
| `position` | One scalar, exactly `"left"`, `"right"`, `"top"`, `"bottom"`. |
| `active-color`, `inactive-color`, `urgent-color` | Optional Color nodes. |
| `active-gradient`, `inactive-gradient`, `urgent-gradient` | Optional Gradient nodes. |

Resolved defaults are in the layout audit: enabled, left, width 4, gap 5, length
Some(0.5), flags false, gaps-between-tabs/corner-radius 0; all three colors and
all three gradients start absent. Present length/position replace as a whole;
optional-argument flags can turn inherited true back to false. The separate
`TabIndicatorRule` has only the six state-color/gradient children and merges
those pairs with the same optional color/gradient macro. It cannot configure
position, length, width or on/off just because TabIndicatorPart can.

`InsertHintPart` has only `off`, `on`, `color`, `gradient` children. It uses the
resolved on-wins rule and the nonoptional color/gradient merge. Defaults are
enabled, RGBA8 `(127,200,255,128)`, no gradient. Tab indicators and insert hints
remain Missing in driftwm; an audited grammar does not implement them.

## Merge precedence and fixture probes

For each color/gradient pair, the macro processes a present color first: replace
the color and clear the inherited gradient. It then applies a present gradient.
Thus a gradient in the **same part** wins regardless of textual node order,
while a later color-only part clears an earlier gradient. An absent pair leaves
both inherited values untouched. The `_opt` variant stores the color in Some
but has the same gradient-clearing/precedence logic. Supplying a gradient alone
does not erase the saved color; it changes the selected gradient.

`BorderRule::merge_with(Self)` and `ShadowRule::merge_with(Self)` normalize
on/off flags with `merge_on_off`: a present off clears saved on, then a present
on clears saved off. Their optional values replace only when present. This
composition stage is separate from applying the combined rule to resolved
appearance. TabIndicatorRule merges only its six optional color/gradient values.

Source-derived probes for future independent decoder/merge fixtures:

| Operation / input | Required observation |
| --- | --- |
| `active-color 255 128 0 64` | Stored unpremultiplied `(1,128/255,0,64/255)`. |
| `active-color 255 128 0` / `active-color 256 0 0 255` | Missing alpha / u8 conversion error. |
| `active-gradient from="#ff0000" to="#0000ff"` | Decoded angle 180, relative-to window, sRGB/shorter. |
| Gradient `in="oklch longer hue"` / `in="srgb longer hue"` | First accepted by the custom string parser; second rejected. |
| Gradient `in="oklch longer"` / `in="oklch longer hue extra"` | Missing hue / trailing-token error. |
| Border with inherited gradient, then color-only part | Gradient cleared; solid color selected. |
| Border with color and gradient in the same part | Gradient selected regardless of those two nodes' order. |
| Border/FocusRing resolved merge with both off/on flags | Enabled after applying the part. |
| Shadow resolved merge with both off/on flags | Disabled after applying the part. |
| ShadowRule composition with both flags, then resolved merge | Composed rule has on=true/off=false, so resolved shadow becomes enabled. |
| Shadow `offset x=3` overriding `(0,5)` | New offset `(3,0)`. |
| Tab `hide-when-single-tab false` overriding true | False, not inheritance. |
| Tab `length {}` overriding Some(0.5) | Present length value with total_proportion None. |
| `softness 1025` / `spread -1025` / tab `width -1` | Numeric decoder errors; recovery zero is not successful acceptance. |

Remaining A0 work: complete override/placement permutations and independent
fixtures beyond the scoped lifecycle audit; unaudited appearance schemas (CornerRadius, workspace shadows, blur/background effects); remaining
configuration modules; ID/index/previous/next action addressing; wider independent
operation fixtures. Full CSS parser dependency behavior, all derive-generated
structural diagnostics and visual geometry remain outside this source-only gate.
