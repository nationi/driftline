# Pinned niri remaining action integration and exclusions

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds the final 32 scoped action links: all 186 Action variants now link an explicit
source-consumer or scope review. Eight actions have bounded Outside decisions;
the other 24 retain integration acceptance. This closes source-action triage,
not the behavioral audit, custom argument grammar or executable acceptance.
Named-schema review count remains 496/576. No driftwm runtime/config change or
niri/client/capture/hardware execution is claimed.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [niri-config/src/recent_windows.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/recent_windows.rs)
- [niri-ipc/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/ui/mru.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/ui/mru.rs)

Earlier [binding/address](niri-binding-address-audit.md),
[device input](niri-device-input-audit.md),
[workspace/cast](niri-workspace-policy-action-audit.md) and
[effects](niri-effects-config-audit.md) audits retain their limits. MRU configuration
and every remaining misc/root decoder still need their dedicated source review.

## Field-specific action scope

| Action | Decision | Concrete scope / retained gate |
| --- | --- | --- |
| `Quit` | Outside | Exit/confirmation command UX outside tiling; window teardown acceptance retained |
| `ChangeVt` | Integration / Unverified | VT/session transition clears suppressed keys and repeat; integration acceptance open |
| `Suspend` | Integration / Unverified | Session transition clears suppressed keys and repeat; resume/lifecycle acceptance open |
| `PowerOffMonitors` | Integration / Unverified | Monitor-active state/backend output power; ownership and resume acceptance open |
| `PowerOnMonitors` | Integration / Unverified | Monitor-active state/backend output power and redraw; recovery acceptance open |
| `ToggleDebugTint` | Outside | Diagnostic tint overlay outside visual parity; ordinary rendering retained |
| `DebugToggleOpaqueRegions` | Outside | Diagnostic region overlay outside visual parity; opacity/occlusion acceptance retained |
| `DebugToggleDamage` | Outside | Diagnostic damage overlay outside visual parity; damage/render correctness retained |
| `ToggleTouchpad` | Integration / Unverified | Runtime toggle reapplies touchpad settings; device/input acceptance open |
| `Spawn` | Outside | Program launch outside tiling; activation and mapped-window policy retained |
| `SpawnSh` | Outside | Shell launch outside tiling; activation and mapped-window policy retained |
| `DoScreenTransition` | Integration / Unverified | Target-specific screen snapshots and transition; rendering/privacy acceptance open |
| `ConfirmScreenshot` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `CancelScreenshot` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `ScreenshotTogglePointer` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `Screenshot` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `ScreenshotScreen` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `ScreenshotWindow` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `ScreenshotWindowById` | Integration / Unverified | Capture UI/output/window target and privacy/render/input context; acceptance open |
| `ToggleKeyboardShortcutsInhibit` | Integration / Unverified | Focused surface inhibitor activation; keyboard resolution acceptance open |
| `SwitchLayout` | Integration / Unverified | XKB next/prev/index switch; keyboard/focus acceptance open |
| `ShowHotkeyOverlay` | Outside | Shortcut-help presentation outside tiling; binding resolution retained |
| `LoadConfigFile` | Integration / Unverified | Watcher-selected config reload; layout/rule reload acceptance open |
| `MruAdvance` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruConfirm` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruCancel` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruCloseCurrentWindow` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruFirst` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruLast` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruSetScope` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `MruCycleScope` | Integration / Unverified | MRU selection/scope/confirm/cancel state; independent focus/view acceptance open |
| `TestAction` | Outside | Test-only dispatcher counter, no production layout policy |

Outside excludes the named command/presentation/test facility, not all downstream
window lifetime, keyboard, activation or rendering consequences. Tiling must still
handle clients launched externally and teardown/session changes correctly. Do not
infer an IPC whitelist or public KDL availability from these rows: the earlier
inventory exposure table distinguishes 136 derived variants from 50 skipped ones.

## Common dispatch, session and input consequences

The reviewed do_action checks lock eligibility before dispatch and cancels active
touch interaction. Its built-in locked-action list includes ChangeVt, Suspend,
monitor power, SwitchLayout, ToggleKeyboardShortcutsInhibit and ToggleTouchpad;
spawn allowance has its separate per-bind policy. This is not a promise that
all actions can run from every key/IPC/UI context.

ChangeVt and Suspend call the backend, clear suppressed-key state and remove an
ongoing repeat timer because releases may not arrive. They therefore retain
integration scope despite not changing a column directly. Power actions set an
idempotent monitors_active flag and call backend.set_monitors_active; power-on
also queues redraw. Power is not synonymous with output removal or workspace
migration; real suspend/resume/DRM recovery needs hardware acceptance.

