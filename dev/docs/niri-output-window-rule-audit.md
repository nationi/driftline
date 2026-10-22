# Pinned niri remaining output and window rules

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds links for 28 output fields and 14 remaining window-rule fields. Together
with earlier identity, lifecycle and effects audits, all named fields in these
two modules now have review links. This is not executed parser/DRM/render evidence
or completed behavior parity. No driftwm runtime or config changes are made.

## Pinned sources

- [niri-config/src/output.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/output.rs)
- [niri-config/src/window_rule.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/window_rule.rs)
- [niri-ipc/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/utils/scale.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/scale.rs)
- [src/utils/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/mod.rs)
- [src/backend/tty.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/backend/tty.rs)
- [src/window/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mod.rs)
- [src/window/mapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mapped.rs)
- [src/layout/floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs)
- [src/layout/tile.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tile.rs)
- [src/handlers/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/mod.rs)

Earlier [identity](niri-input-output-integration-audit.md#output-identity-and-lookup),
[lifecycle](niri-config-lifecycle-audit.md), [effects](niri-effects-config-audit.md),
[focus](niri-input-focus-audit.md) and [layer/overview](niri-layer-overview-audit.md)
audits retain their explicit limits. Fields already linked there are not counted
again merely because their consumer is explained here.

## Output defaults and runtime geometry

Output defaults off=false, transform=Normal, focus-at-startup=false; optional
scale/position/max-bpc/mode/modeline/VRR/colors/hot-corners/layout are absent.
An absent VRR node means off, an empty node means always on, and an on-demand=true
property requests conditional VRR. Off and focus-at-startup are flag children;
position has required i32 x/y properties. Transform delegates to the IPC parser,
accepting normal, 90, 180, 270, flipped, flipped-90, flipped-180 and flipped-270.
These are per-output values, not a fieldwise OutputPart merge schema.

Scale decodes FloatOrInt [0,10], but reload clamps runtime scale to [0.1,10] and
rounds to nearest N/120. Omission first tries winit scale, otherwise monitor scale
inference from dimensions and resolution. The inference uses quarter steps 1–4
and a minimum logical area; unavailable physical dimensions yield 1. Thus decoded
zero is not runtime zero. Transform combines panel orientation and configured
transform; the winit connector is explicitly forced to Flipped180 in the traced
reload path. All transform/mode-change geometry acceptance remains open.

Output positioning sorts by stable identity, then places explicit positions first.
An explicit rectangle overlapping an already placed output is rejected with a
warning and falls back to automatic placement. Automatic placement uses the
rightmost placed edge at y=0 (or origin for the first output). Signed positions
are valid; acceptance depends on overlap. This is not arbitrary overlapping
viewport placement and must not be inferred from Position's numeric grammar.

focus_default_monitor starts from the first sorted output and uses the first
configured focus-at-startup entry matching a connected output, then focuses it
and moves the cursor there. This is a startup decision, not a promise to steal
focus whenever that config reloads. TTY off suppresses/disables connector outputs;
lid/debug device policy can also affect output existence, as already scoped.

Per-output hot corners use whole-value override and the previously audited
one-pixel/latch policy. Backdrop color falls back to global overview backdrop,
with alpha forced to one. Deprecated output background-color is copied only into
an existing output LayoutPart whose background_color is absent in the traced
reload/setup paths; do not describe it as an unconditional layout-independent
fallback. A present layout background takes precedence. This source edge still
needs parser/reload/render fixtures; no compatibility behavior is changed here.

## Modes, modelines and connector properties

Mode requires one string WIDTHxHEIGHT with optional @REFRESH, plus optional boolean
custom property default false. ConfiguredMode stores u16 dimensions and optional
f64 refresh. Parsing splits x/@ and delegates to numeric parsing; do not invent
positive-dimension/finite-refresh validation absent from that scalar parser.
For custom=true the config decoder additionally requires a refresh and rejects
refresh <=0. Arguments beyond the first, children, type names and unexpected
properties are diagnosed. Accepted numeric/error boundaries require execution.

TTY first tries a configured modeline; conversion failure warns and falls back
to mode selection. A custom mode with refresh uses CVT generation. Advertised mode
selection matches dimensions, ignores interlaced candidates in that target search,
and compares requested refresh rounded to mHz; with refresh omitted it prefers
the greatest advertised vrefresh among matching dimensions. If no target matches,
it falls back to a preferred mode and finally the first available mode, recording
fallback. Do not generalize the target-search interlace filter to every fallback.

Modeline is eleven positional values: f64 clock in MHz; u16 hdisplay, hsync-start,
hsync-end, htotal, vdisplay, vsync-start, vsync-end, vtotal; then string horizontal
and vertical polarity. Polarity accepts +hsync/-hsync and +vsync/-vsync. The custom
decoder diagnoses type names, children, properties and excess arguments. It checks
strict display <sync-start <sync-end <total on both axes and positive totals.
The clock is decoded as f64 without a positivity check in this decoder; backend
conversion has a separate failure/fallback path. Partial construction after a
diagnostic is not config acceptance.

MaxBpc has a custom scalar wrapper (no named field rows): integer converted to u8
then restricted to 6,8,10,12,14,16. TTY connector property application checks the
connector's range separately and can fail/warn; parsing a valid enum is not proof
that the hardware applied it. Hardware modesetting/VRR/format parity remains
Unverified integration. Existing surfaceless GPU matrices do not prove scanout.

## Remaining window scalar and popup consumers

Window rules resolve in order using the earlier match/exclude/local-merge policy.
Present optional scalars overwrite, absence preserves earlier rules; explicit
false is meaningful. The source schema groups some fields by lifecycle, but
actual consumers determine when a resolved value is read.

| Field | Shape and traced consumer |
| --- | --- |
| on-xdg-activate | Optional ignore/set-urgent/focus scalar enum. Current mapped resolved rule controls activation requests; it is not solely an initial-configure setting. Token age/urgency policy remains in the focus audit. |
| draw-border-with-background | Optional boolean argument; fallback is !has_ssd. Tile rendering selects border/background behavior, and the focus ring inherits that background choice only when the border is off. |
| baba-is-float | Optional boolean argument. True adds the existing time/view-height bob offset, rounded physically, and keeps animations ongoing. It does not make the window a floating layout tile. |
| block-out-from | Optional shared screencast/screen-capture target policy. Mapped body can render a solid substitute; regular and IME popups are omitted on blocked targets. Output/capture privacy remains in scope. |
| variable-refresh-rate | Optional boolean argument. In output on-demand VRR mode, the consumer searches windows for this output with true and a surface whose primary scanout output equals it; it is not just keyboard focus. |
| scroll-factor | Optional FloatOrInt [0,100]. Multiplies device axis factors in focused-root client delivery, as traced in the input/output audit. |
| pinch-sensitivity | Optional FloatOrInt [0,100]. Multiplies device sensitivity in the exponent applied to forwarded pinch scale, not gesture delta/rotation. |
| tiled-state | Optional boolean argument. Explicit true/false overrides the protocol Tiled state decision; it is not structural tiling membership. |
| popups | Nested PopupsRule merges its optional opacity/radius and background-effect fields into resolved popup rules. |

Tiled-state absence uses negotiated xdg-decoration mode when available, otherwise
KDE decoration mode OR prefer-no-csd, otherwise prefer-no-csd. The helper sets or
unsets all four tiled states. Driftwm deliberately advertises these states for
CSD windows under its own compatibility policy; this is not reference rule parity.

Popup opacity and ResolvedPopupsRules.opacity are optional raw f32 values, not
decoder-bounded [0,1]. Rendering uses default 1 and clamps to [0,1], multiplied by
the supplied parent alpha. IME popups use default popup rules, unlike ordinary
xdg popups. Blocked targets skip popup rendering before that loop. Popup geometry,
effects and xray retain their separate earlier audit; complete retained-content,
privacy and reload combinations still need independent fixtures.

## Floating position

FloatingPosition requires x/y FloatOrInt [-65535,65535] properties; relative-to
defaults top-left, with top-right, bottom-left, bottom-right, top, bottom, left,
right alternatives. It is not a global canvas coordinate or ratio.
stored_or_default_tile_pos first uses stored position scaled by working area,
then the rule default. Right/bottom anchoring subtracts tile size and supplied
inset from the corresponding working-area extent; top/bottom centers the x axis
before adding x, left/right centers y before adding y. Finally working-area origin
is added. Insets can be negative; frame size is tile size. Other placement/clamp
call sites and live rule changes do not follow merely from this helper.

Driftwm canvas placement and exiting tiling for free manipulation do not provide
reference layout-native floating or workspace ownership. Existing min/max client
constraints and native decorations do not implement all these dynamic rules.

## Remaining acceptance

Required executable fixtures cover parse boundaries/diagnostics; scale zero and
N/120 rounding; panel/config transforms; output order/overlap/fallback; startup
focus versus reload; modeline/CVT/advertised mode and connector failures; VRR
visibility and output ownership; deprecated color precedence; dynamic scalar
false/metadata/reload transitions; protocol tiled states; popup type/opacity/
block-out; and stored versus default floating positions with nonzero work areas,
mixed frames and negative insets. This document reports source review only.

Next A0: public binding/action addressing, remaining misc/recent-windows grammar
and independent reference fixtures beyond zero gaps. Named field review coverage
does not close P0/P11/P13 or authorize feature expansion without acceptance.
