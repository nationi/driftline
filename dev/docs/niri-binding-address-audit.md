# Pinned niri bindings and scoped action addressing

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
covers all 14 named binds-module fields, action config exposure and 12 selected
addressing consumers. Action arguments are enum payloads and need review beyond
the named-field index. This is not parser execution, complete action audit or
implemented behavior parity; no driftwm runtime/public config changes are made.

## Pinned sources

- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [niri-config/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs)
- [niri-ipc/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-ipc/src/lib.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs)
- [src/layout/scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs)
- [src/layout/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs)
- [src/layout/monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs)

## Binding grammar and assembly

Binds expects children. Each child node name parses as Key, takes no arguments
or type annotation, and has one action child. A missing/invalid action emits an
error but can return a dummy empty Spawn bind to keep duplicate-key diagnostics
working. That dummy does not mean the invalid config is accepted. Additional
action children are diagnosed. Duplicate normalized keys inside one Binds block
are diagnosed; root assembly of a later binds block instead removes existing
conflicting keys and appends new ones. This supports include/override ordering;
empty later blocks do not clear all previous binds.

Key splits on +; the final component is the trigger. Modifier components are
trimmed, ASCII-insensitive and ORed: mod, ctrl/control, shift, alt, super/win,
iso_level3_shift/mod5 and iso_level5_shift/mod3. mod is a symbolic compositor bit,
resolved against backend Mod policy, not another literal Super alias. Repeating
a modifier does not add another bit. The final trigger is not trimmed by this
parser. MouseLeft/Right/Middle/Back/Forward, WheelScroll directions, TouchpadScroll
directions and TabletStylusButton1/2/3 are recognized ASCII-insensitively. Other
triggers delegate to xkb keysym lookup, normally case-insensitive, with a specific
XF86_Screensaver/ScreenSaver case-resolution exception. Unknown modifiers/keysyms
fail. This source grammar is not a complete keymap/runtime matching proof.

| Bind field/property | Default and source policy |
| --- | --- |
| key, action | Required node-name key and single child action; no named property equivalents |
| repeat | Boolean property, default true |
| cooldown / cooldown-ms | Optional u64 milliseconds converted to Duration; absent means no cooldown |
| allow-when-locked | Boolean, default false; merely specifying it, including false, is rejected for actions other than Spawn/SpawnSh |
| allow-inhibiting | Boolean, default true; ToggleKeyboardShortcutsInhibit forces false |
| hotkey-overlay-title | Absent / explicit null / string represented by Option<Option<String>>; unrelated help presentation is Outside |

Compositor key repeat uses keyboard repeat delay/rate; repeat=false or rate=0
suppresses starting that timer. handle_bind cooldown is keyed by Key, not action;
a suppressed call does nothing, and the timer removes the key when it expires.
Lock eligibility is checked before arming cooldown. do_action separately gates
locked actions with explicit allowance or its built-in allowed_when_locked list.
The input inhibition filter forwards an otherwise matched press if shortcuts are
inhibited and allow-inhibiting is true; a previously suppressed release remains
suppressed. This is selected policy evidence, not all lock/grab/MRU/keyboard paths.

SwitchBinds has optional lid-open/lid-close/tablet-mode-on/tablet-mode-off nodes,
merged by optional replacement. Each SwitchAction requires a spawn child containing
its argument vector. These command hooks remain Unverified integration here;
reviewing their shape does not prove ordering with lid/output topology policy.
No commands or messages were sent as part of this audit.

## Config exposure versus IPC

The pinned Action enum has 186 source variants; 50 carry knuffel(skip), leaving
136 variants handled by the derived config decoder. The generated inventory now
lists the skipped set from source. Derived does not mean its complete argument
shape or every consumer was audited. The skipped set is not exclusively ID-based
and is not equivalent to an IPC whitelist: it also includes internal screenshot,
MRU and test actions. Do not expose all Rust variants as config children.

From<niri_ipc::Action> translates public IPC actions to config/runtime variants;
for example an optional close ID selects CloseWindow versus CloseWindowById,
and IPC FocusWindow maps to the skipped FocusWindow variant. Existing window IDs
are stable runtime identifiers, not column/window ordinals. Unknown mapped IDs
in the reviewed FocusWindow/CloseWindowById dispatch branches yield no action.
Their existence does not prove every ID action has the same fallback or side effects.

