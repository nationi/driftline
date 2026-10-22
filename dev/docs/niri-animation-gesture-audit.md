# Pinned niri animation and gesture configuration

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 audit covers
40 named animation fields and 23 gesture fields, plus the custom animation
parser's tuple wrappers/enums. Six animation rows configure unrelated notification,
exit-confirmation or screenshot UI; their field-specific Outside scope is recorded
below. This is source review, not execution of niri, shader compilation, parser
acceptance or complete gesture/animation parity. Driftwm runtime is unchanged.

## Pinned sources

- [niri-config/src/animations.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/animations.rs)
- [niri-config/src/gestures.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/gestures.rs)
- [src/animation/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/animation/mod.rs)
- [src/animation/clock.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/animation/clock.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/tile.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tile.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/render_helpers/shaders/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/render_helpers/shaders/mod.rs)

## Animation defaults and merge

Animations defaults to off=false, slowdown=1. All individual defaults have
off=false and no custom shader. Tuple wrappers have no named schema rows;
their public nodes still matter. Spring values below are damping ratio,
stiffness and epsilon respectively.

| Node | Default kind/parameters |
| --- | --- |
| workspace-switch | Spring 1 / 1000 / 0.0001 |
| horizontal-view-movement, window-movement, window-resize, overview-open-close | Spring 1 / 800 / 0.0001 |
| window-open | Easing 150 ms, ease-out-expo |
| window-close | Easing 150 ms, ease-out-quad |
| recent-windows-close | Spring 1 / 800 / 0.001 |
| config-notification-open-close | Spring 0.6 / 1000 / 0.001 |
| exit-confirmation-open-close | Spring 0.6 / 500 / 0.01 |
| screenshot-ui-open | Easing 200 ms, ease-out-quad |

The last three UI facilities are Outside tiling behavior scope. Their six owning
Animations/AnimationsPart fields remain indexed and their decoder shares the
rules below. This excludes their UI motion only, not workspace overview, MRU
focus, window animation or rendered capture privacy.

AnimationsPart off/on are flag children, not Flag booleans with arguments.
Merging ORs off, then on clears off, so on wins when both are present. Omitted
slowdown preserves an earlier part; present slowdown decodes FloatOrInt in
[0, i32::MAX]. Each present individual animation replaces its whole previous
value, including shader source and off/kind. It is not a nested parameter merge.
A later empty window-resize therefore restores its default spring and removes a
previous custom shader/off setting. Root on does not clear individual off values.

## Custom animation decoding

Each wrapper supplies its own default to Animation::decode_node. The public body
accepts off, spring, duration-ms, curve and, only for window-open/close/resize,
custom-shader. Spring and easing parameters together emit diagnostics in either
order. Duplicate off, spring, duration-ms or curve emit diagnostics; do not mistake
the decoder's continued construction for an accepted config. Unknown children,
extra curve arguments/properties/children are diagnosed.

An empty body or only off/custom-shader preserves that wrapper's default kind.
Any duration-ms or valid curve selects easing. Missing easing values inherit an
easing default; if the wrapper defaults to spring, fallback is 250 ms and
ease-out-cubic. Thus window-resize { duration-ms 100 } selects cubic easing,
whereas an empty window-resize uses spring. duration-ms is u32. Curves are linear,
ease-out-quad, ease-out-cubic, ease-out-expo or cubic-bezier with four coordinates.
Bezier x coordinates decode FloatOrInt [0,1], y coordinates [i32::MIN,i32::MAX];
do not impose a [0,1] y restriction. Curve is an enum, not another indexed struct.

Spring uses required properties damping-ratio:f64, stiffness:u32, epsilon:f64,
not child nodes or partial defaults. Validation requires damping in [0.1,10],
stiffness >=1 and epsilon in [0.00001,0.1]. Missing properties, arguments, child
nodes, type names and unexpected properties are diagnosed. Boundary/error
behavior still needs actual parser fixtures.

Custom-shader accepts inline source or a path property. When path is present,
the path parser is used, records the expanded path in Includes and reads UTF-8
source; extra arguments/properties/children or unreadable/missing paths fail or
emit errors. Inline/path precedence is not a generic nested merge. On reload,
changed open/close/resize source is passed to its respective renderer setter.
Compilation failure warns and returns before replacing the previous program;
removing source replaces it with None. This is distinct from parser failure and
requires GPU/reload evidence before any parity claim.

## Animation clock and consumers

Startup/reload sets the shared clock rate to 1/max(slowdown,0.001) and completion
flag to root off. Clock::set_rate clamps to [0,1000]; slowdown=0 therefore means
rate 1000, not disable. Animation::new divides initial velocity by max(rate,0.001)
before construction. Per-animation off yields zero-duration/off state.
is_done and is_clamped_done honor global completion immediately, but value_at
returns from when at <= start_time before checking completion. No elapsed time
can therefore retain the initial value within the starting event-loop iteration.
Clamped spring completion is distinct from full duration; easing interpolates its
curve and spring values have a numerical safety clamp.

