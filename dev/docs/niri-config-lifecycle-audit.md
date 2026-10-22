# Pinned niri configuration precedence and lifecycle audit

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 batch traces
global/output/workspace layout parts, window-rule matching/composition, initial
configure/map and config/rule refresh. It adds lifecycle evidence for 22
WindowRule fields, nine Match properties and Output.layout / Workspace.layout:
33 source-index rows. It does not finish every window-rule decoder, output/name
matching grammar, placement policy or reload permutation.

Evidence is inspection of the clean pinned source, not execution of niri or a
differential oracle. The [layout decoder audit](niri-layout-config-audit.md) and
[appearance/local-merge audit](niri-appearance-config-audit.md) remain the source
for value grammar and defaults. No driftwm runtime/config change is made here.

## Pinned source paths

All links target the pinned revision:

- [layout/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/mod.rs): `Options::from_config`, `with_merged_layout`, `adjusted_for_scale`, `Layout::update_config`, `update_options`.
- [monitor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/monitor.rs): `Monitor::new`, `update_config`, `update_layout_config`, `unname_workspace`.
- [workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/workspace.rs): `new_with_config`, `update_config`, `update_layout_config`, `resolve_default_width`, `resolve_default_height`, `unname`.
- [window/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mod.rs): `ResolvedWindowRules::compute`, `window_matches`, `WindowRef`, `compute_open_floating`, constraint application.
- [window/mapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/mapped.rs): `recompute_window_rules`, state setters and dirty-rule flag; [window/unmapped.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/window/unmapped.rs): `InitialConfigureState`.
- [handlers/xdg_shell.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/xdg_shell.rs): `send_initial_configure`, `update_window_rules`, app-ID/title callbacks; [handlers/compositor.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/handlers/compositor.rs): first-buffer mapping and null-buffer unmap.
- [niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs): config application, `reload_output_config`, `refresh_window_rules`, `recompute_window_rules`, startup completion.
- [tile.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tile.rs): `new`, `update_config`, `update_window`; [scrolling.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/scrolling.rs): `Column::new_with_tile`, configuration update/preset invalidation, initial bounds/size calculation.
- [tab_indicator.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/layout/tab_indicator.rs): per-tile rule color/gradient selection during indicator rendering.
- [config window_rule.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/window_rule.rs), [output.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/output.rs), [workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/workspace.rs), [lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs): decoded parts, ordered rule accumulation and include context.

## Layout inheritance

The stable construction/update chain is:

1. `Options::from_config` clones resolved global `config.layout`.
2. Monitor clones those base options and merges its optional output LayoutPart.
3. Workspace clones monitor options, merges its optional workspace LayoutPart,
   then adjusts gaps for the output's fractional scale.
4. A tile derives border/focus-ring/shadow from those resolved workspace values
   and merges the window's composed appearance rules into them.

Output and workspace layout parts use the field-specific merges already audited:
not every value is simply replaced. An omitted scalar inherits, an empty default
width explicitly clears, an empty preset vector restores built-ins, a Struts
value replaces all sides and nested appearance parts merge. Workspace layout
rejects insert-hint and empty-workspace-above-first. `Outputs::find` returns the
first matching output config; this lookup does not merge every matching profile.
Full output-name/multiple-profile parsing remains separate work.

`Options::adjusted_for_scale` rounds the final gaps through
`round_logical_in_physical_max1`; tile border and focus-ring widths are also
rounded at application. Decoded fractional settings are not necessarily the
same values used in geometry. Interactive-move options explicitly merge output
then workspace parts before scale adjustment too; full cross-workspace movement
semantics remain P11/P12 work.

## Rule matching and composition

`window-rule` nodes are appended in config traversal order, including positional
include processing. `ResolvedWindowRules::compute` starts from a fresh default
on every recomputation. A rule participates when its match list is empty **or**
any match block matches, unless any exclude block matches. Properties inside a
single match block are ANDed. Matching rules then apply in list order: later
present scalar/optional values override earlier ones; absence preserves earlier
values. Nested BorderRule/ShadowRule/TabIndicatorRule values compose with their
own merge logic. Explicit false remains a present override.

`match` and `exclude` have these nine optional **properties**:

