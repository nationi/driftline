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


def config_status(module, name, field):
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
        "border/shadow and tab/insert-hint decoding and merges. Overrides, remaining",
        "appearance types and other schemas still need semantic auditing. The",
        "[lifecycle audit](niri-config-lifecycle-audit.md) traces override order and",
        "scoped window-rule initial-map/reload consumers; unscoped behavior stays open.",
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
    lines += ["", f"Action variants indexed: **{len(names)}**.", "", "## Configuration source index", "",
        "Includes complete named public-struct field sets in these reference modules,",
        "including partial/override schemas and non-layout fields. A row is not necessarily",
        "a directly bindable configuration key. Shared nested types are listed at their",
        "own definitions; tuple wrappers and enums are indexed by declaration below.", ""]
    count = 0
    for module in ["layout", "appearance", "gestures", "animations", "window_rule", "workspace", "output"]:
        source = (source_root / f"{module}.rs").read_text()
        lines += [f"### `{module}.rs`", "", "| Schema field | Status | Mapping / next audit |", "| --- | --- | --- |"]
        for match in re.finditer(r"^pub struct (\w+)[^;\n{]*\{", source, re.M):
            name = match.group(1)
            fields = re.findall(r"^    pub (\w+):", block(source, match.group(0)), re.M)
            if module == "layout" and name in {"Layout", "LayoutPart"}:
                assert set(fields) == set(LAYOUT_AUDIT), "Layout audit must account for every field"
            if module == "appearance" and name in APPEARANCE_AUDIT_FIELDS:
                assert set(fields) == set(APPEARANCE_AUDIT_FIELDS[name].split()), "Appearance audit must account for every field"
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