Options::from_config copies animation/gesture settings into layout options;
update_config propagates them. Creation consumers include monitor workspace
switch, scrolling horizontal view movement, tile movement/open/resize and closing
snapshots. Overview's consumer is traced in the
[overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state).
Global shared-clock changes can affect already running animations; copied
individual configs are not proof that reload recreates every in-flight animation.
Full spring solver/duration, curve numerics, velocity continuity, snap/cancellation,
MRU and every reload permutation remain executable acceptance work.

## Gesture defaults and merge

Gestures contains DnD edge view scroll, DnD edge workspace switch and hot corners.
These nodes are not the public decoder for every trackpad swipe/pinch gesture.

| Setting | View scroll | Workspace switch | Decoder |
| --- | --- | --- | --- |
| trigger-width / trigger-height | 30 | 50 | FloatOrInt [0,65535] |
| delay-ms | 100 | 100 | u16 |
| max-speed | 1500 | 1500 | FloatOrInt [0,1000000] |

DnD Part nodes merge individual present fields; omitted fields preserve previous
parts. Empty DnD nodes do not reset prior values. HotCorners is instead replaced
as a whole when present. Its off and four corner fields are flag children,
default false; there is no corresponding on node. An empty replacement clears
previous off/explicit corners, allowing the runtime's top-left fallback.

## Dnd edge consumers

Workspace view scrolling computes horizontal penetration using its working area,
intentionally without extra configured layout struts. Pointer x is clamped to
[0,width], trigger width to [0,width/2]. Left penetration is negative, right
positive, otherwise zero; trigger <0.01 suppresses motion. Penetration divided
by trigger width is multiplied by the caller's speed factor, then forwarded to
the scrolling gesture. The gesture must already be a DnD gesture, not an ordinary
view gesture. It uses unadjusted clock time, resets the nonzero delay timer when
delta becomes zero, waits while elapsed <delay-ms, then integrates delta * elapsed
seconds * max-speed. Changing sign without zero does not itself reset that timer.
Bounds clamp view movement and adjust tracker offset so excess does not build up.

Workspace edge switching similarly uses normalized vertical penetration and
unadjusted time/delay/speed. It also restricts x to the centered overview-scaled
workspace strip; its right edge is excluded. Trigger height is clamped to half
the working-area height. Accumulated distance /1500 becomes workspace-index
movement; min/max clamp also adjusts the start index to discard excess. The
1500 movement constant is distinct from configurable max-speed. Delay/max-speed
are real-time based here, unlike the adjustable animation clock.

Ordinary view touchpad motion separately uses working-area width /1200 as its
normalization factor; it is not controlled by DnD trigger-width. Full swipe axis
selection, velocity projection, snap targets, interrupted gestures, DnD topology
changes and workspace gesture endings need independent fixtures beyond this
config-consumer audit.

## Hot corner consumers

Per-output hot-corners, if present, replaces the global value. off disables all
corners. If no right/bottom corner is set, top-left is implicitly enabled even
though all stored booleans default false. Explicit top-left can coexist with
other corners. Hit boxes are 1x1 logical rectangles at corners of the ceiled
output geometry, not a configurable edge band.

Relative/absolute pointer handling toggles overview on entry when the picked
contents marks hot_corner and pointer focus is None. A latch prevents repeated
toggles while inside. The grab filter blocks ResizeGrab, SpatialMovementGrab and
non-move MoveGrab; DnD is allowed. Click grabs can retain surface focus, which
also prevents triggering. Touch does not trigger this pointer path. These checks
are not equivalent to unconditionally running an action on any edge hit.

## Driftwm differences and remaining probes

Driftwm has camera_speed and effects.animation_speed with frame-rate-independent
remaining-distance factors, and existing opening/closing/resize retention. Those
are not niri's per-event spring/easing/shader settings or shared slowdown clock.
Its configurable hot-corner actions, threshold and fullscreen/drag suppression
are not the reference one-pixel overview toggle. Existing canvas edge pan and
horizontal tiling momentum do not implement the audited DnD workspace/view policy.
These similarities remain Partial/Divergent/Missing, never Exact by name.

Required independent acceptance includes repeated empty/partial animation and
DnD nodes; root on/off versus individual off; spring/easing switching and all
parser diagnostics; exact-time clock/off/slowdown boundaries; in-flight reload
and shader failure recovery; zero/half-size trigger bands; delayed edge entry,
exit and sign reversal; zero speed; fractional geometry/struts; scaled overview
strip boundaries; hot-corner output overrides/latches/grabs; and interrupted
swipe/DnD snap/topology sequences. Source-derived arithmetic cases should become
executable fixtures; this document does not claim they ran.

Next A0: remaining device/backend scope, output/workspace naming, unscoped rule
fields and public binding/action addressing, followed by independent reference
fixtures. Animation/gesture source coverage does not close P0 or P13.
