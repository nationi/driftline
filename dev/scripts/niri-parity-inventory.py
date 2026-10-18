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
    }
    if name in partial:
        return "Layout", "Partial", partial[name], "src/tests/tiling.rs; src/stage/tiling.rs tests"
    if name == "ExpelWindowFromColumn":
        return "Layout", "Divergent", "expel-from-column targets focused tile, reference targets last tile", "grouped_windows_share_height_navigate_and_restore_their_original_rectangles"
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
        "must still be checked against decoded option names, defaults and lifecycle rules.",
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
            for field in fields:
                count += 1
                status = "Unverified"
                mapping = "Inspect decoder and reference behavior; no verified option mapping"
                if module == "layout" and field in {"preset_column_widths", "default_column_width", "gaps"}:
                    status = "Partial"
                    mapping = "Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing"
                if module == "window_rule" and field in {"default_column_width", "default_window_height", "default_column_display", "open_floating"}:
                    status = "Missing"
                    mapping = "P3/P8/P10 lifecycle-aware tiling window rules"
                lines.append(f"| `{name}.{field}` | {status} | {mapping} |")
        types = re.findall(r"^pub (?:struct|enum) (\w+)", source, re.M)
        lines += ["", "Declared types to check for nested values/custom decoding: " + ", ".join(f"`{t}`" for t in types) + ".", ""]
    lines += [f"Named schema fields indexed: **{count}**.", "", "## Behavioral differences to resolve", "",
        "- Consume pulls into the active column in the reference; our move action sends the focused tile to a neighbor.",
        "- Reference expel and directional consume-or-expel differ from our focused-tile expel. Add distinct actions; retain compatibility.",
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
