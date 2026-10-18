# Niri tiling parity inventory

Reference: `12de51d6a60baad8896366920278a24f17921f52`. Rebuild with `python3 dev/scripts/niri-parity-inventory.py /tmp/niri > dev/docs/niri-tiling-inventory.md`.

This is a source-symbol inventory, not a claim that semantic parity is tested.
All Action variants are included to make exclusions visible; `ById` and internal
variants remain separate rows. Names are Rust symbols, not guessed binding aliases.
The reference has decoder attributes/custom parsers: public schema fields below
must still be checked against decoded option names, defaults and lifecycle rules.
P0 reference operation fixtures and the custom-decoder audit remain unfinished.

Status: Partial = related implementation, not full equivalence; Divergent = known
semantic difference; Missing = no matching tiling action; Unverified = not audited;
Outside = explicit non-layout scope exclusion. Never infer Exact from a similar name.

## Reference actions

| Reference Action | Scope | Status | driftwm mapping / gap | Evidence / remaining gate |
| --- | --- | --- | --- | --- |
| `Quit` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ChangeVt` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `Suspend` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `PowerOffMonitors` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `PowerOnMonitors` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ToggleDebugTint` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `DebugToggleOpaqueRegions` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `DebugToggleDamage` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ToggleTouchpad` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `Spawn` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `SpawnSh` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `DoScreenTransition` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ConfirmScreenshot` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `CancelScreenshot` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ScreenshotTogglePointer` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `Screenshot` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ScreenshotScreen` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ScreenshotWindow` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ScreenshotWindowById` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ToggleKeyboardShortcutsInhibit` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `CloseWindow` | Layout | Partial | close-window | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `CloseWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FullscreenWindow` | Layout | Divergent | toggle-fullscreen exits tiling; topology not retained | incompatible_actions_leave_tiling_before_applying_canvas_policy |
| `FullscreenWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleWindowedFullscreen` | Layout | Divergent | toggle-fullscreen exits tiling; topology not retained | incompatible_actions_leave_tiling_before_applying_canvas_policy |
| `ToggleWindowedFullscreenById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindow` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowInColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnLeft` | Layout | Partial | focus-column left | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `FocusColumnLeftUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnRight` | Layout | Partial | focus-column right | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `FocusColumnRightUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnRightOrFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnLeftOrLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowOrMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowOrMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnOrMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusColumnOrMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowDown` | Layout | Partial | focus-column down | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `FocusWindowUp` | Layout | Partial | focus-column up | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `FocusWindowDownOrColumnLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowDownOrColumnRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowUpOrColumnLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowUpOrColumnRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowOrWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowOrWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowTop` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowBottom` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowDownOrTop` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWindowUpOrBottom` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnLeft` | Layout | Partial | move-column left | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `MoveColumnRight` | Layout | Partial | move-column right | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `MoveColumnToFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnLeftOrToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnRightOrToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToIndex` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowDown` | Layout | Partial | move-tiled-window down | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `MoveWindowUp` | Layout | Partial | move-tiled-window up | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `MoveWindowDownOrToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowUpOrToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ConsumeOrExpelWindowLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ConsumeOrExpelWindowLeftById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ConsumeOrExpelWindowRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ConsumeOrExpelWindowRightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ConsumeWindowIntoColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ExpelWindowFromColumn` | Layout | Divergent | expel-from-column targets focused tile, reference targets last tile | grouped_windows_share_height_navigate_and_restore_their_original_rectangles |
| `SwapWindowLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwapWindowRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleColumnTabbedDisplay` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetColumnDisplay` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `CenterColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `CenterWindow` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `CenterWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `CenterVisibleColumns` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspaceDownUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspaceUpUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusWorkspacePrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToWorkspaceById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToIndex` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToIndexByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWorkspaceName` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWorkspaceNameByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `UnsetWorkspaceName` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `UnsetWorkSpaceNameByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToMonitorById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveColumnToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWindowWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWindowWidthById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ResetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ResetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetColumnWidth` | Layout | Partial | cycle-column-width (different presets/units) | src/tests/tiling.rs; src/stage/tiling.rs tests |
| `SwitchPresetColumnWidthBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowWidthBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowWidthById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowWidthBackById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowHeightBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchPresetWindowHeightBackById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MaximizeColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MaximizeWindowToEdges` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MaximizeWindowToEdgesById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SetColumnWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ExpandColumnToAvailableWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchLayout` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ShowHotkeyOverlay` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `MoveWorkspaceToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWorkspaceToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleWindowFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleWindowFloatingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToFloatingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveWindowToTilingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `FocusTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `SwitchFocusBetweenFloatingAndTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `MoveFloatingWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleWindowRuleOpacity` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ToggleWindowRuleOpacityById` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `SetDynamicCastWindow` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `SetDynamicCastWindowById` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `SetDynamicCastMonitor` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ClearDynamicCastTarget` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `StopCast` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `ToggleOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `OpenOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `CloseOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `ToggleWindowUrgent` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `SetWindowUrgent` | Layout | Missing | — | P4/P6–P14; no parity evidence yet |
| `UnsetWindowUrgent` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `LoadConfigFile` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |
| `MruAdvance` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruConfirm` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruCancel` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruCloseCurrentWindow` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruFirst` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruLast` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruSetScope` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `MruCycleScope` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending |
| `TestAction` | Outside | Outside | unrelated compositor action | roadmap scope; reconsider if layout-visible |

