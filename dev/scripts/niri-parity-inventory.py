#!/usr/bin/env python3
"""Index the pinned reference schema; manual tool, never fetches upstream."""

import argparse
from pathlib import Path
import re
import subprocess

REVISION = "12de51d6a60baad8896366920278a24f17921f52"


def block(source, declaration):
    start = source.index(declaration)
    end = source.index("\n}", start)
    return source[start:end]


def action_status(name):
    partial = {
        "FocusColumnLeft": "focus-column left",
        "FocusColumnRight": "focus-column right",
        "FocusWindowUp": "focus-column up",
        "FocusWindowDown": "focus-column down",
        "MoveColumnLeft": "move-column left",
        "MoveColumnRight": "move-column right",
        "MoveWindowUp": "move-tiled-window up",
        "MoveWindowDown": "move-tiled-window down",
        "SwitchPresetColumnWidth": "cycle-column-width (different presets/units)",
        "CloseWindow": "close-window",
        "ConsumeWindowIntoColumn": "consume-window-into-column (topology/focus subset)",
        "ExpelWindowFromColumn": "expel-window-from-column (topology/focus subset)",
    }
    if name in partial:
        return "Layout", "Partial", partial[name], "src/tests/tiling.rs; src/stage/tiling.rs tests"
    if name in {"FullscreenWindow", "ToggleWindowedFullscreen"}:
        return "Layout", "Divergent", "toggle-fullscreen exits tiling; topology not retained", "incompatible_actions_leave_tiling_before_applying_canvas_policy"
    if name.startswith(("Focus", "Move", "Consume", "Expel", "Swap", "Center", "SetWindow", "SetColumn", "SwitchPreset", "ResetWindow", "Maximize")) or name in {
        "CloseWindowById", "FullscreenWindowById", "ToggleWindowedFullscreenById",
        "ToggleColumnTabbedDisplay", "SetColumnDisplay", "ExpandColumnToAvailableWidth",
        "ToggleWindowFloating", "ToggleWindowFloatingById", "SwitchFocusBetweenFloatingAndTiling",
        "ToggleOverview", "OpenOverview", "CloseOverview", "SetWorkspaceName", "SetWorkspaceNameByRef",
        "UnsetWorkspaceName", "UnsetWorkSpaceNameByRef",
    }:
        return "Layout", "Missing", "—", "P4/P6–P14; no parity evidence yet"
    if "Urgent" in name or name.startswith("Mru"):
        return "Integration", "Unverified", "existing focus/window state is not a parity test", "P8/P10/P14; behavior audit pending"
    return "Outside", "Outside", "unrelated compositor action", "roadmap scope; reconsider if layout-visible"


LAYOUT_AUDIT = {
    "focus_ring": ("Missing", "Separate focus-ring facility missing"),
    "border": ("Partial", "Native decoration border differs in grammar/defaults/placement"),
    "shadow": ("Partial", "Native shadow exists; reference settings/defaults not equivalent"),
    "tab_indicator": ("Missing", "Tabbed columns and indicator missing"),
    "insert_hint": ("Missing", "Layout-native insertion hint missing"),
    "preset_column_widths": ("Divergent", "Hardcoded 1/3, 1/2, full vs reference 1/3, 1/2, 2/3; units/empty-list reset missing"),
    "default_column_width": ("Partial", "Default 0.5 agrees; public setting and omitted/empty/client-choice semantics missing"),
    "preset_window_heights": ("Missing", "Height presets and cycling missing"),
    "center_focused_column": ("Partial", "Reveal/fit subset; no public never/always/on-overflow policy"),
    "always_center_single_column": ("Missing", "Dedicated setting/policy missing"),
    "empty_workspace_above_first": ("Missing", "Workspace lifecycle missing; workspace override disallows this node"),
    "default_column_display": ("Missing", "Public normal/tabbed setting and tabbed display missing"),
    "gaps": ("Divergent", "Shared snap.gap defaults to 12 vs 16, rounds to integer; gap accounting differs"),
    "struts": ("Missing", "Signed/fractional custom struts missing; layer usable area is not equivalent"),
    "background_color": ("Partial", "Canvas background differs from per-layout solid default; Color grammar audited separately"),
}