## Scoped ordinal and previous selection

| Reviewed action(s) | Selected source behavior |
| --- | --- |
| FocusColumnFirst / Last | On tiling, select first/last column; last guards empty layout. Workspace dispatch instead uses leftmost/rightmost when floating is active. |
| FocusColumn | usize argument; scrolling converts via saturating_sub(1), clamps to last and guards empty columns. Workspace dispatch switches from floating to tiling first. |
| MoveColumnToIndex | usize argument; scrolling uses the same saturating one-based conversion/clamp and no-ops for an unchanged index. Reorder has its own view/resize state handling. |
| FocusWindowInColumn | u8 argument; focus_index uses saturating_sub(1) clamped to the last tile. Workspace dispatch returns while floating is active. |
| FocusColumnRightOrFirst / LeftOrLast | Workspace tries directional focus then first/last fallback. Ordinary left/right do not gain wrapping implicitly. Floating first/last behavior differs from tiling. |
| FocusWindow | Finds mapped ID and uses focus_window, including output activation/cursor policy; absent ID does nothing. |
| CloseWindowById | Finds the mapped ID before requesting close; absent ID does nothing. It is not a force-kill guarantee. |
| FocusWindowPrevious | Excludes current layout focus, selects maximum get_focus_timestamp over layout windows, commits MRU keyboard focus then focuses that window. It is not the previous ordinal or merely the adjacent column. |
| FocusWorkspace / Previous | Reference parsing and per-output previous identity rules below; target selection and workspace auto-back-and-forth are separate branches. |

Zero can decode for the reviewed ordinal types and reaches the first element
through saturating subtraction; a large valid value reaches the last. Negative,
fractional and overflow decoding still need parser fixtures. Do not copy these
bounds to every action with a numeric argument. Full floating/fullscreen/tab/
monitor context and animation interruption remain acceptance work.

## Workspace reference interfaces

Config WorkspaceReference accepts a string as Name and an integer fitting u8 as
Index. A quoted numeric string is still a name; config decoding cannot directly
construct Id from this scalar. Unsupported/out-of-range values emit diagnostics
and can produce Index(0) as a placeholder, not an accepted config.

CLI WorkspaceReferenceArg::from_str tries i32 first, then u8: parsed integers
outside [0,255] error; strings not parsed as i32 become names. Thus an all-digit
string overflowing i32 can follow the name branch, unlike a negative in-range
i32, which fails u8 conversion. CLI parsing does not build Id through this
FromStr helper, although the enum/IPC serialization has an Id alternative.
Do not invent a #ID CLI shorthand from the existence of that variant.

The runtime helper maps Index through saturating_sub(1) on the current output;
Name and Id resolve existing workspace/output identity. Monitor switching clamps
the resulting index to the last workspace. FocusWorkspace applies automatic
back-and-forth only on the same-output branch; cross-output selection switches
directly and uses centered cursor/output fallback. FocusWorkspacePrevious uses
previous workspace identity on the active monitor. Removed/moved/name-colliding
workspaces and every target action still require fixtures; the
[focus audit](niri-input-focus-audit.md#workspace-back-and-forth) retains its limits.

## Driftwm differences and remaining acceptance

Driftwm has its own TOML action/binding parser, mouse/gesture contexts and partial
column navigation. It does not provide this KDL grammar, complete per-bind flags,
workspace identities or all ID/ordinal/previous/wrapping actions. Existing related
actions remain Partial/Missing/Divergent; this review adds source evidence and
never upgrades them to Exact. Help overlay presentation alone is Outside, while
binding resolution and input consequences remain in scope.

Required independent probes include alias/whitespace/keysym duplicates, same-block
errors versus later-block override, missing/multiple/invalid action children,
skipped config variants versus IPC, all bind flags and cooldown/repeat/lock/
inhibition combinations, unknown/stale IDs, zero/one/large ordinal targets,
quoted numeric workspace names and CLI overflow, previous focus timestamps/MRU,
floating/fullscreen/tab/empty/no-output contexts, deletion/reorder and interrupted
transitions. This source audit does not report those fixtures as passing.

Next A0: remaining action families/argument decoders (size/position changes,
compound monitor/workspace actions and target variants), misc/root/recent-windows
semantics and independent executable reference fixtures. Twelve reviewed actions
do not close the 186-variant behavioral audit or authorize feature expansion.
