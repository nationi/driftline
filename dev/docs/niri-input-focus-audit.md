# Pinned niri input focus and layout debug policy

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source audit
covers 18 indexed fields: three policies in both Input/InputPart, two nested
property fields, and five flags in both Debug/DebugPart. Other device/backend
fields remain unverified. This is source evidence, not execution of niri,
accepted parser fixtures, complete input parity or a runtime change to driftwm.

## Pinned sources

- [input.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/input.rs): Input defaults/merge, FocusFollowsMouse and WarpMouseToFocus grammar.
- [utils.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils.rs): Flag and Percent scalar decoding.
- [debug.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/debug.rs): debug defaults and part merge.
- [niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs): handle_focus_follows_mouse, cursor centering and focus_window.
- [input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs): relative/absolute motion and FocusWorkspace dispatch.
- [layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs): workspace animation focus eligibility, scroll query and options propagation.
- [monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs): previous workspace identity and switching.
- [scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs): normalized scroll distance, column throttling and activated hints.
- [floating.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/floating.rs) and [workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs): floating scroll distance, refresh and focus context.
- [handlers/compositor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/compositor.rs): mapping activation precedence.
- [handlers/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/mod.rs): activation token validation, timeout and urgency.
- [handlers/xdg_shell.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/xdg_shell.rs): transaction pre-commit blocking.

## Input defaults and replacement

Input defaults leave focus-follows-mouse and warp-mouse-to-focus absent (disabled),
and workspace-auto-back-and-forth false. The first two are child nodes containing
optional properties, not booleans. An empty node enables its facility with unset
properties. InputPart omission preserves an earlier value; a present node replaces
the entire optional value through merge_clone_opt, rather than merging properties.
Thus a later empty node clears an earlier max-scroll-amount or mode. The workspace
flag uses Flag: no argument means true, an explicit boolean may set false.
A fresh config reload resolves defaults/parts again; merge preservation within
that process is not permission to retain removed settings from the old config.

FocusFollowsMouse has optional string property max-scroll-amount. Percent requires
a trailing percent sign with nothing after it and parses f64 divided by 100.
This scalar implementation has no explicit finite or 0–100 range validation;
do not infer such a restriction from the option name. Runtime comparison is a
strict greater-than test. WarpMouseToFocus mode accepts exactly center-xy and
center-xy-always; absence is the separate-axis policy below.

## Focus follows mouse

Relative and absolute motion call handle_focus_follows_mouse before recording
new pointer contents. It recomputes old contents at the current pointer location,
rather than relying on the cached hit during animations. It skips pointer grabs
and the open MRU UI. Moving onto another output can focus that output, including
when no window is hit. Window activation requires a changed window/hit target,
closed overview, a non-tab-indicator hit and layout eligibility. During a normal
workspace animation only the target workspace qualifies; an interactive workspace
gesture and the actively moved tile are exceptions in that eligibility helper.
Activation does not raise the window and clears on-demand layer focus. Crossing
onto a layer instead records an on-demand layer candidate; later keyboard focus
policy still decides whether that candidate can receive focus.

The optional threshold compares scroll_amount_to_activate against Percent. For
another tiled column, the amount is the absolute difference between its required
view offset and the current animation target offset, divided by working-area
width. It is not pointer travel or a distance in pixels. The already active
column, a floating tile and the actively moved tile return zero. Equality passes.
Full policy depends on view fitting/centering and must receive reference fixtures.

## Cursor warping