APPEARANCE_AUDIT_FIELDS = {
    "Color": "r g b a",
    "Gradient": "from to angle relative_to in_",
    "GradientInterpolation": "color_space hue_interpolation",
    "FocusRing": "off width active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
    "Border": "off width active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
    "Shadow": "on offset softness spread draw_behind_window color inactive_color",
    "ShadowOffset": "x y",
    "TabIndicator": "off hide_when_single_tab place_within_column gap width length position gaps_between_tabs corner_radius active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
    "TabIndicatorPart": "off on hide_when_single_tab place_within_column gap width length position gaps_between_tabs corner_radius active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
    "TabIndicatorLength": "total_proportion",
    "InsertHint": "off color gradient",
    "InsertHintPart": "off on color gradient",
    "BorderRule": "off on width active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
    "ShadowRule": "off on offset softness spread draw_behind_window color inactive_color",
    "TabIndicatorRule": "active_color inactive_color urgent_color active_gradient inactive_gradient urgent_gradient",
}


def appearance_status(name, field):
    if name == "Color":
        status, mapping, section = "Partial", "RGBA exists; internal float components are not public keys, CSS/numeric KDL grammar differs", "color-and-gradient-decoding"
    elif name in {"Gradient", "GradientInterpolation"}:
        status, mapping, section = "Missing", "No gradient/interpolation configuration", "color-and-gradient-decoding"
    elif name in {"Border", "BorderRule", "FocusRing"}:
        status = "Partial" if name != "FocusRing" and field in {"off", "on", "width", "active_color", "inactive_color"} else "Missing"
        mapping, section = "Native borders cover a subset; separate focus ring, urgent colors and gradients missing", "border-and-focus-ring-rules"
    elif name in {"Shadow", "ShadowRule", "ShadowOffset"}:
        status = "Partial" if name != "ShadowOffset" and field in {"on", "off"} else "Missing"
        mapping, section = "Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ", "shadow-rules"
    else:
        status, mapping, section = "Missing", "No tabbed indicator or insertion-hint implementation", "tab-indicator-and-insertion-hint"
    return status, f"{mapping}; [decoder/merge audit](niri-appearance-config-audit.md#{section})"


LIFECYCLE_WINDOW_FIELDS = {
    "matches": ("Partial", "Ordered app/title matching exists; niri OR match blocks and dynamic states differ"),
    "excludes": ("Missing", "No niri exclude blocks"),
    "default_column_width": ("Missing", "No rule-level tiled default width with omitted/empty/present semantics"),
    "default_window_height": ("Missing", "No rule-level tiled default height"),
    "open_on_output": ("Partial", "Canvas output placement exists; niri target precedence/owner lifecycle differs"),
    "open_on_workspace": ("Missing", "No workspace routing"),
    "open_maximized": ("Missing", "No reference full-width column opening policy"),
    "open_maximized_to_edges": ("Missing", "No reference maximized-to-edges opening policy"),
    "open_fullscreen": ("Divergent", "Fullscreen rules use canvas transitions, not retained tiling topology"),
    "open_floating": ("Missing", "No layout-native floating opening policy"),
    "open_focused": ("Partial", "focus_on_open exists; token/smart/strict reference policy not equivalent"),
    "default_column_display": ("Missing", "No normal/tabbed column default rule"),
    "default_floating_position": ("Missing", "Generic canvas position is not reference floating initial placement"),
    "min_width": ("Missing", "Client constraints exist; rule-level extra constraint missing"),
    "min_height": ("Missing", "Client constraints exist; rule-level extra constraint missing"),
    "max_width": ("Missing", "Client constraints exist; rule-level extra constraint missing"),
    "max_height": ("Missing", "Client constraints exist; rule-level extra constraint missing"),
    "focus_ring": ("Missing", "No separate reference focus ring"),
    "border": ("Divergent", "Native border rules are stored at mapping, not dynamically recomputed"),
    "shadow": ("Divergent", "Native shadow rule flags are stored at mapping; reference merges/reload differ"),
    "tab_indicator": ("Missing", "No per-window tab-indicator rule"),
    "opacity": ("Divergent", "Mapped appearance rule is sticky; reference recomputes metadata/state/reload matches"),
}
MATCH_AUDIT_FIELDS = "app_id title is_active is_focused is_active_in_column is_floating is_window_cast_target is_urgent at_startup"