Action variants indexed: **186**.

## Configuration source index

Includes complete named public-struct field sets in these reference modules,
including partial/override schemas and non-layout fields. A row is not necessarily
a directly bindable configuration key. Shared nested types are listed at their
own definitions; tuple wrappers and enums are indexed by declaration below.

### `layout.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Layout.focus_ring` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.border` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.shadow` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.tab_indicator` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.insert_hint` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.preset_column_widths` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `Layout.default_column_width` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `Layout.preset_window_heights` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.center_focused_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.always_center_single_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.empty_workspace_above_first` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.default_column_display` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.gaps` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `Layout.struts` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Layout.background_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.focus_ring` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.border` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.shadow` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.tab_indicator` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.insert_hint` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.preset_column_widths` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `LayoutPart.default_column_width` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `LayoutPart.preset_window_heights` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.center_focused_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.always_center_single_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.empty_workspace_above_first` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.default_column_display` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.gaps` | Partial | Hardcoded widths / shared snap.gap; P3 dedicated tiling settings and units missing |
| `LayoutPart.struts` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `LayoutPart.background_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Struts.left` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Struts.right` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Struts.top` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Struts.bottom` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Layout`, `LayoutPart`, `PresetSize`, `DefaultPresetSize`, `Struts`, `CenterFocusedColumn`.

### `appearance.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Color.r` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Color.g` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Color.b` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Color.a` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gradient.from` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gradient.to` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gradient.angle` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gradient.relative_to` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gradient.in_` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `GradientInterpolation.color_space` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `GradientInterpolation.hue_interpolation` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `CornerRadius.top_left` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `CornerRadius.top_right` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `CornerRadius.bottom_right` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `CornerRadius.bottom_left` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FocusRing.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Border.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.softness` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.spread` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.draw_behind_window` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Shadow.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowOffset.x` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowOffset.y` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadow.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadow.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadow.softness` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadow.spread` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadow.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.softness` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.spread` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WorkspaceShadowPart.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.hide_when_single_tab` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.place_within_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.gap` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.length` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.position` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.gaps_between_tabs` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.corner_radius` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicator.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.hide_when_single_tab` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.place_within_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.gap` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.length` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.position` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.gaps_between_tabs` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.corner_radius` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorPart.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorLength.total_proportion` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHint.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHint.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHint.gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHintPart.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHintPart.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHintPart.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `InsertHintPart.gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BorderRule.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.softness` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.spread` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.draw_behind_window` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ShadowRule.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.active_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.inactive_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.urgent_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.active_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.inactive_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `TabIndicatorRule.urgent_gradient` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Blur.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Blur.passes` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Blur.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Blur.noise` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Blur.saturation` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.passes` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.offset` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.noise` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BlurPart.saturation` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffectRule.xray` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffectRule.blur` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffectRule.noise` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffectRule.saturation` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffect.xray` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffect.blur` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffect.noise` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `BackgroundEffect.saturation` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Color`, `Gradient`, `GradientRelativeTo`, `GradientInterpolation`, `GradientColorSpace`, `HueInterpolation`, `CornerRadius`, `FocusRing`, `Border`, `Shadow`, `ShadowOffset`, `WorkspaceShadow`, `WorkspaceShadowPart`, `TabIndicator`, `TabIndicatorPart`, `TabIndicatorLength`, `TabIndicatorPosition`, `InsertHint`, `InsertHintPart`, `BlockOutFrom`, `BorderRule`, `ShadowRule`, `TabIndicatorRule`, `Blur`, `BlurPart`, `BackgroundEffectRule`, `BackgroundEffect`.

### `gestures.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Gestures.dnd_edge_view_scroll` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gestures.dnd_edge_workspace_switch` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Gestures.hot_corners` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `GesturesPart.dnd_edge_view_scroll` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `GesturesPart.dnd_edge_workspace_switch` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `GesturesPart.hot_corners` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScroll.trigger_width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScroll.delay_ms` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScroll.max_speed` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScrollPart.trigger_width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScrollPart.delay_ms` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeViewScrollPart.max_speed` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitch.trigger_height` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitch.delay_ms` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitch.max_speed` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitchPart.trigger_height` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitchPart.delay_ms` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `DndEdgeWorkspaceSwitchPart.max_speed` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `HotCorners.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `HotCorners.top_left` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `HotCorners.top_right` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `HotCorners.bottom_left` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `HotCorners.bottom_right` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Gestures`, `GesturesPart`, `DndEdgeViewScroll`, `DndEdgeViewScrollPart`, `DndEdgeWorkspaceSwitch`, `DndEdgeWorkspaceSwitchPart`, `HotCorners`.