ToggleTouchpad flips runtime state and reapplies settings to devices with tap
finger count >0. Reload and off/external-mouse precedence retain the input audit.
ToggleKeyboardShortcutsInhibit finds the current keyboard-focus surface's
inhibitor and activates/inactivates it; none means no inhibitor action. The
binding decoder forces this toggle to be uninhibitable so it can be used again.

SwitchLayout is XKB layout selection, not a tiling-layout switch. next/prev cycle
keymap layouts; an explicit index is zero-based and checked against layout count,
warning instead of switching when absent. Its public string scalar uses next,
prev or a numeric index. It must not inherit one-based window/workspace index
conversion. Keymap/focus/virtual-keyboard acceptance remains open.

LoadConfigFile forwards path to an existing config watcher; without a watcher it
does nothing. It is not proof a new config parsed/applied successfully. The root
reload/error/include audit and in-flight layout acceptance remain separate work.
Spawn/SpawnSh create external activation tokens before launch; launch UX itself
is Outside, while the already reviewed map/activation policy remains in scope.

## Capture UI, targets and transitions

Screenshot opens interactive selection (when unlocked, not already open and a
default pointer/active output exists), captures output images, then cancels
pointing grabs before opening UI. Dispatch also cancels MRU. Confirm delegates
to UI capture/write behavior; Cancel returns if closed, otherwise closes UI,
restores cursor and redraws. ScreenshotTogglePointer toggles selection pointer
state. Exact capture bounds, cursor pixels, locks and UI-owned directional action
branches still need independent fixtures; image filename/path presentation alone
remains the earlier scoped exclusion.

ScreenshotScreen captures active output; ScreenshotWindow uses layout focus and
its output association; ScreenshotWindowById resolves a mapped window with a
monitor before capture. Missing targets do not fall back to another arbitrary
window. Output and window capture use ScreenCapture render target, retaining
block-out/effect privacy rules, scale/transform and popup/chrome acceptance.
Window capture also considers fullscreen/ignored-rule opacity, so generic canvas
screenshots are not automatically equivalent.

DoScreenTransition refreshes render elements and captures separate Output,
Screencast and ScreenCapture textures per output without pointer; if any target
capture fails, that output is omitted from the collected transition snapshots.
It then uses default or requested delay. It is not a single screenshot reused
across privacy targets. Full transition timing, capture failure recovery and
output lifecycle remain integration gates, not proven by reading dispatch.

## MRU open, selection and close

MruAdvance while closed requires recent-windows enabled, a nonempty WindowMru
model and active output. It commits pending keyboard MRU timestamp first,
chooses requested or remembered scope, applies optional filter and opens on the
active output. Forward opening keeps the first candidate if no layout window is
focused; otherwise it advances. While already open, advance uses direction and
optional filter, not the action's scope argument; MruSetScope/CycleScope changes
scope separately. It does not behave like repeated FocusWindowPrevious.

MruFirst/Last/SetScope/CycleScope and CloseCurrentWindow act only while open.
First/last/scope operations release frozen pointer-driven view state; close
requests the selected toplevel to close, without guaranteeing immediate removal.
Closing the last candidate can cancel the UI through its window-removal path.
Full ordering, filter fallback, default MRU bindings, highlight/preview grammar,
reload and hotplug remain the dedicated recent-windows audit and fixtures.

MruConfirm closes with Confirm, resolves a still-live selected ID, forces keyboard
focus update after the MRU UI closes, then focus_window applies output/cursor
policy. MruCancel closes with Cancel without that focus_window dispatch. UI
closing animation/state is distinct from open status; no assumption of immediate
resource destruction follows. Open delay uses unadjusted clock time. Focus history
uses debounce/pending keyboard commit logic; an input event can commit earlier
than the timer. Exact timing, modifier release, focus churn and output changes
require deterministic protocol/input fixtures.

## Explicit exclusions and scope corrections

Quit confirmation/exit UX, command launch, help presentation, diagnostic overlays
and TestAction's cfg(test) counter are the eight bounded Outside decisions.
Their code paths were located; no processes, session transitions, screenshots or
messages were triggered during this source review.

Sixteen formerly generic Outside actions are now Integration/Unverified: session/
power/input controls, screen transition/capture, keyboard inhibit/layout switch
and config reload. MRU's eight existing Unverified integration statuses stay.
The earlier seven opacity/cast corrections remain separate historical changes.
No row becomes Exact or implemented through this classification.

## Continuation and acceptance

All 186 action rows now have source-scope evidence, but many consumers remain
Unverified/Partial/Missing/Divergent. Remaining MRU/misc/root argument schemas,
independent reference geometry/operation traces beyond zero gaps, parser error
fixtures and full input/client/animation/capture/hardware contexts still gate
A0/P0 and feature expansion. Source coverage is not a compatibility percentage.
Next A0: recent-windows config/model and remaining misc/root integration decoders,
then independent executable fixtures and explicit retained-scope decisions.