EFFECT_AUDIT_FIELDS = {
    "CornerRadius": "top_left top_right bottom_right bottom_left",
    "WorkspaceShadow": "off offset softness spread color",
    "WorkspaceShadowPart": "off on offset softness spread color",
    "Blur": "off passes offset noise saturation",
    "BlurPart": "off on passes offset noise saturation",
    "BackgroundEffectRule": "xray blur noise saturation",
    "BackgroundEffect": "xray blur noise saturation",
}
EFFECT_CONSUMER_FIELDS = {
    "WindowRule": "geometry_corner_radius clip_to_geometry background_effect",
    "PopupsRule": "geometry_corner_radius background_effect",
    "ResolvedPopupsRules": "geometry_corner_radius background_effect",
}


def effect_status(name, field):
    if name == "CornerRadius":
        status, mapping, section = "Partial", "Scalar integer clipping exists; four-corner/fractional grammar and geometry policy differ", "corner-radius-decoding-and-consumers"
    elif name in {"WorkspaceShadow", "WorkspaceShadowPart"}:
        status, mapping, section = "Missing", "No overview workspace shadow; reference normalizes by logical view height", "workspace-shadows"
    elif name in {"Blur", "BlurPart"}:
        status = "Missing" if field in {"noise", "saturation"} else "Partial"
        mapping, section = "Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ", "blur-tuning-and-background-effect-rules"
    else:
        status = "Divergent" if field == "blur" else "Missing"
        mapping, section = "Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost", "client-regions-popup-rules-and-policy-differences"
    return status, f"{mapping}; [radius/effect audit](niri-effects-config-audit.md#{section})"


