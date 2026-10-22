# Pinned niri recent-windows configuration and model

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source audit
covers all 29 named recent-windows fields, tuple wrapper/enum grammar and selected
model/render/reload consumers. It adds no action rows: all 186 already link scope
review. It does not execute the parser, MRU UI, clients or GPU, and changes no
driftwm runtime/public config. Complete focus/input/render acceptance stays open.

## Pinned sources

- [niri-config/src/recent_windows.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/recent_windows.rs)
- [niri-config/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs)
- [niri-config/src/binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/ui/mru.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/ui/mru.rs)

The [action scope audit](niri-integration-action-scope-audit.md#mru-open-selection-and-close)
records the eight action dispatch paths; this document adds schema/model detail.
Shared Key and Color retain their [binding](niri-binding-address-audit.md) and
[appearance](niri-appearance-config-audit.md) decoders.

## Defaults and merge

| Setting | Default / public shape |
| --- | --- |
| on | true; Part on/off are child flags |
| debounce-ms | 750; optional u16 argument |
| open-delay-ms | 150; optional u16 argument |
| highlight active-color | RGBA (0.6,0.6,0.6,1); shared Color child |
| highlight urgent-color | RGBA (1,0.6,0.6,1); shared Color child |
| highlight padding | 30; FloatOrInt [0,65535] child argument |
| highlight corner-radius | 0; FloatOrInt [0,65535] child argument |
| previews max-height | 480; FloatOrInt [1,65535] child argument |
| previews max-scale | 0.5; FloatOrInt [0,1] child argument |
| binds | Eight built-in keyboard bindings until the first custom binds section |

Merge first ORs on, then off sets on=false; off wins within the same part. Omitted
scalar/Color values preserve earlier parts. Highlight and previews merge their
individual present fields; empty nested blocks do not reset previously supplied
values. A later on part can enable after off. Do not reuse the animation root's
opposite on/off precedence.

Default bindings are Alt and symbolic Mod with Tab for next-window and Shift+Tab
for previous-window, filter All; grave and Shift+grave use AppId filtering. That
is eight records before backend Mod resolution. Default bindings allow inhibition
and have no custom hotkey title.

Root ConfigPart shares SawMruBinds across include parsing. On the first present
recent-windows binds child it clears built-in defaults before merging, even if
the block is empty. Later binds blocks remove existing matching Key records and
append replacements. A later empty block does not clear all prior custom binds.
This root behavior is separate from RecentWindows::merge_with; inspecting the
merge helper alone would incorrectly preserve defaults. Full include-error and
reload reconstruction acceptance remains open.

## MRU binding and enum grammar

MruBinds accepts child keybind nodes. Key parsing uses the shared modifier/keysym
rules, but MRU requires a nonempty modifier set and keyboard Keysym trigger.
Mouse/wheel/tablet triggers that parse as general keys are diagnosed here.
Duplicates within one MRU binds block are errors after normalized Key comparison;
later-block overrides use the merge policy above. Bind type annotations and
arguments are rejected, one action child is required, and extra actions or
unknown properties emit diagnostics. Missing/invalid action may return a dummy
NextWindow solely to retain duplicate diagnostics; that is not config success.

MruBind accepts allow-inhibiting boolean default true and hotkey-overlay-title
as absent/null/string. It does not accept general Bind repeat, cooldown or
allow-when-locked properties. Conversion to Bind fixes repeat=true, cooldown=None,
allow_when_locked=false and preserves inhibition/title. Help-title presentation
alone is Outside; MRU key/focus resolution stays in scope.

Public MRU actions are next-window and previous-window, each with optional scope
and filter property. Scope decodes all/output/workspace; filter's AppId decodes
app-id. Filter All is the omitted default and explicit filter="all" is also
accepted, as confirmed by the [executable parser contracts](niri-parser-reference-fixtures.md).
The earlier inference from knuffel(skip) was incorrect: pinned knuffel-derive
3.2.0 DecodeScalar enumerates unit variants without applying that variant attribute. Direction Forward/Backward is supplied
by the chosen action, not a user child field. Conversion creates MruAdvance with
an explicit Some(filter), including default All, so a default bind can clear an
existing AppId filter when advancing. Runtime cycle order is All → Workspace →
Output → All; declaration order must not be substituted.

## Model ordering, scope and filter

WindowMru::new requires an active output, otherwise creates an empty model. It
collects mapped workspace windows, recording whether each belongs to the opening
output/workspace, then sorts descending by optional focus timestamp. It initially
selects the first. Exact tie/None ordering and focus debounce need independent
fixtures. is_empty checks the complete vector, distinct from the filtered list.
A nonempty collected model can therefore have no selected candidate after scope
filtering; do not infer otherwise from the initial nonempty guard.

Next/previous wraps within the filtered iterator. First/last selects filtered
endpoints. Changing scope keeps the same candidate when allowed, otherwise the
nearest candidate to its left in original vector order, then the first on the
right; with no current selection it picks the first matching candidate. It is
not an unconditional reset to most recent on every scope change.

AppId filtering captures the current selected thumbnail's cached app ID, not an
arbitrary current keyboard-focus surface. If that thumbnail has no app ID, the
requested filter cannot be established in that branch. Repeating AppId when
already filtered does not replace the captured string. Removing All filter can
restore candidate selection. Scope and AppId predicates are combined; app ID comparison is exact string
equality, not regex matching or ASCII case folding.

Thumbnail app ID is explicitly cached rather than updated live to avoid
refiltering; its output/workspace flags originate at model construction. Existing
thumbnail size can update from mapped geometry, and removal updates the model;
last-candidate removal can cancel UI. The selected paths do not prove every new
window, metadata, output migration or hotplug event updates every cached field.
These require lifecycle fixtures rather than assuming live refilter/re-sort.

## Timing, rendering and reload

Keyboard focus history uses the configured debounce: a first timestamp or zero
debounce commits immediately, otherwise a pending timer can be committed early
by keyboard interaction. That affects later model ordering. The UI open delay
uses unadjusted time; model/UI can be open before it is visible. Pointer selection
waits until fully open. Closing before that point returns selection/cancel
without a fade; visible close uses recent-windows-close animation. Focus/warp
ordering for Confirm remains in the action audit.

Preview sizing clamps max-height to at least 1 and max-scale to at least 0.001
at runtime. Thus decoded max-scale=0 does not yield a zero-scale preview. It also
limits height by output height*max-scale, derives an output-aspect width limit,
fits window size into that box and caps scale. It then applies a minimum scale
min(1,16/max(window_width,window_height)) before physical rounding; that floor can
override the earlier maximum scale/box. These are not unconditional upper bounds
or a fixed 480-pixel raster independent of output/scale/window geometry.

Highlight padding is rounded physically; configured corner radius and colors feed
background/border geometry. Urgent color is selected for urgent windows, otherwise
active color; inactive entries dim it by 0.4. Thumbnail title/padding, opacity,
privacy block-out and closing fade have additional consumers, so this source
reading is not a pixel/shape acceptance result.

Reload calls update_binds when bindings change and update_config for changed MRU
settings. UI update_config touches Open and Closing states, resets freeze_view
and copies preview settings to existing thumbnails. It does not recompute the
already stored open_at deadline in that helper. Config is shared for other draw
settings; this does not imply every cached texture/model field resets.

Binding selection includes MRU binds when enabled OR already open. General binds
are included only when MRU is not open and precede MRU binds there; open UI adds
its own dynamic bindings. Disabling recent-windows is therefore not by itself
proof that an already-open switcher closes or loses its close controls. Full
modifier-release behavior, inhibition and lock handling stay executable gates.

## Driftwm gap and continuation

Existing driftwm focus history/canvas navigation is not this configurable MRU
model/UI, scoped app filtering, preview policy or binding grammar. The facility's
rows are Missing, with the help-title presentation row Outside; this is source
comparison, not new implementation. Action rows retain their conservative
integration labels. No Exact or completion claim follows from source links.

Independent fixtures must cover on/off order, first/empty/later binds across
includes, duplicate/invalid binds, explicit/omitted All filter and unknown values,
missing/cached app IDs and scope selection, timestamp ties/debounce, filtered
empty models, wrap/removal, opening/closing delays, zero preview scale and
fractional geometry, reload while open/closing, modifier release/inhibition,
output migration, metadata change, capture privacy and candidate churn.

Next A0: remaining misc/root integration decoders and evidence-backed retained
scope, then independent executable parser/geometry/operation fixtures beyond
zero gaps. MRU source coverage does not close behavioral P0/P14 acceptance.