| Public property | Source-derived meaning |
| --- | --- |
| `app-id`, `title` | RegexEq strings compiled by regex::Regex; no implicit full-string anchoring. Missing role metadata fails that property. |
| `is-active` | Pending server Activated state, not synonymous with keyboard focus. |
| `is-focused` | WindowRef focus state. |
| `is-active-in-column` | WindowRef selection state. |
| `is-floating` | WindowRef floating state, not a prediction from open-floating. |
| `is-window-cast-target` | Current cast-target state. |
| `is-urgent` | Current urgency state. |
| `at-startup` | Compared to the compositor's is_at_startup flag before the other checks. |

For Unmapped windows, WindowRef reports focused/urgent/floating/cast-target false
and active-in-column true. The source explicitly notes that is-floating cannot
be used to select initial-configure properties based on the future open-floating
result. Pending Activated state and app-ID/title still come from the role.
Startup completion flips is_at_startup false and recomputes rules.

Composition is not the last step for appearance: all matching shadow rules first
merge into a ShadowRule (on normalizes over off), then Tile merges that combined
rule into the resolved workspace Shadow (whose direct merge gives off precedence
if it receives both raw flags). Global/output/workspace ShadowRule parts merge
directly into resolved Shadow, while window rules compose before application.
This locates the distinct paths recorded in the appearance audit; one universal
flag precedence would change the pinned behavior.

## Initial configure and first buffer

`send_initial_configure` computes rules for WindowRef::Unmapped. For target
monitor selection it tries the rule's named workspace, rule output, client
fullscreen-request output, parent monitor, then active monitor. Invalid/unavailable
candidates fall through. If following a parent, the target output is not frozen,
so mapping can follow a parent that moved. Workspace selection uses the named
workspace if found, otherwise the selected monitor's active workspace, with the
layout active workspace as the headless fallback.

| Scoped rule fields / public children | Consumer and lifecycle |
| --- | --- |
| `default-column-width`, `default-window-height` | DefaultPresetSize wrappers. Width resolution: explicit Some(value) wins, explicit empty clears, absent floating width stays client-chosen, absent tiled width inherits workspace layout. Height has no global default and absent remains client-chosen. |
| `open-on-output`, `open-on-workspace` | Initial target hints; map revalidates connected output / existing workspace. They are not general dynamic move commands. |
| `open-maximized`, `open-maximized-to-edges`, `open-fullscreen` | Separate concepts. open-maximized is the full-width column choice; maximized-to-edges uses the Maximized sizing state; fullscreen has its own state and client-request fallback. Explicit false can override the corresponding client/default choice. |
| `open-floating` | Explicit bool overrides the heuristic. Otherwise a parent or a positive fixed height after rule/client constraints implies floating. Computed for initial sizing and again just before mapping because constraints can arrive between configure and first buffer. |
| `open-focused` | Map-time activation override. If absent, token age and normal smart/strict focus policy decide; it does not refocus a mapped window on each recompute. |
| `default-column-display` | Consulted for initial bounds/size and new-column construction, with window rule before layout default. Recomputing the stored rule alone is not proof an existing column changes display mode. |
| `default-floating-position` | Consulted by floating initial-position logic; full position decoder, coordinate transform and transition coverage remain open. |
| `min-width`, `min-height`, `max-width`, `max-height` | Optional u16 per-rule constraints applied in addition to client constraints. Min takes max; max uses the bounded positive rule cap where appropriate, treating client zero as unconstrained. Full conflicting-constraint geometry remains separate fixture work. |
| `focus-ring`, `border`, `shadow`, `tab-indicator` | Composed per-window appearance rules. Tile creation/update applies border/focus-ring/shadow over workspace appearance; tab-indicator rendering consumes the color/gradient-only tab rule. |
| `opacity` | Stored in dynamically recomputed rules; the mapped-window recompute also resets its ignore-opacity toggle when the new rule no longer makes the window semitransparent. |

InitialConfigureState::Configured stores rules **and separate resolved widths,
heights, full-width choice, output/workspace target and pending maximized state**.
The first-buffer path consumes those values, revalidates output/workspace,
recomputes open-floating and applies map-time focus policy. A title/app-ID change
or rule reload while configured-but-unmapped replaces its stored rules, not all
those cached resolved sizing/target fields. Do not promise a fresh initial size
offer from the fact the rules were recomputed. A null-buffer unmap creates a
new Unmapped initial state; a later map goes through a new handshake.

## Reload and dynamic changes