SCHEMA_MODULES = ("lib", "layout", "appearance", "gestures", "animations", "window_rule", "workspace", "output", "misc", "layer_rule", "input", "debug", "binds", "recent_windows")
MODULE_COVERAGE = {
    "lib.rs": ("Mixed/root", "Config fields plus custom root-node dispatch; internal helpers are not public keys"),
    "layout.rs": ("Layout", "Column geometry, defaults and partial layout overrides"),
    "appearance.rs": ("Layout/integration", "Decoration, geometry, blur and effect values"),
    "gestures.rs": ("Layout/integration", "View scrolling, workspace switching and hot corners"),
    "animations.rs": ("Layout", "Movement, resize and workspace/overview transitions"),
    "window_rule.rs": ("Layout/integration", "Window matching, opening, sizing and dynamic appearance"),
    "workspace.rs": ("Layout", "Named workspace routing and layout overrides"),
    "output.rs": ("Layout/integration", "Output owners, modes/scales and layout overrides"),
    "misc.rs": ("Mixed", "Includes Overview/OverviewPart; unrelated process/UI settings are explicitly excluded by root scope"),
    "layer_rule.rs": ("Integration", "Layer matching, placement/backdrop and effects; full semantic audit pending"),
    "input.rs": ("Mixed/integration", "Focus, warping, workspace policy and modifiers coexist with device settings; no blanket exclusion"),
    "debug.rs": ("Mixed/integration", "Transaction, resize/focus and output toggles coexist with backend flags; no blanket exclusion"),
    "binds.rs": ("Layout/integration", "Public bindings/actions and switch events; aliases/units/addressing need audit"),
    "recent_windows.rs": ("Layout/integration", "MRU focus overlay and previews; distinct from unrelated help UI"),
    "error.rs": ("Infrastructure", "Diagnostic/result payloads, not configurable option schemas; named fields intentionally not indexed"),
    "macros.rs": ("Infrastructure", "Merge helpers, audited through their consumer schemas; no named option fields"),
    "utils.rs": ("Infrastructure", "Custom scalar/wrapper decoders, audited through consumers; declarations are not standalone option trees"),
    "utils/merge_with.rs": ("Infrastructure", "Merge trait implementation helper; no configurable option fields"),
}
ROOT_NODES = {
    "input": ("input", "Integration", "Unverified", "Includes focus/warping/workspace policy; audit per field"),
    "cursor": ("cursor", "Integration", "Unverified", "Cursor interaction/visibility boundaries not audited"),
    "clipboard": ("clipboard", "Outside", "Outside", "Selection policy, not tiling topology or sizing"),
    "hotkey-overlay": ("hotkey_overlay", "Outside", "Outside", "Shortcut help UI, not workspace overview or MRU focus"),
    "config-notification": ("config_notification", "Outside", "Outside", "Config-error notification presentation, not layout policy"),
    "animations": ("animations", "Layout", "Divergent", "Frame factors differ from per-event spring/easing; scoped source audit recorded"),
    "blur": ("blur", "Layout/integration", "Partial", "Tuning/effect-selection source audit exists; runtime parity not established"),
    "gestures": ("gestures", "Layout/integration", "Missing", "Reference DnD policy missing; hot-corner behavior differs; scoped source audit recorded"),
    "overview": ("overview", "Layout", "Missing", "Overview exists in pinned schema; no reference overview implementation"),
    "xwayland-satellite": ("xwayland_satellite", "Integration", "Unverified", "Client/protocol integration; reconsider layout-visible lifecycle effects"),
    "switch-events": ("switch_events", "Integration", "Unverified", "Lid/output actions may affect layout lifecycle"),
    "debug": ("debug", "Integration", "Unverified", "Mixed layout-visible transaction/focus flags and backend controls"),
    "output": ("outputs", "Layout/integration", "Partial", "Repeated output configs; ownership/layout/name/mode surface not fully audited"),
    "spawn-at-startup": ("spawn_at_startup", "Outside", "Outside", "Process launch list; resulting window map remains in layout scope"),
    "spawn-sh-at-startup": ("spawn_sh_at_startup", "Outside", "Outside", "Shell process launch; not a tiling action contract"),
    "window-rule": ("window_rules", "Layout/integration", "Partial", "Scoped decoder/lifecycle audits exist; remaining fields and fixtures pending"),
    "layer-rule": ("layer_rules", "Integration", "Missing", "Dedicated rule tree missing; scoped decoder/consumer audit recorded"),
    "workspace": ("workspaces", "Layout", "Missing", "Named workspace/layout hierarchy not implemented"),
    "binds": ("binds", "Layout/integration", "Unverified", "Action inventory is not a public binding alias/decoder audit"),
    "environment": ("environment", "Outside", "Outside", "Child-process environment, not layout behavior"),
    "prefer-no-csd": ("prefer_no_csd", "Integration", "Unverified", "Decoration negotiation/tiled-state behavior can affect layout"),
    "screenshot-path": ("screenshot_path", "Outside", "Outside", "Destination filename policy; capture/effect behavior remains separately in scope"),
    "layout": ("layout", "Layout", "Partial", "Top-level names/defaults audited; full option behavior/fixtures remain partial"),
    "recent-windows": ("recent_windows", "Layout/integration", "Missing", "MRU focus overlay/preview configuration has no parity implementation"),
    "include": (None, "Infrastructure", "Unverified", "Custom positional decoder directive with no Config field; full path/error behavior pending"),
}


def schema_structs(source):
    for match in re.finditer(r"^pub struct (\w+)[^;\n{]*\{", source, re.M):
        yield match.group(1), re.findall(r"^    pub (\w+):", block(source, match.group(0)), re.M)


