# Pinned niri keyboard and device input

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source audit
covers the remaining 89 named input fields. With the eight focus/warp/workspace
and six scroll/pinch fields already reviewed, all 103 named input fields now
have source-review links. This does not prove runtime equivalence: device rows
remain Unverified integration, not automatic exclusions or implemented parity.
No niri/parser/libinput execution or driftwm runtime change is claimed.

## Pinned sources

- [niri-config/src/input.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/input.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/backend/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/backend/mod.rs)

## Assembly and scalar grammar

Input defaults device structs and Keyboard; optional compositor Mod keys are
absent and disable-power-key-handling is false. InputPart merges KeyboardPart
by field. Present touchpad/mouse/trackpoint/trackball/tablet/touch nodes replace
their whole device value; omission preserves an earlier part. An empty device
node therefore resets its settings to struct defaults. Xkb is likewise replaced
as a whole when present inside KeyboardPart; its individual strings do not merge.
Flags in device structs are child flags, not arbitrary boolean arguments.
The explicit exceptions drag and KeyboardPart numlock have different shapes below.

Device boolean fields default false; accel-speed defaults zero; optional settings
are absent. Accel-speed accepts FloatOrInt [-1,1], scroll-button is optional u32,
drag is an optional boolean argument. Calibration-matrix is an optional list of
f32 arguments; map-to-output is an optional string. Previously audited scroll
factors and pinch sensitivity retain their
[delivery semantics](niri-input-output-integration-audit.md#client-scroll-and-pinch).

| Value | Accepted spellings |
| --- | --- |
| ClickMethod | clickfinger, button-areas |
| AccelProfile | adaptive, flat |
| ScrollMethod | no-scroll, two-finger, edge, on-button-down |
| TapButtonMap | left-right-middle, left-middle-right |
| TrackLayout | global, window (derived scalar enum) |
| ModKey | ctrl/control, shift, alt, super/win, iso_level3_shift/mod5, iso_level5_shift/mod3 |

The four device FromStr enums are case-sensitive. ModKey lowercases ASCII before
matching. Do not assume arbitrary XKB modifier names or case-insensitivity for
every enum. Actual parser diagnostics still need executable fixtures.

## Keyboard configuration and consumers

Keyboard defaults repeat-delay=600 ms, repeat-rate=25, track-layout=global and
numlock=false. Repeat decoder types are u16 delay and u8 rate. KeyboardPart numlock
uses Flag: bare true, explicit false allowed. Xkb rules/model/layout/variant
default to empty strings; options and file are absent. Xkb::to_xkb_config forwards
those five conventional fields; file has a separate consumer.

Seat setup first tries the configured XKB settings, warning/falling back to a
default keymap on failure. numlock=true sets initial modifier state. This startup
path is not evidence that toggling the option on reload resets current Num Lock.
Changed repeat settings call change_repeat_info. Compositor binding repeat reads
current keyboard settings; rate zero suppresses its repeat timer. Client repeat
metadata and compositor action repetition are separate consumers.

Changed Xkb triggers keymap reload. load_configured_keymap tries an expanded-home
file first; successful file loading avoids conventional XKB construction. On
failure it warns and falls back to conventional settings. With all conventional
fields unset it can use locale1 settings. Replacing a seat keymap restores saved
keyboard state: a virtual-keyboard transition has its own saved state, otherwise
Num Lock is carried over. Do not equate failed file load with a successful new
keymap or assume every startup/reload/virtual-keyboard permutation is covered.

In track-layout=window, keyboard focus change saves the old surface's active
layout and restores the new surface's remembered layout; a new surface starts
with the first layout. If changing layout for a surface, code clears keyboard
focus before switching XKB state, then sets the new focus. This is surface-local
state, not simply a single global layout label. Popup/layer/lock/virtual-keyboard
interactions require focused protocol fixtures.

Backend Mod selection is explicit: TTY/headless uses configured mod-key or Super.
Winit uses mod-key-nested when present; otherwise it selects Super if the main
Mod is Alt, and Alt otherwise. disable-power-key-handling prevents the built-in
XF86PowerOff-to-Suspend fallback; it does not disable parsing user bindings or
all power/session actions. Public binding resolution remains a separate audit.

## Libinput application and classification

apply_libinput_settings identifies a touchpad by tap finger count >0. A mouse
requires Pointer capability and excludes touchpad/trackball/trackpoint; trackball
and pointing-stick identification uses udev property presence. The latter device
branches are independent checks, not a universal mutually exclusive class enum.
TabletTool and Touch capabilities independently select their calibration branches.
Settings application ignores libinput setter return values, so a decoded option
is not proof that a particular device supports or accepted it.

| Struct | Fields traced to device configuration |
| --- | --- |
| Touchpad | off, tap, dwt, dwtp, drag, drag_lock, natural_scroll, click_method, accel_speed, accel_profile, scroll_method, scroll_button, scroll_button_lock, tap_button_map, left_handed, disabled_on_external_mouse, middle_emulation |
| Mouse | off, natural_scroll, accel_speed, accel_profile, scroll_method, scroll_button, scroll_button_lock, left_handed, middle_emulation |
| Trackpoint / Trackball | off, natural_scroll, accel_speed, accel_profile, scroll_method, scroll_button, scroll_button_lock, left_handed, middle_emulation |
| Tablet | off, calibration_matrix, left_handed; mapping flags/target use compositor policy below |
| Touch | off, calibration_matrix; mapping target uses compositor policy below |

Touchpad off OR the runtime touchpad-toggle state wins over
disabled-on-external-mouse, then ordinary enabled mode. tap, dwt/dwtp, natural
scroll, handedness, middle emulation and speed are sent directly. drag-lock maps
to EnabledTimeout or Disabled. Absent drag restores the device default, rather
than forcing false. Absent acceleration profile, click method and tap button map
try their available device defaults. The mouse/trackpoint/trackball branches
apply their corresponding options and fallback acceleration/scroll methods.

Scroll-button and button lock are applied when the selected or default scroll
method is OnButtonDown. A configured button alone does not force that method.
An omitted button leaves the setter untouched in this branch; do not claim all
omissions reset hardware state. Calibration tries exactly six configured values,
then the device default matrix, then identity. An arbitrary-length list can pass
the schema but fall back here; this is runtime policy, not a six-argument parser
restriction. Tablet left-handed is a separate setter.

New devices receive settings. Reload compares all six device structs and reapplies
settings to current devices when changed, preserving the runtime touchpad toggle.
Device removal/re-addition, unsupported capabilities and setter failures remain
hardware/integration acceptance; no device field is excluded merely because it
is configured through libinput.

## Tablet and touch target policy

output_for_tablet gives focused-output mapping priority over configured output
name; otherwise it resolves map-to-output with the
[reference identity helper](niri-input-output-integration-audit.md#output-identity-and-lookup).
compute_tablet_position first prefers a still-live device-provided output over
that fallback. map-to-focused-window only applies while keyboard focus belongs
to the layout, using the active window's visual rectangle on the mapped or active
output. Overview can suppress that rectangle. Otherwise mapped-output geometry
is used; without a mapping the global bounding rectangle is used.

Window/output targeting preserves aspect ratio and considers transform and
fractional output scale; unified global mapping deliberately does not preserve
aspect ratio. No-output and exact boundary/clamp behavior need fixtures rather
than extrapolation from one helper. Touch's output helper resolves configured
name then falls back to the first registered output. Tablet and touch therefore
do not share a single universal mapping/fallback rule.

## Driftwm differences and remaining acceptance

Driftwm has keyboard/XKB/repeat and libinput configuration, but a similarly named
option does not establish default, grammar, reload or supported-device parity.
For example its repeat delay defaults to 200 ms versus reference 600; repeat rate
25 agrees but public numeric types differ. Per-surface track-layout, backend Mod
selection, device defaults, hardware classification and tablet target behavior
need independent comparisons. Existing canvas input behavior must stay intact.

Required fixtures include ordered empty/partial keyboard/device parts; numeric
bounds and enum errors; first-layout and per-surface focus switches; repeat zero
and reload; keymap file failure, locale1 and virtual-keyboard restoration; nested
Mod selection and power fallback; unsupported setter/device default behavior;
OnButtonDown with absent/configured button; six-value versus invalid-length
calibration; device hotplug/toggle/reload; target precedence, dead outputs,
nonuniform topology/scales/transforms and focused-window/overview changes.
These are requirements, not reported test results.

Next A0: unscoped output and window-rule fields, public binding/action addressing
and independent executable reference fixtures. Retained backend/capture and
input integration acceptance remains open; source coverage does not close P0.