Config application first unnames removed configured workspaces, updates layout
options and existing named-workspace parts, then ensures newly named workspaces.
Monitor/workspace update paths rebuild effective options, update scrolling and
floating spaces, backgrounds and appearance. Output-config reload updates monitor
layout overrides separately; changing the list to no override passes None to
update_layout_config. In the audited reload path, deprecated output background
color fills a missing layout background only inside an existing output layout
part; full legacy/output initialization behavior remains unaudited.

One pinned edge must stay explicit: removing a named workspace config calls
Workspace::unname, which clears **only the name**. A nonempty workspace survives
cleanup; Layout::update_config skips now-unnamed workspaces when refreshing named
parts, and Workspace::update_config still merges its saved layout_config. Thus
this source path retains that saved override after un-naming, rather than
necessarily reverting to global/output values. This is a source-derived edge
case needing an independent fixture, not a proposed driftwm fix or a universal
statement about every workspace move/removal path.

Changing window_rules triggers recompute_window_rules for unmapped and mapped
windows. Mapped values are recomputed from scratch; changed windows are passed
to layout.update_window and redraw. App-ID/title callbacks recompute immediately.
Focus/activation/column-selection/floating/cast/urgency changes mark rules dirty;
refresh_window_rules recomputes them after layout refresh and updates tiled state
before send_pending_configure. Global layout changes also reapply appearance
through Tile::update_config, even if the window-rule list itself did not change.

Preset reload has state effects beyond value replacement: tile floating preset
indices are cleared when lists change; Column clears its width index and converts
preset heights to automatic heights when height presets change. This is not
permission to resize every existing window to the new default-column-width, nor
to rerun initial open-on/fullscreen/focus rules on an already mapped window.
Exact visual/configure sequences remain independent fixture work.

## Driftwm comparison and next fixtures

Driftwm [resolve_window_rules](../../src/config/mod.rs) folds matching rules in
order, using [AppliedWindowRule::merge_from](../../src/config/types.rs). Its
app-ID/title patterns default to globs, with explicit slash-delimited regexes;
there is no niri match-list OR/exclude block or nine-state matcher grammar.
The [first-commit handler](../../src/handlers/compositor.rs) applies/stores rules
while pending_center is active. The stored appearance does not automatically
follow later metadata/config rule changes. [Reload](../../src/state/reload.rs)
swaps global config and clears visual caches; global effect defaults can change,
but clearing a cache is not recomputing mapped AppliedWindowRule values.
[Live input rules](../../src/state/window_lifecycle.rs) deliberately re-resolve
for pass_mouse; [its reload regression](../../src/tests/pass_mouse.rs) is an
exception, not proof of dynamic appearance parity. Existing
[blur reload tests](../../src/tests/blur_pixels.rs) cover driftwm's global-default
policy while retaining mapped rule flags. The fork has no output/workspace tiling
layout override hierarchy; canvas multi-output is not that missing facility.

Source-derived probes for later independent fixtures:

| Trigger | Required observation |
| --- | --- |
| Global gap 16, output gap 8, workspace gap 4 | Workspace effective gap 4 before scale adjustment; removing its layout part while retaining the named workspace falls back to output 8. |
| Output border off, workspace border on, window border off | Final window border off; nested rule composition must precede resolved appearance application. |
| Both raw shadow flags in layout part vs window-rule part | Direct resolved layout merge ends off; composed window-rule path normalizes to on before tile application. |
| Two matching rules set opacity, then second stops matching | Recompute starts from default; the first rule takes effect again rather than retaining the old second result. |
| Regex app-id `foo` against `foobar` | Regex matches; driftwm's default literal glob `foo` does not. No implicit anchor equivalence. |
| Initial rule selected by is-floating=true, another opens floating | Unmapped matcher still reports false; do not use future floating state to match the initial rule. |
| Rule default width absent vs empty, in floating vs tiled | Preserve the distinct client-choice/inheritance cases above. |
| Reload a configured unmapped window's open-on-output/default width | Rules change; cached resolved initial target/width fields are not wholesale recomputed by that updater. |
| Remove a configured workspace name while it still has windows | Check saved layout override retention on the unname/update path, not an assumed fallback. |
| Reload preset heights while windows use presets | Conversion to automatic heights, not dangling preset indices. |

Remaining work includes complete override/placement permutations and decoder
coverage for unscoped rule fields, floating position and output/workspace naming;
CornerRadius/workspace-shadow/blur/background-effect schemas; other config modules;
ID/index/previous/next actions; and independent executable reference fixtures.
A0/P0 remains open. These source traces neither implement missing driftwm settings
nor establish exact runtime/configuration parity.