ANIMATION_GESTURE_AUDIT_FIELDS = {
    ("animations", "Animations"): "off slowdown workspace_switch window_open window_close horizontal_view_movement window_movement window_resize config_notification_open_close exit_confirmation_open_close screenshot_ui_open overview_open_close recent_windows_close",
    ("animations", "AnimationsPart"): "off on slowdown workspace_switch window_open window_close horizontal_view_movement window_movement window_resize config_notification_open_close exit_confirmation_open_close screenshot_ui_open overview_open_close recent_windows_close",
    ("animations", "Animation"): "off kind",
    ("animations", "EasingParams"): "duration_ms curve",
    ("animations", "SpringParams"): "damping_ratio stiffness epsilon",
    ("animations", "WindowOpenAnim"): "anim custom_shader",
    ("animations", "WindowCloseAnim"): "anim custom_shader",
    ("animations", "WindowResizeAnim"): "anim custom_shader",
    ("gestures", "Gestures"): "dnd_edge_view_scroll dnd_edge_workspace_switch hot_corners",
    ("gestures", "GesturesPart"): "dnd_edge_view_scroll dnd_edge_workspace_switch hot_corners",
    ("gestures", "DndEdgeViewScroll"): "trigger_width delay_ms max_speed",
    ("gestures", "DndEdgeViewScrollPart"): "trigger_width delay_ms max_speed",
    ("gestures", "DndEdgeWorkspaceSwitch"): "trigger_height delay_ms max_speed",
    ("gestures", "DndEdgeWorkspaceSwitchPart"): "trigger_height delay_ms max_speed",
    ("gestures", "HotCorners"): "off top_left top_right bottom_left bottom_right",
}


def animation_gesture_status(module, name, field):
    if module == "animations":
        section = "animation-defaults-and-merge" if name.startswith("Animations") else "custom-animation-decoding"
        if field in {"config_notification_open_close", "exit_confirmation_open_close", "screenshot_ui_open"}:
            status, mapping = "Outside", "Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited"
        elif field == "custom_shader":
            status, mapping = "Missing", "Reference open/close/resize shader API missing"
        else:
            status, mapping = "Divergent", "Frame factors/retention differ from per-event spring/easing and shared clock"
    else:
        section = "hot-corner-consumers" if name == "HotCorners" or field == "hot_corners" else "dnd-edge-consumers"
        if section == "hot-corner-consumers":
            status, mapping = "Divergent", "Configurable actions/threshold differ from one-pixel overview toggle"
        else:
            status, mapping = "Missing", "Dedicated reference DnD edge view/workspace policy missing"
    return status, f"{mapping}; [animation/gesture audit](niri-animation-gesture-audit.md#{section})"


INPUT_FOCUS_AUDIT_FIELDS = {
    ("input", "Input"): "warp_mouse_to_focus focus_follows_mouse workspace_auto_back_and_forth",
    ("input", "InputPart"): "warp_mouse_to_focus focus_follows_mouse workspace_auto_back_and_forth",
    ("input", "FocusFollowsMouse"): "max_scroll_amount",
    ("input", "WarpMouseToFocus"): "mode",
    ("debug", "Debug"): "disable_resize_throttling disable_transactions strict_new_window_focus_policy honor_xdg_activation_with_invalid_serial deactivate_unfocused_windows",
    ("debug", "DebugPart"): "disable_resize_throttling disable_transactions strict_new_window_focus_policy honor_xdg_activation_with_invalid_serial deactivate_unfocused_windows",
}


def input_focus_status(name, field):
    section = {
        "warp_mouse_to_focus": "cursor-warping", "mode": "cursor-warping",
        "focus_follows_mouse": "focus-follows-mouse", "max_scroll_amount": "focus-follows-mouse",
        "workspace_auto_back_and_forth": "workspace-back-and-forth",
    }.get(field, "layout-debug-flags")
    status = "Divergent" if field == "focus_follows_mouse" else "Missing"
    mapping = "Boolean hover policy lacks reference threshold/transition semantics" if status == "Divergent" else "Dedicated reference policy/setting missing; existing integration is not equivalence"
    return status, f"{mapping}; [input focus audit](niri-input-focus-audit.md#{section})"


LAYER_OVERVIEW_AUDIT_FIELDS = {
    ("layer_rule", "LayerRule"): "matches excludes opacity block_out_from shadow geometry_corner_radius place_within_backdrop baba_is_float background_effect popups",
    ("layer_rule", "Match"): "namespace at_startup layer",
    ("misc", "Overview"): "zoom backdrop_color workspace_shadow",
    ("misc", "OverviewPart"): "zoom backdrop_color workspace_shadow",
}