### `animations.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Animations.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.slowdown` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.workspace_switch` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.window_open` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.window_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.horizontal_view_movement` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.window_movement` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.window_resize` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.config_notification_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.exit_confirmation_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.screenshot_ui_open` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.overview_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animations.recent_windows_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.on` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.slowdown` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.workspace_switch` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.window_open` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.window_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.horizontal_view_movement` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.window_movement` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.window_resize` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.config_notification_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.exit_confirmation_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.screenshot_ui_open` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.overview_open_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `AnimationsPart.recent_windows_close` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animation.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Animation.kind` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `EasingParams.duration_ms` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `EasingParams.curve` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `SpringParams.damping_ratio` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `SpringParams.stiffness` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `SpringParams.epsilon` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowOpenAnim.anim` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowOpenAnim.custom_shader` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowCloseAnim.anim` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowCloseAnim.custom_shader` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowResizeAnim.anim` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowResizeAnim.custom_shader` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Animations`, `AnimationsPart`, `Animation`, `Kind`, `EasingParams`, `Curve`, `SpringParams`, `WorkspaceSwitchAnim`, `WindowOpenAnim`, `WindowCloseAnim`, `HorizontalViewMovementAnim`, `WindowMovementAnim`, `WindowResizeAnim`, `ConfigNotificationOpenCloseAnim`, `ExitConfirmationOpenCloseAnim`, `ScreenshotUiOpenAnim`, `OverviewOpenCloseAnim`, `RecentWindowsCloseAnim`.

### `window_rule.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `WindowRule.matches` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.excludes` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.default_column_width` | Missing | P3/P8/P10 lifecycle-aware tiling window rules |
| `WindowRule.default_window_height` | Missing | P3/P8/P10 lifecycle-aware tiling window rules |
| `WindowRule.open_on_output` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.open_on_workspace` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.open_maximized` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.open_maximized_to_edges` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.open_fullscreen` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.open_floating` | Missing | P3/P8/P10 lifecycle-aware tiling window rules |
| `WindowRule.open_focused` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.on_xdg_activate` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.min_width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.min_height` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.max_width` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.max_height` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.focus_ring` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.border` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.shadow` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.tab_indicator` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.draw_border_with_background` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.opacity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.geometry_corner_radius` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.clip_to_geometry` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.baba_is_float` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.block_out_from` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.variable_refresh_rate` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.default_column_display` | Missing | P3/P8/P10 lifecycle-aware tiling window rules |
| `WindowRule.default_floating_position` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.scroll_factor` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.pinch_sensitivity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.tiled_state` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.background_effect` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `WindowRule.popups` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `PopupsRule.opacity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `PopupsRule.geometry_corner_radius` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `PopupsRule.background_effect` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ResolvedPopupsRules.opacity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ResolvedPopupsRules.geometry_corner_radius` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `ResolvedPopupsRules.background_effect` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.app_id` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.title` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_active` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_focused` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_active_in_column` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_floating` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_window_cast_target` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.is_urgent` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Match.at_startup` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FloatingPosition.x` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FloatingPosition.y` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `FloatingPosition.relative_to` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `WindowRule`, `PopupsRule`, `ResolvedPopupsRules`, `Match`, `FloatingPosition`, `OnXdgActivate`, `RelativeTo`.

### `workspace.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Workspace.name` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Workspace.open_on_output` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Workspace.layout` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Workspace`, `WorkspaceName`, `WorkspaceLayoutPart`.

### `output.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Mode.custom` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Mode.mode` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.clock` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.hdisplay` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.hsync_start` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.hsync_end` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.htotal` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.vdisplay` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.vsync_start` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.vsync_end` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.vtotal` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.hsync_polarity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Modeline.vsync_polarity` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.off` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.name` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.scale` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.transform` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.position` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.max_bpc` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.mode` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.modeline` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.variable_refresh_rate` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.focus_at_startup` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.background_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.backdrop_color` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.hot_corners` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Output.layout` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `OutputName.connector` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `OutputName.make` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `OutputName.model` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `OutputName.serial` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Position.x` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Position.y` | Unverified | Inspect decoder and reference behavior; no verified option mapping |
| `Vrr.on_demand` | Unverified | Inspect decoder and reference behavior; no verified option mapping |

Declared types to check for nested values/custom decoding: `Outputs`, `Mode`, `Modeline`, `Output`, `OutputName`, `Position`, `MaxBpc`, `Vrr`.

Named schema fields indexed: **318**.

## Behavioral differences to resolve

- Consume pulls into the active column in the reference; our move action sends the focused tile to a neighbor.
- Reference expel and directional consume-or-expel differ from our focused-tile expel. Add distinct actions; retain compatibility.
- Reference width presets include 2/3, proportional widths account for gaps and fixed widths have specific content/frame units.
- Automatic tile heights use reference weighting semantics; equal content heights are not equivalent with mixed decorations.
- Current fullscreen, floating exclusion, generic momentum and one-output fallback are not parity implementations.
- Structural reorder now exists, but reference movement animation, fullscreen/tab context and complete view policy remain pending.
