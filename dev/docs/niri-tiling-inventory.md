# Niri tiling parity inventory

Reference: `12de51d6a60baad8896366920278a24f17921f52`. Rebuild with `python3 dev/scripts/niri-parity-inventory.py /tmp/niri > dev/docs/niri-tiling-inventory.md`.

This is a source-symbol inventory, not a claim that semantic parity is tested.
All Action variants are included to make exclusions visible; `ById` and internal
variants remain separate rows. Names are Rust symbols, not guessed binding aliases.
The reference has decoder attributes/custom parsers: public schema fields below
are not automatically public keys. The [layout audit](niri-layout-config-audit.md)
covers top-level layout names/defaults and shared size/flag/strut decoding;
the [appearance audit](niri-appearance-config-audit.md) covers color/gradient,
border/shadow and tab/insert-hint decoding and merges. The
[lifecycle audit](niri-config-lifecycle-audit.md) traces override order and
scoped window-rule initial-map/reload consumers. The [effects audit](niri-effects-config-audit.md)
adds radius/workspace-shadow/blur/background-effect decoding and consumers.
The [schema coverage audit](niri-config-schema-coverage.md) accounts for all
18 source files, 25 root nodes and 14 indexed modules. Fields are source
inventory, not a completed public decoder/consumer audit. Unscoped behavior stays open.
The [layer/overview consumers](niri-layer-overview-audit.md) audit traces newly indexed
layer rules and overview value consumers; complete interactions remain open.
The [input focus consumers](niri-input-focus-audit.md) audit traces scoped focus, warp,
workspace back-and-forth and five layout-visible debug flags; other fields stay open.
The [animation/gesture consumers](niri-animation-gesture-audit.md) audit traces those
schema fields and scoped consumers; physics/interaction execution remains open.
The [input/output integration consumers](niri-input-output-integration-audit.md) review
scroll/pinch delivery, identity and remaining debug scope; retained Unverified gates stay open.
The [device input consumers](niri-device-input-audit.md) review completes named input
source coverage; retained hardware/keymap/protocol acceptance stays open.
The [output/window rule consumers](niri-output-window-rule-audit.md) review completes
named rows in those modules; runtime/hardware/decoder fixtures remain open.
P0 reference operation fixtures and the custom-decoder audit remain unfinished.

Status: Partial = related implementation, not full equivalence; Divergent = known
semantic difference; Missing = no matching tiling action; Unverified = not audited;
Outside = explicit non-layout scope exclusion. Never infer Exact from a similar name.

## Reference actions