def layer_overview_status(name, field):
    if name in {"Overview", "OverviewPart"}:
        status, mapping, section = "Missing", "No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent", "overview-defaults-runtime-zoom-and-state"
    elif name == "Match":
        status = "Divergent" if field == "namespace" else "Missing"
        mapping, section = "Generic namespace-as-app-id patterns lack reference regex/startup/current-layer predicate semantics", "layer-matching-defaults-and-local-merge"
    else:
        status = {
            "matches": "Partial", "excludes": "Missing", "opacity": "Partial",
            "block_out_from": "Missing", "shadow": "Partial",
            "geometry_corner_radius": "Divergent", "place_within_backdrop": "Missing",
            "baba_is_float": "Missing", "background_effect": "Divergent", "popups": "Missing",
        }[field]
        mapping, section = "Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs", "layer-matching-defaults-and-local-merge"
    return status, f"{mapping}; [layer/overview audit](niri-layer-overview-audit.md#{section})"


def config_status(module, name, field):
    if (module, name) in ANIMATION_GESTURE_AUDIT_FIELDS:
        return animation_gesture_status(module, name, field)
    if field in INPUT_FOCUS_AUDIT_FIELDS.get((module, name), "").split():
        return input_focus_status(name, field)
    if (module, name) in LAYER_OVERVIEW_AUDIT_FIELDS:
        return layer_overview_status(name, field)
    coverage_link = "[schema coverage](niri-config-schema-coverage.md)"
    if module == "lib" and name == "Config":
        node, (_, _, status, reason) = next((node, spec) for node, spec in ROOT_NODES.items() if spec[0] == field)
        return status, f"Root `{node}`: {reason}; {coverage_link}"
    if module == "misc" and name in {"SpawnAtStartup", "SpawnShAtStartup", "Clipboard", "ClipboardPart", "HotkeyOverlay", "HotkeyOverlayPart", "ConfigNotification", "ConfigNotificationPart", "EnvironmentVariable"}:
        return "Outside", f"Process/selection/help/error-UI setting; explicit root-scope exclusion; {coverage_link}"
    if module in {"misc", "layer_rule", "input", "debug", "binds", "recent_windows"}:
        return "Unverified", f"Newly indexed surface; public decoder/default/consumer audit pending; {coverage_link}"
    if module == "appearance" and name in EFFECT_AUDIT_FIELDS:
        return effect_status(name, field)
    if module == "window_rule" and field in EFFECT_CONSUMER_FIELDS.get(name, "").split():
        status = "Divergent" if name == "WindowRule" and field != "clip_to_geometry" else "Missing"
        return status, "No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md)"
    lifecycle_link = "[precedence/lifecycle audit](niri-config-lifecycle-audit.md)"
    if module == "window_rule" and name == "WindowRule" and field in LIFECYCLE_WINDOW_FIELDS:
        status, mapping = LIFECYCLE_WINDOW_FIELDS[field]
        return status, f"{mapping}; {lifecycle_link}"
    if module == "window_rule" and name == "Match":
        status = "Divergent" if field in {"app_id", "title"} else "Missing"
        return status, f"Regex/dynamic match properties differ from mapped app/title glob rules; {lifecycle_link}"
    if (module, name, field) in {("output", "Output", "layout"), ("workspace", "Workspace", "layout")}:
        return "Missing", f"No per-output/workspace tiling layout hierarchy; {lifecycle_link}"
    if module == "appearance" and name in APPEARANCE_AUDIT_FIELDS:
        return appearance_status(name, field)
    if module == "layout" and name in {"Layout", "LayoutPart"}:
        status, mapping = LAYOUT_AUDIT[field]
        return status, f"{mapping}; [public name/default audit](niri-layout-config-audit.md#public-layout-children)"
    if module == "layout" and name == "Struts":
        return "Missing", "No custom struts; [signed numeric/default audit](niri-layout-config-audit.md#size-decoders)"
    if module == "window_rule" and field in {"default_column_width", "default_window_height", "default_column_display", "open_floating"}:
        return "Missing", "P3/P8/P10 lifecycle-aware tiling window rules"
    return "Unverified", "Inspect decoder and reference behavior; no verified option mapping"


