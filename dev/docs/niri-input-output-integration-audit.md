# Pinned niri input, debug scope and output identity

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds 45 indexed review links: six scroll/pinch fields, five output identity fields
and the remaining 34 debug storage/Part fields. Debug rows below include scope
triage, not complete backend decoder/runtime acceptance: Unverified rows remain
open. Workspace naming is cross-referenced without double-counting its already
indexed lifecycle rows. No runtime or config changes are made to driftwm.

## Pinned sources

- [niri-config/src/input.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/input.rs)
- [niri-config/src/debug.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/debug.rs)
- [niri-config/src/output.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/output.rs)
- [niri-config/src/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/workspace.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/backend/tty.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/backend/tty.rs)
- [src/backend/winit.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/backend/winit.rs)
- [src/screencasting/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/screencasting/mod.rs)
- [src/dbus/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/dbus/mod.rs)

## Client scroll and pinch

ScrollFactor has optional base argument FloatOrInt [0,100], and horizontal /
vertical properties FloatOrInt [-100,100]. Missing base means 1; each missing axis
inherits base. Explicit axis zero suppresses that axis; a negative axis reverses
it. These values are not restricted to positive speed multipliers. Touchpad and
Mouse each have an optional scroll-factor child. Touchpad additionally has
pinch-sensitivity FloatOrInt [0,100], absent by default and treated as 1 at use.
InputPart replaces a present whole device struct; an empty replacement resets
its omitted fields to struct defaults, unlike fieldwise KeyboardPart merging.

In pointer-axis dispatch, the source chooses Mouse for Wheel, Touchpad for Finger,
and no device factor for other sources. The focused root shell window contributes
its own scroll factor, default 1. Device axis factors multiply that window factor
before client axis/v120 delivery. v120 is converted back to i32. Earlier bound
scroll handling can consume events before this delivery branch; this is not
proof that every bound compositor action sees the scaled amount.

The forwarded pinch update multiplies input and focused-window sensitivity and
raises event.scale to that product. Delta and rotation are forwarded unchanged.
This is exponentiation of scale, not multiplication of scale or pointer travel.
Zero sensitivity yields scale 1. This client-forwarding branch is distinct from
workspace overview gestures and from driftwm canvas pinch/scroll navigation.
Hardware natural scrolling, axis-source production, gesture grabs and all
window-rule permutations still need integration fixtures.

## Output identity and lookup

Output.name is a required string argument. OutputName stores connector plus
optional make/model/serial metadata; these are identity components, not four
additional output config child nodes. OutputName::matches first compares connector
with eq_ignore_ascii_case. If all metadata is absent, only connector matches.
Otherwise it compares the complete make/model/serial string, separated by single
spaces, with absent components represented as Unknown. There is no glob/wildcard
or substring lookup in this helper. ASCII case folding is not Unicode folding.
Extra separators or a trailing space are not ignored.

Outputs::find/find_mut returns the first configured entry whose name matches;
output_by_name_match returns the first matching connected output. These are
distinct search collections, not a specificity ranking. Formatting for saved
identity prefers make/model/serial when any metadata exists, otherwise connector.
from_ipc_output converts make/model equal to Unknown into None while retaining
serial as provided. Do not assume all constructors normalize Unknown identically.

Debug disable-monitor-names makes TTY known_crtc_name return connector with all
metadata absent, directly affecting matching and preferred identity formatting.
It is therefore layout integration scope, not an unrelated rendering diagnostic.
Driftwm's config lookup uses case-sensitive connector equality followed by a
literal * fallback; state lookup also compares exact output names. This differs
from reference identity matching even though both support named output settings.
Full persistence, output ordering and ambiguous identical-monitor resolution
remain P11/reference-fixture work.

## Workspace names and placement

WorkspaceName accepts strings and checks duplicate configured names with
ASCII-insensitive equality in decoder context; duplicates emit an error and
produce an empty placeholder while decoding continues. That placeholder does
not mean the full config succeeds. There is no explicit nonempty-name check in
this scalar implementation. WorkspaceLayoutPart still rejects monitor-only
empty-workspace-above-first and insert-hint, as recorded in the
[lifecycle audit](niri-config-lifecycle-audit.md).

Workspace open-on-output is an ordered repeated string child, not a single
output property. ensure_named_workspace first avoids creating an already named
workspace. Without preferences it chooses the active monitor. With preferences
it chooses the first preferred name matching a connected monitor, falling back
to primary if none matches; without outputs it stores a no-output workspace.
This creation path does not itself prove every subsequent hotplug migration rule.