| Reference Action | Scope | Status | driftwm mapping / gap | Evidence / remaining gate |
| --- | --- | --- | --- | --- |
| `Quit` | Outside | Outside | Exit/confirmation command UX outside tiling; window teardown acceptance retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ChangeVt` | Integration | Unverified | VT/session transition clears suppressed keys and repeat; integration acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `Suspend` | Integration | Unverified | Session transition clears suppressed keys and repeat; resume/lifecycle acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `PowerOffMonitors` | Integration | Unverified | Monitor-active state/backend output power; ownership and resume acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `PowerOnMonitors` | Integration | Unverified | Monitor-active state/backend output power and redraw; recovery acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ToggleDebugTint` | Outside | Outside | Diagnostic tint overlay outside visual parity; ordinary rendering retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `DebugToggleOpaqueRegions` | Outside | Outside | Diagnostic region overlay outside visual parity; opacity/occlusion acceptance retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `DebugToggleDamage` | Outside | Outside | Diagnostic damage overlay outside visual parity; damage/render correctness retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ToggleTouchpad` | Integration | Unverified | Runtime toggle reapplies touchpad settings; device/input acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `Spawn` | Outside | Outside | Program launch outside tiling; activation and mapped-window policy retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `SpawnSh` | Outside | Outside | Shell launch outside tiling; activation and mapped-window policy retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `DoScreenTransition` | Integration | Unverified | Target-specific screen snapshots and transition; rendering/privacy acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ConfirmScreenshot` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `CancelScreenshot` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ScreenshotTogglePointer` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `Screenshot` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ScreenshotScreen` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ScreenshotWindow` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ScreenshotWindowById` | Integration | Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ToggleKeyboardShortcutsInhibit` | Integration | Unverified | Focused surface inhibitor activation; keyboard resolution acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `CloseWindow` | Layout | Partial | close-window | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `CloseWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FullscreenWindow` | Layout | Divergent | toggle-fullscreen exits tiling; topology not retained | incompatible_actions_leave_tiling_before_applying_canvas_policy; [local/state action audit](niri-local-state-action-audit.md) |
| `FullscreenWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `ToggleWindowedFullscreen` | Layout | Divergent | toggle-fullscreen exits tiling; topology not retained | incompatible_actions_leave_tiling_before_applying_canvas_policy; [local/state action audit](niri-local-state-action-audit.md) |
| `ToggleWindowedFullscreenById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindow` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusWindowInColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusWindowPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusColumnLeft` | Layout | Partial | focus-column left | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusColumnLeftUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusColumnRight` | Layout | Partial | focus-column right | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusColumnRightUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusColumnFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusColumnLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusColumnRightOrFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusColumnLeftOrLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusWindowOrMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusWindowOrMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusColumnOrMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusColumnOrMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusWindowDown` | Layout | Partial | focus-column down | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowUp` | Layout | Partial | focus-column up | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowDownOrColumnLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowDownOrColumnRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowUpOrColumnLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowUpOrColumnRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowOrWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusWindowOrWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusWindowTop` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowBottom` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowDownOrTop` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusWindowUpOrBottom` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveColumnLeft` | Layout | Partial | move-column left | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveColumnRight` | Layout | Partial | move-column right | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveColumnToFirst` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveColumnToLast` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveColumnLeftOrToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnRightOrToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToIndex` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `MoveWindowDown` | Layout | Partial | move-tiled-window down | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowUp` | Layout | Partial | move-tiled-window up | src/tests/tiling.rs; src/stage/tiling.rs tests; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowDownOrToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowUpOrToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `ConsumeOrExpelWindowLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ConsumeOrExpelWindowLeftById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ConsumeOrExpelWindowRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ConsumeOrExpelWindowRightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ConsumeWindowIntoColumn` | Layout | Partial | consume-window-into-column (topology/focus subset) | src/tests/tiling.rs; src/stage/tiling.rs tests; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ExpelWindowFromColumn` | Layout | Partial | expel-window-from-column (topology/focus subset) | src/tests/tiling.rs; src/stage/tiling.rs tests; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwapWindowLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwapWindowRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ToggleColumnTabbedDisplay` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `SetColumnDisplay` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `CenterColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `CenterWindow` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `CenterWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `CenterVisibleColumns` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `FocusWorkspaceDownUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `FocusWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `FocusWorkspaceUpUnderMouse` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `FocusWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `FocusWorkspacePrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [action address audit](niri-binding-address-audit.md#scoped-ordinal-and-previous-selection) |
| `MoveWindowToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveWindowToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveWindowToWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWindowToWorkspaceById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveColumnToWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveColumnToWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveColumnToWorkspace` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWorkspaceDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveWorkspaceUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `MoveWorkspaceToIndex` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWorkspaceToIndexByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWorkspaceToMonitorByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWorkspaceToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `SetWorkspaceName` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `SetWorkspaceNameByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `UnsetWorkspaceName` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `UnsetWorkSpaceNameByRef` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `FocusMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `FocusMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWindowToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveWindowToMonitorById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `MoveColumnToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveColumnToMonitor` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `SetWindowWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `SetWindowWidthById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `SetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `SetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `ResetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `ResetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetColumnWidth` | Layout | Partial | cycle-column-width (different presets/units) | src/tests/tiling.rs; src/stage/tiling.rs tests; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetColumnWidthBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowWidthBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowWidthById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowWidthBackById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowHeight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowHeightBack` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowHeightById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SwitchPresetWindowHeightBackById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `MaximizeColumn` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `MaximizeWindowToEdges` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `MaximizeWindowToEdgesById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [preset/topology action audit](niri-preset-topology-action-audit.md) |
| `SetColumnWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `ExpandColumnToAvailableWidth` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `SwitchLayout` | Integration | Unverified | XKB next/prev/index switch; keyboard/focus acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `ShowHotkeyOverlay` | Outside | Outside | Shortcut-help presentation outside tiling; binding resolution retained | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MoveWorkspaceToMonitorLeft` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWorkspaceToMonitorRight` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWorkspaceToMonitorDown` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWorkspaceToMonitorUp` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWorkspaceToMonitorPrevious` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `MoveWorkspaceToMonitorNext` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [navigation/view action audit](niri-navigation-view-action-audit.md) |
| `ToggleWindowFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `ToggleWindowFloatingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowToFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowToFloatingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowToTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveWindowToTilingById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusFloating` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `FocusTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `SwitchFocusBetweenFloatingAndTiling` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `MoveFloatingWindowById` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [size/transfer action audit](niri-size-transfer-action-audit.md) |
| `ToggleWindowRuleOpacity` | Integration | Missing | Reference ignore-opacity-rule toggle missing | Mapped opacity/focus/reload acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `ToggleWindowRuleOpacityById` | Integration | Missing | Reference ignore-opacity-rule toggle missing | Mapped opacity/focus/reload acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `SetDynamicCastWindow` | Integration | Unverified | Cast target can affect rule matching/privacy; not excluded | Feature/session/target/lifecycle acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `SetDynamicCastWindowById` | Integration | Unverified | Cast target can affect rule matching/privacy; not excluded | Feature/session/target/lifecycle acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `SetDynamicCastMonitor` | Integration | Unverified | Cast target can affect rule matching/privacy; not excluded | Feature/session/target/lifecycle acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `ClearDynamicCastTarget` | Integration | Unverified | Cast target can affect rule matching/privacy; not excluded | Feature/session/target/lifecycle acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `StopCast` | Integration | Unverified | Cast target can affect rule matching/privacy; not excluded | Feature/session/target/lifecycle acceptance remains open; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `ToggleOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `OpenOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `CloseOverview` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [local/state action audit](niri-local-state-action-audit.md) |
| `ToggleWindowUrgent` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `SetWindowUrgent` | Layout | Missing | — | P4/P6–P14; no parity evidence yet; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `UnsetWindowUrgent` | Integration | Unverified | existing focus/window state is not a parity test | P8/P10/P14; behavior audit pending; [workspace/policy action audit](niri-workspace-policy-action-audit.md) |
| `LoadConfigFile` | Integration | Unverified | Watcher-selected config reload; layout/rule reload acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruAdvance` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruConfirm` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruCancel` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruCloseCurrentWindow` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruFirst` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruLast` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruSetScope` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `MruCycleScope` | Integration | Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |
| `TestAction` | Outside | Outside | Test-only dispatcher counter, no production layout policy | Explicit consumer/exclusion review; retained acceptance open; [integration/scope action audit](niri-integration-action-scope-audit.md) |

Action variants indexed: **186**.

## Action config exposure

Derived config decoder variants: **136**; skipped variants: **50**.
Derived does not mean fully audited arguments/consumers. Skipped is not an IPC whitelist.
Source-skipped config variants: `ChangeVt`, `ConfirmScreenshot`, `CancelScreenshot`, `ScreenshotTogglePointer`, `ScreenshotWindowById`, `CloseWindowById`, `FullscreenWindowById`, `ToggleWindowedFullscreenById`, `FocusWindow`, `FocusColumnLeftUnderMouse`, `FocusColumnRightUnderMouse`, `ConsumeOrExpelWindowLeftById`, `ConsumeOrExpelWindowRightById`, `CenterWindowById`, `FocusWorkspaceDownUnderMouse`, `FocusWorkspaceUpUnderMouse`, `MoveWindowToWorkspaceById`, `MoveWorkspaceToIndexByRef`, `MoveWorkspaceToMonitorByRef`, `SetWorkspaceNameByRef`, `UnsetWorkSpaceNameByRef`, `MoveWindowToMonitorById`, `SetWindowWidthById`, `SetWindowHeightById`, `ResetWindowHeightById`, `SwitchPresetWindowWidthById`, `SwitchPresetWindowWidthBackById`, `SwitchPresetWindowHeightById`, `SwitchPresetWindowHeightBackById`, `MaximizeWindowToEdgesById`, `ToggleWindowFloatingById`, `MoveWindowToFloatingById`, `MoveWindowToTilingById`, `MoveFloatingWindowById`, `ToggleWindowRuleOpacityById`, `SetDynamicCastWindowById`, `StopCast`, `ToggleWindowUrgent`, `SetWindowUrgent`, `UnsetWindowUrgent`, `LoadConfigFile`, `MruAdvance`, `MruConfirm`, `MruCancel`, `MruCloseCurrentWindow`, `MruFirst`, `MruLast`, `MruSetScope`, `MruCycleScope`, `TestAction`.
See the [binding/address interfaces](niri-binding-address-audit.md#config-exposure-versus-ipc).

## Module coverage

| Source file | Scope | Index | Named fields | Coverage / exclusion reason |
| --- | --- | --- | --- | --- |
| `lib.rs` | Mixed/root | Indexed | 24 | Config fields plus custom root-node dispatch; internal helpers are not public keys |
| `layout.rs` | Layout | Indexed | 34 | Column geometry, defaults and partial layout overrides |
| `appearance.rs` | Layout/integration | Indexed | 132 | Decoration, geometry, blur and effect values |
| `gestures.rs` | Layout/integration | Indexed | 23 | View scrolling, workspace switching and hot corners |
| `animations.rs` | Layout | Indexed | 40 | Movement, resize and workspace/overview transitions |
| `window_rule.rs` | Layout/integration | Indexed | 52 | Window matching, opening, sizing and dynamic appearance |
| `workspace.rs` | Layout | Indexed | 3 | Named workspace routing and layout overrides |
| `output.rs` | Layout/integration | Indexed | 34 | Output owners, modes/scales and layout overrides |
| `misc.rs` | Mixed | Indexed | 31 | Includes Overview/OverviewPart; unrelated process/UI settings are explicitly excluded by root scope |
| `layer_rule.rs` | Integration | Indexed | 13 | Layer matching, placement/backdrop and effects; full semantic audit pending |
| `input.rs` | Mixed/integration | Indexed | 103 | Focus, warping, workspace policy and modifiers coexist with device settings; no blanket exclusion |
| `debug.rs` | Mixed/integration | Indexed | 44 | Transaction, resize/focus and output toggles coexist with backend flags; no blanket exclusion |
| `binds.rs` | Layout/integration | Indexed | 14 | Public bindings/actions and switch events; aliases/units/addressing need audit |
| `recent_windows.rs` | Layout/integration | Indexed | 29 | MRU focus overlay and previews; distinct from unrelated help UI |
| `error.rs` | Infrastructure | Infrastructure only | 4 | Diagnostic/result payloads, not configurable option schemas; named fields intentionally not indexed |
| `macros.rs` | Infrastructure | Infrastructure only | 0 | Merge helpers, audited through their consumer schemas; no named option fields |
| `utils.rs` | Infrastructure | Infrastructure only | 0 | Custom scalar/wrapper decoders, audited through consumers; declarations are not standalone option trees |
| `utils/merge_with.rs` | Infrastructure | Infrastructure only | 0 | Merge trait implementation helper; no configurable option fields |

## Public root decoder nodes

`include` is a custom decoder directive, not a missing Config field. Singular
public node names such as `output` and `window-rule` are not plural Rust fields.

| Public root node | Config field | Scope | Status | Evidence / next audit |
| --- | --- | --- | --- | --- |
| `input` | `Config.input` | Integration | Unverified | Includes focus/warping/workspace policy; audit per field; [coverage](niri-config-schema-coverage.md) |
| `cursor` | `Config.cursor` | Integration | Unverified | Cursor interaction/visibility boundaries not audited; [coverage](niri-config-schema-coverage.md) |
| `clipboard` | `Config.clipboard` | Outside | Outside | Selection policy, not tiling topology or sizing; [coverage](niri-config-schema-coverage.md) |
| `hotkey-overlay` | `Config.hotkey_overlay` | Outside | Outside | Shortcut help UI, not workspace overview or MRU focus; [coverage](niri-config-schema-coverage.md) |
| `config-notification` | `Config.config_notification` | Outside | Outside | Config-error notification presentation, not layout policy; [coverage](niri-config-schema-coverage.md) |
| `animations` | `Config.animations` | Layout | Divergent | Frame factors differ from per-event spring/easing; scoped source audit recorded; [coverage](niri-config-schema-coverage.md) |
| `blur` | `Config.blur` | Layout/integration | Partial | Tuning/effect-selection source audit exists; runtime parity not established; [coverage](niri-config-schema-coverage.md) |
| `gestures` | `Config.gestures` | Layout/integration | Missing | Reference DnD policy missing; hot-corner behavior differs; scoped source audit recorded; [coverage](niri-config-schema-coverage.md) |
| `overview` | `Config.overview` | Layout | Missing | Overview exists in pinned schema; no reference overview implementation; [coverage](niri-config-schema-coverage.md) |
| `xwayland-satellite` | `Config.xwayland_satellite` | Integration | Unverified | Client/protocol integration; reconsider layout-visible lifecycle effects; [coverage](niri-config-schema-coverage.md) |
| `switch-events` | `Config.switch_events` | Integration | Unverified | Lid/output actions may affect layout lifecycle; [coverage](niri-config-schema-coverage.md) |
| `debug` | `Config.debug` | Integration | Unverified | Mixed layout-visible transaction/focus flags and backend controls; [coverage](niri-config-schema-coverage.md) |
| `output` | `Config.outputs` | Layout/integration | Partial | Repeated output configs; ownership/layout/name/mode surface not fully audited; [coverage](niri-config-schema-coverage.md) |
| `spawn-at-startup` | `Config.spawn_at_startup` | Outside | Outside | Process launch list; resulting window map remains in layout scope; [coverage](niri-config-schema-coverage.md) |
| `spawn-sh-at-startup` | `Config.spawn_sh_at_startup` | Outside | Outside | Shell process launch; not a tiling action contract; [coverage](niri-config-schema-coverage.md) |
| `window-rule` | `Config.window_rules` | Layout/integration | Partial | Scoped decoder/lifecycle audits exist; remaining fields and fixtures pending; [coverage](niri-config-schema-coverage.md) |
| `layer-rule` | `Config.layer_rules` | Integration | Missing | Dedicated rule tree missing; scoped decoder/consumer audit recorded; [coverage](niri-config-schema-coverage.md) |
| `workspace` | `Config.workspaces` | Layout | Missing | Named workspace/layout hierarchy not implemented; [coverage](niri-config-schema-coverage.md) |
| `binds` | `Config.binds` | Layout/integration | Unverified | Action inventory is not a public binding alias/decoder audit; [coverage](niri-config-schema-coverage.md) |
| `environment` | `Config.environment` | Outside | Outside | Child-process environment, not layout behavior; [coverage](niri-config-schema-coverage.md) |
| `prefer-no-csd` | `Config.prefer_no_csd` | Integration | Unverified | Decoration negotiation/tiled-state behavior can affect layout; [coverage](niri-config-schema-coverage.md) |
| `screenshot-path` | `Config.screenshot_path` | Outside | Outside | Destination filename policy; capture/effect behavior remains separately in scope; [coverage](niri-config-schema-coverage.md) |
| `layout` | `Config.layout` | Layout | Partial | Top-level names/defaults audited; full option behavior/fixtures remain partial; [coverage](niri-config-schema-coverage.md) |
| `recent-windows` | `Config.recent_windows` | Layout/integration | Missing | MRU focus overlay/preview configuration has no parity implementation; [coverage](niri-config-schema-coverage.md) |
| `include` | — | Infrastructure | Unverified | Positional path/error source review recorded; executable parser/watcher acceptance pending; [coverage](niri-config-schema-coverage.md) |

## Configuration source index

Includes complete named public-struct field sets in these reference modules,
including partial/override schemas and non-layout fields. A row is not necessarily
a directly bindable configuration key. Shared nested types are listed at their
own definitions; tuple wrappers and enums are indexed by declaration below.

### `lib.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Config.input` | Unverified | Root input: Includes focus/warping/workspace policy; audit per field; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.outputs` | Partial | Root output: Repeated output configs; ownership/layout/name/mode surface not fully audited; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.spawn_at_startup` | Outside | Root spawn-at-startup: Process launch list; resulting window map remains in layout scope; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.spawn_sh_at_startup` | Outside | Root spawn-sh-at-startup: Shell process launch; not a tiling action contract; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.layout` | Partial | Root layout: Top-level names/defaults audited; full option behavior/fixtures remain partial; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.prefer_no_csd` | Unverified | Root prefer-no-csd: Decoration negotiation/tiled-state behavior can affect layout; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.cursor` | Unverified | Root cursor: Cursor interaction/visibility boundaries not audited; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.screenshot_path` | Outside | Root screenshot-path: Destination filename policy; capture/effect behavior remains separately in scope; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.clipboard` | Outside | Root clipboard: Selection policy, not tiling topology or sizing; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.hotkey_overlay` | Outside | Root hotkey-overlay: Shortcut help UI, not workspace overview or MRU focus; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.config_notification` | Outside | Root config-notification: Config-error notification presentation, not layout policy; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.animations` | Divergent | Root animations: Frame factors differ from per-event spring/easing; scoped source audit recorded; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.blur` | Partial | Root blur: Tuning/effect-selection source audit exists; runtime parity not established; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.gestures` | Missing | Root gestures: Reference DnD policy missing; hot-corner behavior differs; scoped source audit recorded; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.overview` | Missing | Root overview: Overview exists in pinned schema; no reference overview implementation; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.environment` | Outside | Root environment: Child-process environment, not layout behavior; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.xwayland_satellite` | Unverified | Root xwayland-satellite: Client/protocol integration; reconsider layout-visible lifecycle effects; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.window_rules` | Partial | Root window-rule: Scoped decoder/lifecycle audits exist; remaining fields and fixtures pending; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.layer_rules` | Missing | Root layer-rule: Dedicated rule tree missing; scoped decoder/consumer audit recorded; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.binds` | Unverified | Root binds: Action inventory is not a public binding alias/decoder audit; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.switch_events` | Unverified | Root switch-events: Lid/output actions may affect layout lifecycle; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.debug` | Unverified | Root debug: Mixed layout-visible transaction/focus flags and backend controls; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.workspaces` | Missing | Root workspace: Named workspace/layout hierarchy not implemented; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |
| `Config.recent_windows` | Missing | Root recent-windows: MRU focus overlay/preview configuration has no parity implementation; [root/misc audit](niri-root-misc-audit.md#root-storage-scope-and-ordering) |

Declared types to check for nested values/custom decoding: `Config`, `ConfigPath`, `ConfigPart`.

### `layout.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Layout.focus_ring` | Missing | Separate focus-ring facility missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.border` | Partial | Native decoration border differs in grammar/defaults/placement; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.shadow` | Partial | Native shadow exists; reference settings/defaults not equivalent; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.tab_indicator` | Missing | Tabbed columns and indicator missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.insert_hint` | Missing | Layout-native insertion hint missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.preset_column_widths` | Divergent | Hardcoded 1/3, 1/2, full vs reference 1/3, 1/2, 2/3; units/empty-list reset missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.default_column_width` | Partial | Default 0.5 agrees; public setting and omitted/empty/client-choice semantics missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.preset_window_heights` | Missing | Height presets and cycling missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.center_focused_column` | Partial | Reveal/fit subset; no public never/always/on-overflow policy; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.always_center_single_column` | Missing | Dedicated setting/policy missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.empty_workspace_above_first` | Missing | Workspace lifecycle missing; workspace override disallows this node; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.default_column_display` | Missing | Public normal/tabbed setting and tabbed display missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.gaps` | Divergent | Shared snap.gap defaults to 12 vs 16, rounds to integer; gap accounting differs; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.struts` | Missing | Signed/fractional custom struts missing; layer usable area is not equivalent; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Layout.background_color` | Partial | Canvas background differs from per-layout solid default; Color grammar audited separately; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.focus_ring` | Missing | Separate focus-ring facility missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.border` | Partial | Native decoration border differs in grammar/defaults/placement; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.shadow` | Partial | Native shadow exists; reference settings/defaults not equivalent; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.tab_indicator` | Missing | Tabbed columns and indicator missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.insert_hint` | Missing | Layout-native insertion hint missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.preset_column_widths` | Divergent | Hardcoded 1/3, 1/2, full vs reference 1/3, 1/2, 2/3; units/empty-list reset missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.default_column_width` | Partial | Default 0.5 agrees; public setting and omitted/empty/client-choice semantics missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.preset_window_heights` | Missing | Height presets and cycling missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.center_focused_column` | Partial | Reveal/fit subset; no public never/always/on-overflow policy; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.always_center_single_column` | Missing | Dedicated setting/policy missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.empty_workspace_above_first` | Missing | Workspace lifecycle missing; workspace override disallows this node; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.default_column_display` | Missing | Public normal/tabbed setting and tabbed display missing; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.gaps` | Divergent | Shared snap.gap defaults to 12 vs 16, rounds to integer; gap accounting differs; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.struts` | Missing | Signed/fractional custom struts missing; layer usable area is not equivalent; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `LayoutPart.background_color` | Partial | Canvas background differs from per-layout solid default; Color grammar audited separately; [public name/default audit](niri-layout-config-audit.md#public-layout-children) |
| `Struts.left` | Missing | No custom struts; [signed numeric/default audit](niri-layout-config-audit.md#size-decoders) |
| `Struts.right` | Missing | No custom struts; [signed numeric/default audit](niri-layout-config-audit.md#size-decoders) |
| `Struts.top` | Missing | No custom struts; [signed numeric/default audit](niri-layout-config-audit.md#size-decoders) |
| `Struts.bottom` | Missing | No custom struts; [signed numeric/default audit](niri-layout-config-audit.md#size-decoders) |

Declared types to check for nested values/custom decoding: `Layout`, `LayoutPart`, `PresetSize`, `DefaultPresetSize`, `Struts`, `CenterFocusedColumn`.

### `appearance.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Color.r` | Partial | RGBA exists; internal float components are not public keys, CSS/numeric KDL grammar differs; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Color.g` | Partial | RGBA exists; internal float components are not public keys, CSS/numeric KDL grammar differs; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Color.b` | Partial | RGBA exists; internal float components are not public keys, CSS/numeric KDL grammar differs; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Color.a` | Partial | RGBA exists; internal float components are not public keys, CSS/numeric KDL grammar differs; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Gradient.from` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Gradient.to` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Gradient.angle` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Gradient.relative_to` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `Gradient.in_` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `GradientInterpolation.color_space` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `GradientInterpolation.hue_interpolation` | Missing | No gradient/interpolation configuration; [decoder/merge audit](niri-appearance-config-audit.md#color-and-gradient-decoding) |
| `CornerRadius.top_left` | Partial | Scalar integer clipping exists; four-corner/fractional grammar and geometry policy differ; [radius/effect audit](niri-effects-config-audit.md#corner-radius-decoding-and-consumers) |
| `CornerRadius.top_right` | Partial | Scalar integer clipping exists; four-corner/fractional grammar and geometry policy differ; [radius/effect audit](niri-effects-config-audit.md#corner-radius-decoding-and-consumers) |
| `CornerRadius.bottom_right` | Partial | Scalar integer clipping exists; four-corner/fractional grammar and geometry policy differ; [radius/effect audit](niri-effects-config-audit.md#corner-radius-decoding-and-consumers) |
| `CornerRadius.bottom_left` | Partial | Scalar integer clipping exists; four-corner/fractional grammar and geometry policy differ; [radius/effect audit](niri-effects-config-audit.md#corner-radius-decoding-and-consumers) |
| `FocusRing.off` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.width` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.active_color` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.inactive_color` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.urgent_color` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.active_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.inactive_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `FocusRing.urgent_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.off` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.width` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.active_color` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.inactive_color` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.urgent_color` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.active_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.inactive_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Border.urgent_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `Shadow.on` | Partial | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.offset` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.softness` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.spread` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.draw_behind_window` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.color` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `Shadow.inactive_color` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowOffset.x` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowOffset.y` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `WorkspaceShadow.off` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadow.offset` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadow.softness` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadow.spread` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadow.color` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.off` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.on` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.offset` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.softness` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.spread` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `WorkspaceShadowPart.color` | Missing | No overview workspace shadow; reference normalizes by logical view height; [radius/effect audit](niri-effects-config-audit.md#workspace-shadows) |
| `TabIndicator.off` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.hide_when_single_tab` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.place_within_column` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.gap` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.width` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.length` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.position` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.gaps_between_tabs` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.corner_radius` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.active_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.inactive_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.urgent_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.active_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.inactive_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicator.urgent_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.off` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.on` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.hide_when_single_tab` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.place_within_column` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.gap` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.width` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.length` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.position` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.gaps_between_tabs` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.corner_radius` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.active_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.inactive_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.urgent_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.active_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.inactive_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorPart.urgent_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorLength.total_proportion` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHint.off` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHint.color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHint.gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHintPart.off` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHintPart.on` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHintPart.color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `InsertHintPart.gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `BorderRule.off` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.on` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.width` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.active_color` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.inactive_color` | Partial | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.urgent_color` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.active_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.inactive_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `BorderRule.urgent_gradient` | Missing | Native borders cover a subset; separate focus ring, urgent colors and gradients missing; [decoder/merge audit](niri-appearance-config-audit.md#border-and-focus-ring-rules) |
| `ShadowRule.off` | Partial | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.on` | Partial | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.offset` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.softness` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.spread` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.draw_behind_window` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.color` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `ShadowRule.inactive_color` | Missing | Native shadow enablement exists; parameter grammar and context-dependent on/off merges differ; [decoder/merge audit](niri-appearance-config-audit.md#shadow-rules) |
| `TabIndicatorRule.active_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorRule.inactive_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorRule.urgent_color` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorRule.active_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorRule.inactive_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `TabIndicatorRule.urgent_gradient` | Missing | No tabbed indicator or insertion-hint implementation; [decoder/merge audit](niri-appearance-config-audit.md#tab-indicator-and-insertion-hint) |
| `Blur.off` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `Blur.passes` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `Blur.offset` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `Blur.noise` | Missing | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `Blur.saturation` | Missing | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.off` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.on` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.passes` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.offset` | Partial | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.noise` | Missing | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BlurPart.saturation` | Missing | Kawase tuning exists with different defaults; reference parse/runtime pass bounds differ; [radius/effect audit](niri-effects-config-audit.md#blur-tuning-and-background-effect-rules) |
| `BackgroundEffectRule.xray` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffectRule.blur` | Divergent | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffectRule.noise` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffectRule.saturation` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffect.xray` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffect.blur` | Divergent | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffect.noise` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |
| `BackgroundEffect.saturation` | Missing | Reference optional blur veto/xray/noise/saturation differs from driftwm rule-or-default frost; [radius/effect audit](niri-effects-config-audit.md#client-regions-popup-rules-and-policy-differences) |

Declared types to check for nested values/custom decoding: `Color`, `Gradient`, `GradientRelativeTo`, `GradientInterpolation`, `GradientColorSpace`, `HueInterpolation`, `CornerRadius`, `FocusRing`, `Border`, `Shadow`, `ShadowOffset`, `WorkspaceShadow`, `WorkspaceShadowPart`, `TabIndicator`, `TabIndicatorPart`, `TabIndicatorLength`, `TabIndicatorPosition`, `InsertHint`, `InsertHintPart`, `BlockOutFrom`, `BorderRule`, `ShadowRule`, `TabIndicatorRule`, `Blur`, `BlurPart`, `BackgroundEffectRule`, `BackgroundEffect`.

### `gestures.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Gestures.dnd_edge_view_scroll` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `Gestures.dnd_edge_workspace_switch` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `Gestures.hot_corners` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `GesturesPart.dnd_edge_view_scroll` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `GesturesPart.dnd_edge_workspace_switch` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `GesturesPart.hot_corners` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `DndEdgeViewScroll.trigger_width` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeViewScroll.delay_ms` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeViewScroll.max_speed` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeViewScrollPart.trigger_width` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeViewScrollPart.delay_ms` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeViewScrollPart.max_speed` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitch.trigger_height` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitch.delay_ms` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitch.max_speed` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitchPart.trigger_height` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitchPart.delay_ms` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `DndEdgeWorkspaceSwitchPart.max_speed` | Missing | Dedicated reference DnD edge view/workspace policy missing; [animation/gesture audit](niri-animation-gesture-audit.md#dnd-edge-consumers) |
| `HotCorners.off` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `HotCorners.top_left` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `HotCorners.top_right` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `HotCorners.bottom_left` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |
| `HotCorners.bottom_right` | Divergent | Configurable actions/threshold differ from one-pixel overview toggle; [animation/gesture audit](niri-animation-gesture-audit.md#hot-corner-consumers) |

Declared types to check for nested values/custom decoding: `Gestures`, `GesturesPart`, `DndEdgeViewScroll`, `DndEdgeViewScrollPart`, `DndEdgeWorkspaceSwitch`, `DndEdgeWorkspaceSwitchPart`, `HotCorners`.

### `animations.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Animations.off` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.slowdown` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.workspace_switch` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.window_open` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.window_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.horizontal_view_movement` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.window_movement` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.window_resize` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.config_notification_open_close` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.exit_confirmation_open_close` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.screenshot_ui_open` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.overview_open_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animations.recent_windows_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.off` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.on` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.slowdown` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.workspace_switch` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.window_open` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.window_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.horizontal_view_movement` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.window_movement` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.window_resize` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.config_notification_open_close` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.exit_confirmation_open_close` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.screenshot_ui_open` | Outside | Unrelated notification/exit-confirmation/screenshot UI motion; shared decoder still audited; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.overview_open_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `AnimationsPart.recent_windows_close` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#animation-defaults-and-merge) |
| `Animation.off` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `Animation.kind` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `EasingParams.duration_ms` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `EasingParams.curve` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `SpringParams.damping_ratio` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `SpringParams.stiffness` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `SpringParams.epsilon` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowOpenAnim.anim` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowOpenAnim.custom_shader` | Missing | Reference open/close/resize shader API missing; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowCloseAnim.anim` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowCloseAnim.custom_shader` | Missing | Reference open/close/resize shader API missing; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowResizeAnim.anim` | Divergent | Frame factors/retention differ from per-event spring/easing and shared clock; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |
| `WindowResizeAnim.custom_shader` | Missing | Reference open/close/resize shader API missing; [animation/gesture audit](niri-animation-gesture-audit.md#custom-animation-decoding) |

Declared types to check for nested values/custom decoding: `Animations`, `AnimationsPart`, `Animation`, `Kind`, `EasingParams`, `Curve`, `SpringParams`, `WorkspaceSwitchAnim`, `WindowOpenAnim`, `WindowCloseAnim`, `HorizontalViewMovementAnim`, `WindowMovementAnim`, `WindowResizeAnim`, `ConfigNotificationOpenCloseAnim`, `ExitConfirmationOpenCloseAnim`, `ScreenshotUiOpenAnim`, `OverviewOpenCloseAnim`, `RecentWindowsCloseAnim`.

### `window_rule.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `WindowRule.matches` | Partial | Ordered app/title matching exists; niri OR match blocks and dynamic states differ; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.excludes` | Missing | No niri exclude blocks; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.default_column_width` | Missing | No rule-level tiled default width with omitted/empty/present semantics; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.default_window_height` | Missing | No rule-level tiled default height; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_on_output` | Partial | Canvas output placement exists; niri target precedence/owner lifecycle differs; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_on_workspace` | Missing | No workspace routing; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_maximized` | Missing | No reference full-width column opening policy; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_maximized_to_edges` | Missing | No reference maximized-to-edges opening policy; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_fullscreen` | Divergent | Fullscreen rules use canvas transitions, not retained tiling topology; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_floating` | Missing | No layout-native floating opening policy; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.open_focused` | Partial | focus_on_open exists; token/smart/strict reference policy not equivalent; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.on_xdg_activate` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.min_width` | Missing | Client constraints exist; rule-level extra constraint missing; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.min_height` | Missing | Client constraints exist; rule-level extra constraint missing; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.max_width` | Missing | Client constraints exist; rule-level extra constraint missing; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.max_height` | Missing | Client constraints exist; rule-level extra constraint missing; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.focus_ring` | Missing | No separate reference focus ring; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.border` | Divergent | Native border rules are stored at mapping, not dynamically recomputed; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.shadow` | Divergent | Native shadow rule flags are stored at mapping; reference merges/reload differ; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.tab_indicator` | Missing | No per-window tab-indicator rule; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.draw_border_with_background` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.opacity` | Divergent | Mapped appearance rule is sticky; reference recomputes metadata/state/reload matches; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.geometry_corner_radius` | Divergent | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `WindowRule.clip_to_geometry` | Missing | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `WindowRule.baba_is_float` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.block_out_from` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.variable_refresh_rate` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.default_column_display` | Missing | No normal/tabbed column default rule; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.default_floating_position` | Missing | Generic canvas position is not reference floating initial placement; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `WindowRule.scroll_factor` | Divergent | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.pinch_sensitivity` | Divergent | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.tiled_state` | Divergent | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `WindowRule.background_effect` | Divergent | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `WindowRule.popups` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `PopupsRule.opacity` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `PopupsRule.geometry_corner_radius` | Missing | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `PopupsRule.background_effect` | Missing | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `ResolvedPopupsRules.opacity` | Missing | Dynamic protocol/render/input/popup policy differs or dedicated rule is missing; [output/window rule audit](niri-output-window-rule-audit.md#remaining-window-scalar-and-popup-consumers) |
| `ResolvedPopupsRules.geometry_corner_radius` | Missing | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `ResolvedPopupsRules.background_effect` | Missing | No equivalent independent geometry/popup/effect rule policy; [radius/effect audit](niri-effects-config-audit.md) |
| `Match.app_id` | Divergent | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.title` | Divergent | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_active` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_focused` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_active_in_column` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_floating` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_window_cast_target` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.is_urgent` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `Match.at_startup` | Missing | Regex/dynamic match properties differ from mapped app/title glob rules; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `FloatingPosition.x` | Missing | Reference stored/default working-area floating placement missing; [output/window rule audit](niri-output-window-rule-audit.md#floating-position) |
| `FloatingPosition.y` | Missing | Reference stored/default working-area floating placement missing; [output/window rule audit](niri-output-window-rule-audit.md#floating-position) |
| `FloatingPosition.relative_to` | Missing | Reference stored/default working-area floating placement missing; [output/window rule audit](niri-output-window-rule-audit.md#floating-position) |

Declared types to check for nested values/custom decoding: `WindowRule`, `PopupsRule`, `ResolvedPopupsRules`, `Match`, `FloatingPosition`, `OnXdgActivate`, `RelativeTo`.

### `workspace.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Workspace.name` | Missing | Reference workspace identity/placement missing; earlier consumer review now linked per row; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `Workspace.open_on_output` | Missing | Reference workspace identity/placement missing; earlier consumer review now linked per row; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `Workspace.layout` | Missing | No per-output/workspace tiling layout hierarchy; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |

Declared types to check for nested values/custom decoding: `Workspace`, `WorkspaceName`, `WorkspaceLayoutPart`.

### `output.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Mode.custom` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Mode.mode` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.clock` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.hdisplay` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.hsync_start` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.hsync_end` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.htotal` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.vdisplay` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.vsync_start` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.vsync_end` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.vtotal` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.hsync_polarity` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Modeline.vsync_polarity` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Output.off` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.name` | Divergent | Exact connector/* lookup differs from ASCII-insensitive connector or metadata identity; [input/output integration audit](niri-input-output-integration-audit.md#output-identity-and-lookup) |
| `Output.scale` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.transform` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.position` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.max_bpc` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Output.mode` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Output.modeline` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Output.variable_refresh_rate` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |
| `Output.focus_at_startup` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.background_color` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.backdrop_color` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.hot_corners` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Output.layout` | Missing | No per-output/workspace tiling layout hierarchy; [precedence/lifecycle audit](niri-config-lifecycle-audit.md) |
| `OutputName.connector` | Divergent | Exact connector/* lookup differs from ASCII-insensitive connector or metadata identity; [input/output integration audit](niri-input-output-integration-audit.md#output-identity-and-lookup) |
| `OutputName.make` | Divergent | Exact connector/* lookup differs from ASCII-insensitive connector or metadata identity; [input/output integration audit](niri-input-output-integration-audit.md#output-identity-and-lookup) |
| `OutputName.model` | Divergent | Exact connector/* lookup differs from ASCII-insensitive connector or metadata identity; [input/output integration audit](niri-input-output-integration-audit.md#output-identity-and-lookup) |
| `OutputName.serial` | Divergent | Exact connector/* lookup differs from ASCII-insensitive connector or metadata identity; [input/output integration audit](niri-input-output-integration-audit.md#output-identity-and-lookup) |
| `Position.x` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Position.y` | Divergent | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#output-defaults-and-runtime-geometry) |
| `Vrr.on_demand` | Unverified | Public decode and selected runtime/fallback consumers traced; independent output/hardware acceptance open; [output/window rule audit](niri-output-window-rule-audit.md#modes-modelines-and-connector-properties) |

Declared types to check for nested values/custom decoding: `Outputs`, `Mode`, `Modeline`, `Output`, `OutputName`, `Position`, `MaxBpc`, `Vrr`.

### `misc.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `SpawnAtStartup.command` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `SpawnShAtStartup.command` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `Cursor.xcursor_theme` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `Cursor.xcursor_size` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `Cursor.hide_when_typing` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `Cursor.hide_after_inactive_ms` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `CursorPart.xcursor_theme` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `CursorPart.xcursor_size` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `CursorPart.hide_when_typing` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `CursorPart.hide_after_inactive_ms` | Unverified | Cursor visibility/theme/timer source traced; protocol/input/capture parity open; [root/misc audit](niri-root-misc-audit.md#cursor-defaults-and-consumers) |
| `HotkeyOverlay.skip_at_startup` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `HotkeyOverlay.hide_not_bound` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `HotkeyOverlayPart.skip_at_startup` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `HotkeyOverlayPart.hide_not_bound` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `ConfigNotification.disable_failed` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `ConfigNotificationPart.disable_failed` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `Clipboard.disable_primary` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `ClipboardPart.disable_primary` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `Overview.zoom` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `Overview.backdrop_color` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `Overview.workspace_shadow` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `OverviewPart.zoom` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `OverviewPart.backdrop_color` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `OverviewPart.workspace_shadow` | Missing | No reference overview; runtime zoom clamps separately from decode and canvas zoom is not equivalent; [layer/overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state) |
| `EnvironmentVariable.name` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `EnvironmentVariable.value` | Outside | Concrete process/selection/help/error-UI exclusion; layout-visible consequences retained; [root/misc audit](niri-root-misc-audit.md#remaining-exclusions-and-workspace-row-links) |
| `XwaylandSatellite.off` | Divergent | Eager satellite launch differs from reference on-demand sockets and reload policy; [root/misc audit](niri-root-misc-audit.md#decoration-preference-and-satellite-lifecycle) |
| `XwaylandSatellite.path` | Divergent | Eager satellite launch differs from reference on-demand sockets and reload policy; [root/misc audit](niri-root-misc-audit.md#decoration-preference-and-satellite-lifecycle) |
| `XwaylandSatellitePart.off` | Divergent | Eager satellite launch differs from reference on-demand sockets and reload policy; [root/misc audit](niri-root-misc-audit.md#decoration-preference-and-satellite-lifecycle) |
| `XwaylandSatellitePart.on` | Divergent | Eager satellite launch differs from reference on-demand sockets and reload policy; [root/misc audit](niri-root-misc-audit.md#decoration-preference-and-satellite-lifecycle) |
| `XwaylandSatellitePart.path` | Divergent | Eager satellite launch differs from reference on-demand sockets and reload policy; [root/misc audit](niri-root-misc-audit.md#decoration-preference-and-satellite-lifecycle) |

Declared types to check for nested values/custom decoding: `SpawnAtStartup`, `SpawnShAtStartup`, `Cursor`, `CursorPart`, `ScreenshotPath`, `HotkeyOverlay`, `HotkeyOverlayPart`, `ConfigNotification`, `ConfigNotificationPart`, `Clipboard`, `ClipboardPart`, `Overview`, `OverviewPart`, `Environment`, `EnvironmentVariable`, `XwaylandSatellite`, `XwaylandSatellitePart`.

### `layer_rule.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `LayerRule.matches` | Partial | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.excludes` | Missing | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.opacity` | Partial | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.block_out_from` | Missing | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.shadow` | Partial | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.geometry_corner_radius` | Divergent | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.place_within_backdrop` | Missing | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.baba_is_float` | Missing | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.background_effect` | Divergent | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `LayerRule.popups` | Missing | Generic layer chrome/effects cover a subset; dedicated matching/backdrop/popup policy differs; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `Match.namespace` | Divergent | Generic namespace-as-app-id patterns lack reference regex/startup/current-layer predicate semantics; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `Match.at_startup` | Missing | Generic namespace-as-app-id patterns lack reference regex/startup/current-layer predicate semantics; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |
| `Match.layer` | Missing | Generic namespace-as-app-id patterns lack reference regex/startup/current-layer predicate semantics; [layer/overview audit](niri-layer-overview-audit.md#layer-matching-defaults-and-local-merge) |

Declared types to check for nested values/custom decoding: `LayerRule`, `Match`.

### `input.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Input.keyboard` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.touchpad` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.mouse` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.trackpoint` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.trackball` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.tablet` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.touch` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `Input.disable_power_key_handling` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Input.warp_mouse_to_focus` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#cursor-warping) |
| `Input.focus_follows_mouse` | Divergent | Boolean hover policy lacks reference threshold/transition semantics; [input focus audit](niri-input-focus-audit.md#focus-follows-mouse) |
| `Input.workspace_auto_back_and_forth` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#workspace-back-and-forth) |
| `Input.mod_key` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Input.mod_key_nested` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `InputPart.keyboard` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.touchpad` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.mouse` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.trackpoint` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.trackball` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.tablet` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.touch` | Unverified | Keyboard field merge versus whole-device replacement; integration acceptance pending; [device input audit](niri-device-input-audit.md#assembly-and-scalar-grammar) |
| `InputPart.disable_power_key_handling` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `InputPart.warp_mouse_to_focus` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#cursor-warping) |
| `InputPart.focus_follows_mouse` | Divergent | Boolean hover policy lacks reference threshold/transition semantics; [input focus audit](niri-input-focus-audit.md#focus-follows-mouse) |
| `InputPart.workspace_auto_back_and_forth` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#workspace-back-and-forth) |
| `InputPart.mod_key` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `InputPart.mod_key_nested` | Unverified | Backend Mod selection/power fallback source traced; public binding acceptance pending; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Keyboard.xkb` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Keyboard.repeat_delay` | Divergent | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Keyboard.repeat_rate` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Keyboard.track_layout` | Missing | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Keyboard.numlock` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `KeyboardPart.xkb` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `KeyboardPart.repeat_delay` | Divergent | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `KeyboardPart.repeat_rate` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `KeyboardPart.track_layout` | Missing | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `KeyboardPart.numlock` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.rules` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.model` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.layout` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.variant` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.options` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `Xkb.file` | Unverified | Keyboard/keymap/default/reload source traced; complete focus and protocol parity remains open; [device input audit](niri-device-input-audit.md#keyboard-configuration-and-consumers) |
| `ScrollFactor.base` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `ScrollFactor.horizontal` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `ScrollFactor.vertical` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `Touchpad.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.tap` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.dwt` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.dwtp` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.drag` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.drag_lock` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.natural_scroll` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.click_method` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.accel_speed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.accel_profile` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.scroll_method` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.scroll_button` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.scroll_button_lock` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.tap_button_map` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.left_handed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.disabled_on_external_mouse` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.middle_emulation` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touchpad.scroll_factor` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `Touchpad.pinch_sensitivity` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `Mouse.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.natural_scroll` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.accel_speed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.accel_profile` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.scroll_method` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.scroll_button` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.scroll_button_lock` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.left_handed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.middle_emulation` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Mouse.scroll_factor` | Divergent | Client delivery factors/exponentiation differ from canvas navigation controls; [input/output integration audit](niri-input-output-integration-audit.md#client-scroll-and-pinch) |
| `Trackpoint.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.natural_scroll` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.accel_speed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.accel_profile` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.scroll_method` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.scroll_button` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.scroll_button_lock` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.left_handed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackpoint.middle_emulation` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.natural_scroll` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.accel_speed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.accel_profile` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.scroll_method` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.scroll_button` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.scroll_button_lock` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.left_handed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Trackball.middle_emulation` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Tablet.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Tablet.calibration_matrix` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Tablet.map_to_output` | Unverified | Target precedence/geometry source traced; hardware and focus parity pending; [device input audit](niri-device-input-audit.md#tablet-and-touch-target-policy) |
| `Tablet.map_to_focused_output` | Unverified | Target precedence/geometry source traced; hardware and focus parity pending; [device input audit](niri-device-input-audit.md#tablet-and-touch-target-policy) |
| `Tablet.map_to_focused_window` | Unverified | Target precedence/geometry source traced; hardware and focus parity pending; [device input audit](niri-device-input-audit.md#tablet-and-touch-target-policy) |
| `Tablet.left_handed` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touch.off` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touch.calibration_matrix` | Unverified | Device classification/default/setter source traced; actual support and parity not proven; [device input audit](niri-device-input-audit.md#libinput-application-and-classification) |
| `Touch.map_to_output` | Unverified | Target precedence/geometry source traced; hardware and focus parity pending; [device input audit](niri-device-input-audit.md#tablet-and-touch-target-policy) |
| `FocusFollowsMouse.max_scroll_amount` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#focus-follows-mouse) |
| `WarpMouseToFocus.mode` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#cursor-warping) |

Declared types to check for nested values/custom decoding: `Input`, `InputPart`, `Keyboard`, `KeyboardPart`, `Xkb`, `TrackLayout`, `ScrollFactor`, `Touchpad`, `Mouse`, `Trackpoint`, `Trackball`, `ClickMethod`, `AccelProfile`, `ScrollMethod`, `TapButtonMap`, `Tablet`, `Touch`, `FocusFollowsMouse`, `WarpMouseToFocus`, `WarpMouseToFocusMode`, `ModKey`.

### `debug.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Debug.preview_render` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.dbus_interfaces_in_non_session_instances` | Outside | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.wait_for_frame_completion_before_queueing` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.enable_overlay_planes` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_cursor_plane` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_direct_scanout` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.restrict_primary_scanout_to_matching_format` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.force_disable_connectors_on_resume` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.render_drm_device` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.ignored_drm_devices` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.force_pipewire_invalid_modifier` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_pipewire_dmabuf` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.emulate_zero_presentation_time` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_resize_throttling` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `Debug.disable_transactions` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `Debug.keep_laptop_panel_on_when_lid_is_closed` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_monitor_names` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.strict_new_window_focus_policy` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `Debug.honor_xdg_activation_with_invalid_serial` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `Debug.deactivate_unfocused_windows` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `Debug.skip_cursor_only_updates_during_vrr` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `Debug.disable_10bit_output` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.preview_render` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.dbus_interfaces_in_non_session_instances` | Outside | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.wait_for_frame_completion_before_queueing` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.enable_overlay_planes` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_cursor_plane` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_direct_scanout` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.restrict_primary_scanout_to_matching_format` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.force_disable_connectors_on_resume` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.render_drm_device` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.ignored_drm_devices` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.force_pipewire_invalid_modifier` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_pipewire_dmabuf` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.emulate_zero_presentation_time` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_resize_throttling` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `DebugPart.disable_transactions` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `DebugPart.keep_laptop_panel_on_when_lid_is_closed` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_monitor_names` | Missing | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.strict_new_window_focus_policy` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `DebugPart.honor_xdg_activation_with_invalid_serial` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `DebugPart.deactivate_unfocused_windows` | Missing | Dedicated reference policy/setting missing; existing integration is not equivalence; [input focus audit](niri-input-focus-audit.md#layout-debug-flags) |
| `DebugPart.skip_cursor_only_updates_during_vrr` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |
| `DebugPart.disable_10bit_output` | Unverified | Field-specific backend/capture/topology scope recorded; retained acceptance is open; [input/output integration audit](niri-input-output-integration-audit.md#remaining-debug-scope) |

Declared types to check for nested values/custom decoding: `Debug`, `DebugPart`, `PreviewRender`.

### `binds.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `Bind.key` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.action` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.repeat` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.cooldown` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.allow_when_locked` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.allow_inhibiting` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Bind.hotkey_overlay_title` | Outside | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Key.trigger` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `Key.modifiers` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `SwitchBinds.lid_open` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `SwitchBinds.lid_close` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `SwitchBinds.tablet_mode_on` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `SwitchBinds.tablet_mode_off` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |
| `SwitchAction.spawn` | Unverified | Custom binding decode/merge and selected consumers traced; runtime acceptance remains open; [bind/address audit](niri-binding-address-audit.md#binding-grammar-and-assembly) |

Declared types to check for nested values/custom decoding: `Binds`, `Bind`, `Key`, `Trigger`, `SwitchBinds`, `SwitchAction`, `Action`, `WorkspaceReference`.

### `recent_windows.rs`

| Schema field | Status | Mapping / next audit |
| --- | --- | --- |
| `RecentWindows.on` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindows.debounce_ms` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindows.open_delay_ms` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindows.highlight` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindows.previews` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindows.binds` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.on` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.off` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.debounce_ms` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.open_delay_ms` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.highlight` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.previews` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `RecentWindowsPart.binds` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlight.active_color` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlight.urgent_color` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlight.padding` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlight.corner_radius` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlightPart.active_color` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlightPart.urgent_color` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlightPart.padding` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruHighlightPart.corner_radius` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruPreviews.max_height` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruPreviews.max_scale` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruPreviewsPart.max_height` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruPreviewsPart.max_scale` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#defaults-and-merge) |
| `MruBind.key` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#mru-binding-and-enum-grammar) |
| `MruBind.action` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#mru-binding-and-enum-grammar) |
| `MruBind.allow_inhibiting` | Missing | Reference MRU model/UI/settings missing; [recent-windows audit](niri-recent-windows-audit.md#mru-binding-and-enum-grammar) |
| `MruBind.hotkey_overlay_title` | Outside | Help-title presentation outside scope; MRU binding integration retained; [recent-windows audit](niri-recent-windows-audit.md#mru-binding-and-enum-grammar) |

Declared types to check for nested values/custom decoding: `RecentWindows`, `RecentWindowsPart`, `MruHighlight`, `MruHighlightPart`, `MruPreviews`, `MruPreviewsPart`, `MruBind`, `MruDirection`, `MruScope`, `MruFilter`, `MruAction`, `MruBinds`.

Named schema fields indexed: **576**.

## Behavioral differences to resolve

- Consume pulls into the active column in the reference; our move action sends the focused tile to a neighbor.
- Distinct first-consume/last-expel actions exist; complete boundary, ID-targeted, tab and view behavior remains unaudited.
- Reference width presets include 2/3, proportional widths account for gaps and fixed widths have specific content/frame units.
- Automatic tile heights use reference weighting semantics; equal content heights are not equivalent with mixed decorations.
- Current fullscreen, floating exclusion, generic momentum and one-output fallback are not parity implementations.
- Structural reorder now exists, but reference movement animation, fullscreen/tab context and complete view policy remain pending.