def generate(root):
    revision = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        raise SystemExit(f"Expected {REVISION}, found {revision}; do not silently change the baseline")
    source_root = root / "niri-config/src"
    paths = {str(path.relative_to(source_root)) for path in source_root.rglob("*.rs")}
    assert paths == set(MODULE_COVERAGE), "Every pinned source file needs a coverage/exclusion entry"
    sources = {path: (source_root / path).read_text() for path in paths}
    root_nodes = re.findall(r'^                "([\w-]+)" =>', block(sources["lib.rs"], "impl<S> knuffel::DecodeChildren<S> for ConfigPart"), re.M)
    assert len(root_nodes) == len(ROOT_NODES) and set(root_nodes) == set(ROOT_NODES), "Every root decoder node needs scope evidence"
    root_fields = dict(schema_structs(sources["lib.rs"]))["Config"]
    assert set(root_fields) == {spec[0] for spec in ROOT_NODES.values() if spec[0] is not None}, "Root node aliases must cover Config fields"
    actions = block((source_root / "binds.rs").read_text(), "pub enum Action {")
    names = re.findall(r"^    ([A-Z]\w*)\b", actions, re.M)
    assert names and len(set(names)) == len(names)
    lines = [
        "# Niri tiling parity inventory", "",
        f"Reference: `{REVISION}`. Rebuild with `python3 dev/scripts/niri-parity-inventory.py /tmp/niri > dev/docs/niri-tiling-inventory.md`.", "",
        "This is a source-symbol inventory, not a claim that semantic parity is tested.",
        "All Action variants are included to make exclusions visible; `ById` and internal",
        "variants remain separate rows. Names are Rust symbols, not guessed binding aliases.",
        "The reference has decoder attributes/custom parsers: public schema fields below",
        "are not automatically public keys. The [layout audit](niri-layout-config-audit.md)",
        "covers top-level layout names/defaults and shared size/flag/strut decoding;",
        "the [appearance audit](niri-appearance-config-audit.md) covers color/gradient,",
        "border/shadow and tab/insert-hint decoding and merges. The",
        "[lifecycle audit](niri-config-lifecycle-audit.md) traces override order and",
        "scoped window-rule initial-map/reload consumers. The [effects audit](niri-effects-config-audit.md)",
        "adds radius/workspace-shadow/blur/background-effect decoding and consumers.",
        "The [schema coverage audit](niri-config-schema-coverage.md) accounts for all",
        "18 source files, 25 root nodes and 14 indexed modules. Fields are source",
        "inventory, not a completed public decoder/consumer audit. Unscoped behavior stays open.",
        "The [layer/overview consumers](niri-layer-overview-audit.md) audit traces newly indexed",
        "layer rules and overview value consumers; complete interactions remain open.",
        "The [input focus consumers](niri-input-focus-audit.md) audit traces scoped focus, warp,",
        "workspace back-and-forth and five layout-visible debug flags; other fields stay open.",
        "The [animation/gesture consumers](niri-animation-gesture-audit.md) audit traces those",
        "schema fields and scoped consumers; physics/interaction execution remains open.",
        "P0 reference operation fixtures and the custom-decoder audit remain unfinished.", "",
        "Status: Partial = related implementation, not full equivalence; Divergent = known",
        "semantic difference; Missing = no matching tiling action; Unverified = not audited;",
        "Outside = explicit non-layout scope exclusion. Never infer Exact from a similar name.", "",
        "## Reference actions", "",
        "| Reference Action | Scope | Status | driftwm mapping / gap | Evidence / remaining gate |",
        "| --- | --- | --- | --- | --- |",
    ]
    for name in names:
        scope, status, mapping, evidence = action_status(name)
        lines.append(f"| `{name}` | {scope} | {status} | {mapping} | {evidence} |")
    lines += ["", f"Action variants indexed: **{len(names)}**.", "", "## Module coverage", "",
        "| Source file | Scope | Index | Named fields | Coverage / exclusion reason |",
        "| --- | --- | --- | --- | --- |"]
    for path, (scope, reason) in MODULE_COVERAGE.items():
        indexed = path.removesuffix(".rs") in SCHEMA_MODULES
        field_count = sum(len(fields) for _, fields in schema_structs(sources[path]))
        lines.append(f"| `{path}` | {scope} | {'Indexed' if indexed else 'Infrastructure only'} | {field_count} | {reason} |")
    lines += ["", "## Public root decoder nodes", "",
        "`include` is a custom decoder directive, not a missing Config field. Singular",
        "public node names such as `output` and `window-rule` are not plural Rust fields.", "",
        "| Public root node | Config field | Scope | Status | Evidence / next audit |",
        "| --- | --- | --- | --- | --- |"]
    for node in root_nodes:
        field, scope, status, reason = ROOT_NODES[node]
        storage = f"`Config.{field}`" if field else "—"
        lines.append(f"| `{node}` | {storage} | {scope} | {status} | {reason}; [coverage](niri-config-schema-coverage.md) |")
    lines += ["", "## Configuration source index", "",
        "Includes complete named public-struct field sets in these reference modules,",
        "including partial/override schemas and non-layout fields. A row is not necessarily",
        "a directly bindable configuration key. Shared nested types are listed at their",
        "own definitions; tuple wrappers and enums are indexed by declaration below.", ""]
    count = 0
    for module in SCHEMA_MODULES:
        source = sources[f"{module}.rs"]
        lines += [f"### `{module}.rs`", "", "| Schema field | Status | Mapping / next audit |", "| --- | --- | --- |"]
        for name, fields in schema_structs(source):
            if (module, name) in ANIMATION_GESTURE_AUDIT_FIELDS:
                assert set(fields) == set(ANIMATION_GESTURE_AUDIT_FIELDS[(module, name)].split()), "Animation/gesture audit must cover every field"
            if (module, name) in INPUT_FOCUS_AUDIT_FIELDS:
                assert set(INPUT_FOCUS_AUDIT_FIELDS[(module, name)].split()).issubset(fields), "Input focus audit fields must exist"
            if (module, name) in LAYER_OVERVIEW_AUDIT_FIELDS:
                assert set(fields) == set(LAYER_OVERVIEW_AUDIT_FIELDS[(module, name)].split()), "Layer/overview audit must cover every field"
            if module == "layout" and name in {"Layout", "LayoutPart"}:
                assert set(fields) == set(LAYOUT_AUDIT), "Layout audit must account for every field"
            if module == "appearance" and name in APPEARANCE_AUDIT_FIELDS:
                assert set(fields) == set(APPEARANCE_AUDIT_FIELDS[name].split()), "Appearance audit must account for every field"
            if module == "appearance" and name in EFFECT_AUDIT_FIELDS:
                assert set(fields) == set(EFFECT_AUDIT_FIELDS[name].split()), "Effect audit must account for every field"
            if module == "window_rule" and name in EFFECT_CONSUMER_FIELDS:
                assert set(EFFECT_CONSUMER_FIELDS[name].split()).issubset(fields), "Effect consumers must exist in pinned schema"
            if module == "window_rule" and name == "Match":
                assert set(fields) == set(MATCH_AUDIT_FIELDS.split()), "Match audit must account for every property"
            if module == "window_rule" and name == "WindowRule":
                assert set(LIFECYCLE_WINDOW_FIELDS).issubset(fields), "Lifecycle field names must exist in pinned schema"
            for field in fields:
                count += 1
                status, mapping = config_status(module, name, field)
                lines.append(f"| `{name}.{field}` | {status} | {mapping} |")
        types = re.findall(r"^pub (?:struct|enum) (\w+)", source, re.M)
        lines += ["", "Declared types to check for nested values/custom decoding: " + ", ".join(f"`{t}`" for t in types) + ".", ""]
    lines += [f"Named schema fields indexed: **{count}**.", "", "## Behavioral differences to resolve", "",
        "- Consume pulls into the active column in the reference; our move action sends the focused tile to a neighbor.",
        "- Distinct first-consume/last-expel actions exist; complete boundary, ID-targeted, tab and view behavior remains unaudited.",
        "- Reference width presets include 2/3, proportional widths account for gaps and fixed widths have specific content/frame units.",
        "- Automatic tile heights use reference weighting semantics; equal content heights are not equivalent with mixed decorations.",
        "- Current fullscreen, floating exclusion, generic momentum and one-output fallback are not parity implementations.",
        "- Structural reorder now exists, but reference movement animation, fullscreen/tab context and complete view policy remain pending.", ""]
    return "\n".join(lines).rstrip()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    args = parser.parse_args()
    print(generate(args.checkout))