Warping to the focused tile requires keyboard focus on the layout, no tablet
cursor interaction, an active output and an active-window visual rectangle. The
rectangle is translated into global output coordinates. Overview can suppress
that rectangle, as recorded in the [overview audit](niri-layer-overview-audit.md#overview-defaults-runtime-zoom-and-state).

| Mode | Pointer already inside both axes | Pointer outside one axis |
| --- | --- | --- |
| Unset / separate axes | No move | Center only the out-of-bounds axis |
| center-xy | No move | Center both axes |
| center-xy-always | Center both axes | Center both axes |

The rectangle test includes both edges. The centered helper promotes unset mode
to center-xy, retaining center-xy-always. focus_window uses this helper when the
active output changes and falls back to moving onto that output if it returns
false. Same-output focus uses the ordinary helper. Programmatic cursor movement
does not itself show a hidden pointer. These helpers/call sites do not establish
all pointer-constraint, lock, tablet, MRU or multi-output action permutations.

## Workspace back and forth

FocusWorkspace dispatch applies workspace-auto-back-and-forth on the current
output. A resolved different output instead focuses that output and switches
directly, then uses centered cursor warp/output fallback. Up/down and explicit
previous actions have separate branches; this option does not universally alter
all workspace movement. The monitor clamps the requested index before comparison.
If it equals the active index, a still-resolvable previous workspace is selected;
with no previous workspace it does nothing. Otherwise it selects the request.
Previous is stored by workspace identity, not a stale ordinal. Changing selection
records the old identity; activating the same index does not replace it.
Full named/ID/index resolution and removal/reorder behavior remain action audit work.

## Layout debug flags

All five audited debug booleans default false. Their Part fields use Flag;
omission preserves earlier parts and explicit false can override true. The layout
copies transaction/throttling/deactivation options through Options::from_config
and update_config. Map-time and token-creation checks read current config directly;
a flag change is not evidence of retroactively revisiting an earlier decision.

- disable-transactions bypasses pending transaction commit blocking in the mapped
  pre-commit path, substitutes completed close-animation blockers, and makes
  scrolling resize throttling individual. It does not imply all DMA-BUF readiness
  or every protocol blocker is disabled.
- disable-resize-throttling selects CanSend in floating refresh. In scrolling,
  combined throttling normally folds all column configure intents: ShouldSend
  wins and Throttled can prevent a merely CanSend group. Transactions-disabled
  or tabbed columns use individual throttling. Importantly, the later per-window
  branch then reads win.configure_intent() again, overriding the earlier CanSend
  selection even when disable-resize-throttling is true. Preserve this source
  edge for execution fixtures; do not describe the flag as an unconditional
  throttle bypass in all modes.
- strict-new-window-focus-policy changes the map-time fallback from Smart to No.
  An explicit open-focused rule wins first; otherwise a still-fresh activation
  token wins before strict policy. Smart itself is context-dependent, not Yes.
- honor-xdg-activation-with-invalid-serial permits a serial-bearing token after
  its seat resolves, bypassing keyboard/pointer last-enter freshness checks.
  Without a serial, token creation already marks urgency-only and accepts it.
  The flag does not remove the ten-second request timeout. Mapped requests obey
  on-xdg-activate ignore/urgent/focus; default urgency-only tokens mark urgent,
  other accepted tokens activate. Unmapped requests retain token data, and mapping
  rechecks its age. The presented token is removed after request processing.
- deactivate-unfocused-windows additionally requires keyboard/layout focus for
  activated hints. In scrolling it restricts activation to the selected tile;
  without it, all tabs in an active tabbed column may advertise activated. Floating
  refresh applies focus gating to its active tile. Activated protocol hints and
  actual keyboard focus are distinct state.

## Driftwm comparison and acceptance probes

Driftwm exposes a boolean focus_follows_mouse, default false. Its
src/input/mod.rs::maybe_hover_focus skips grabs/layers/widgets, handles fullscreen
and pinned priority, and deliberately supports camera-driven hover pulls. It has
no niri max-scroll-amount/workspace-transition policy. Existing warp_pointer and
camera navigation do not provide the audited public cursor-warp modes. Tiling
workspaces/back-and-forth and the five corresponding debug settings are missing.
Existing bounded resize groups do not prove niri configure-throttling parity.
Driftwm's activation handler accepts token creation, normally requires a serial
for activation, special-cases relaunch adoption and active self-activation, and
does not implement niri's audited validation/urgency/timeout policy.

Independent executable probes still required:

1. Omitted, empty and repeated optional nodes; explicit false flags; accepted and
   rejected mode/Percent inputs, including nonfinite/negative boundary cases.
2. Same-column versus other-column hover with threshold below/equal/above the
   required normalized scroll, nonzero gaps, in-flight view targets, floating
   windows, tabs, overview, MRU, grabs and workspace animation/gesture.
3. Cursor inside/on/outside each rectangle edge for all modes, same/different
   output, keyboard focus elsewhere, tablet use and constrained/hidden pointer.
4. Same workspace repeated, missing previous identity, reorder/removal and
   cross-output named/index requests, separately from up/down/previous actions.
5. Combined/individual/tabbed throttling with both flags in every combination;
   stalled peer, completed transaction and independent DMA-BUF readiness.
6. Rule/token/strict precedence, expired and urgency-only tokens, invalid serial,
   unfocused/tabbed activated hints and config reload at each decision boundary.

These are acceptance requirements, not new test results. Next A0 work is
animation/gesture configuration, remaining device/backend scope and naming/rule
fields, public binding/action addressing and executable reference fixtures.