Runtime named lookup uses ASCII-insensitive equality. The action reference helper
maps an index through saturating_sub(1) on the current output; names and IDs resolve
to an existing workspace's current output/index. Full public parsing, allowed
index bounds, removal, previous/next and all action call-site behavior remain the
binding/action audit. Reordering must not turn a stable workspace ID into an
ordinal. Driftwm tiling workspaces remain missing; canvas output viewports are
not equivalent. Previous lifecycle reload/removal limits still apply.

## Remaining debug scope

Debug defaults all booleans false, optional preview/render-device absent and
ignored devices empty. Part flags use Flag, so bare means true and explicit false
can override earlier true; omitted fields preserve earlier config parts. Preview
and render-device use optional replacement. Repeated ignore-drm-device paths
append across parts. These are configuration assembly rules, not a guarantee
that reload reapplies every backend setting. The five layout flags already have
an [input focus audit](niri-input-focus-audit.md#layout-debug-flags).

| Remaining field | Located consumer and scope decision |
| --- | --- |
| preview_render | Niri::render changes Output target to Screencast/ScreenCapture before rendering. Missing equivalent debug policy; privacy/effect target acceptance stays in scope. |
| dbus_interfaces_in_non_session_instances | dbus/mod.rs gates interface export outside a session instance. Outside the tiling action/geometry contract; this excludes the export switch only, not workspace/focus state exposed over IPC. |
| wait_for_frame_completion_before_queueing | TTY and winit frame-queue paths. Unverified backend scheduling; rendering/transaction acceptance remains in scope. |
| enable_overlay_planes | TTY DRM compositor frame flags. Unverified scanout path; no offscreen-GPU evidence substitutes for hardware. |
| disable_cursor_plane | TTY DRM frame flags. Unverified cursor/scanout integration. |
| disable_direct_scanout | TTY DRM frame flags. Unverified scanout/render integration. |
| restrict_primary_scanout_to_matching_format | TTY primary scanout flags. Unverified format/scanout integration. |
| force_disable_connectors_on_resume | TTY session resume. Missing equivalent reference switch; topology transitions remain P11 integration. |
| render_drm_device | TTY primary-node selection from configured path. Unverified renderer/topology integration, not excluded wholesale. |
| ignored_drm_devices | Paths resolve primary/render nodes into ignored-node set; reload detects list changes. Missing equivalent reference option; output removal/ownership still in scope. |
| force_pipewire_invalid_modifier | Screencast negotiation fallback/modifier selection. Unverified capture integration; filename/path exclusions do not cover it. |
| disable_pipewire_dmabuf | Screencast format/transport selection. Unverified capture integration. |
| emulate_zero_presentation_time | TTY presentation-time choice. Unverified timing integration; not a layout size option. |
| keep_laptop_panel_on_when_lid_is_closed | TTY only disables laptop panels for closed lid with an external monitor unless this flag is set. Missing reference policy switch; output loss/recovery remains in scope. |
| disable_monitor_names | TTY identity metadata suppression described above. Missing reference switch; name matching remains in scope. |
| skip_cursor_only_updates_during_vrr | TTY DRM compositor frame flags. Unverified VRR/frame scheduling integration. |
| disable_10bit_output | TTY color-format selection. Unverified rendering/format integration. |

These labels locate concrete consumers without claiming full backend parity or
silently excluding mixed debug surfaces. Revisit Unverified rows against the
reference acceptance scope and hardware/capture evidence, not their debug label.

## Remaining acceptance and continuation

Executable probes are still required for base/axis precedence and negative/zero
factors, source selection, bound-event consumption, v120 conversion, focused-rule
multiplication and pinch exponentiation; connector versus metadata case/spacing,
missing components, first-match ambiguity and identity persistence; duplicate,
empty and non-ASCII workspace names, preference fallbacks, no outputs, reload and
hotplug; and each retained debug backend/capture/topology path. The source audit
does not claim these fixtures ran or implement any Missing setting.

Next A0: remaining keyboard/device input grammar and consumers, unresolved debug
acceptance scope, unscoped output/rule fields and public binding/action addressing.
Independent reference fixtures beyond zero gaps remain mandatory before feature
expansion; this review does not close A0/P0 or multi-output P11.
