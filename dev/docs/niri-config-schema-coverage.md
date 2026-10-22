# Pinned niri configuration schema coverage

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 reconciliation
accounts for all **18 Rust source files** under niri-config/src, including the
nested utils helper. Four infrastructure files have explicit reasons for not
indexing their named fields as options. The other **14 modules** contribute
**576 named public-struct fields**, up from 318 in seven selected modules.
The action inventory remains 186 variants. These are source-coverage counts,
not a parity percentage or a count of public KDL keys.

The [generated inventory](niri-tiling-inventory.md#module-coverage) contains the
file-by-file scope, inclusion/exclusion reason and raw field counts. Its
[root decoder table](niri-tiling-inventory.md#public-root-decoder-nodes) accounts
for all **25 public root nodes** and their aliases to **24 Config fields**.
`include` has no Config field because it is a custom decoder directive.
The [manual generator](../scripts/niri-parity-inventory.py) validates these
coverage sets against the pinned source before producing the inventory.

This is a source-index reconciliation, not a full decoder/default/lifecycle audit.
The newly indexed fields stay Unverified unless an existing scoped audit or an
explicit exclusion justifies a stronger label. No driftwm runtime/configuration
behavior changes or Exact claims are made here.

## Evidence and coverage method

- [Root Config and ConfigPart decoder](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs): actual root-node match arms, singular/plural aliases and module declarations.
- [misc.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/misc.rs): Overview/OverviewPart, cursor and unrelated process/UI helpers.
- [layer_rule.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/layer_rule.rs): layer match/exclude, placement/backdrop, shadows, geometry and popup effects.
- [input.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/input.rs): focus, pointer warping, workspace back-and-forth and modifier policy mixed with device configuration.
- [debug.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/debug.rs): layout-visible transaction/resize/focus switches mixed with backend diagnostics.
- [binds.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/binds.rs): Bind/Key/SwitchBinds/SwitchAction storage plus custom Action parsing; [recent_windows.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/recent_windows.rs): MRU focus UI, preview settings and binding overrides.
- [error.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/error.rs), [macros.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/macros.rs), [utils.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils.rs), [merge trait](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils/merge_with.rs): diagnostics or helper mechanisms, not independent configurable trees.

The source file set must equal the curated coverage manifest; an added/missing
file cannot silently disappear from the reconciliation. Root nodes are extracted
from ConfigPart's actual dispatch arms and checked against an explicit scope/
Config-field map. The mapped storage fields must equal all Config fields, with
include deliberately mapped to no field. This catches aliases such as `output`
→ outputs, `window-rule` → window_rules and `workspace` → workspaces without
mistaking plural Rust names for public syntax.

Named fields are extracted from public named structs, including resolved/runtime
storage and partial schemas. Tuple/newtype wrappers and enums are listed as
declared types and still need custom-decoder/value audits. For example, indexing
Bind.action is not auditing every binding alias or every Action argument, and
indexing internal Color.r does not make r a public KDL property. Generic data
coverage and semantic auditing are separate gates.

## Newly indexed surfaces

| Module | New named fields | Reason it cannot be omitted |
| --- | --- | --- |
| lib.rs | 24 | Root Config storage and actual custom public node dispatch must agree. |
| misc.rs | 31 | Overview/OverviewPart live here, alongside cursor and non-layout settings. |
| layer_rule.rs | 13 | Layer placement/backdrop/effects can affect the scene and layout integration. |
| input.rs | 103 | Focus/warping/workspace policy is layout-visible; device-only settings cannot justify excluding the whole file. |
| debug.rs | 44 | Resize throttling, transactions, strict focus and unfocused-window activation policy can affect observable layout behavior. |
| binds.rs | 14 | Binding aliases, ID/index/unit arguments and switch-event actions are not covered by merely listing Action variants. |
| recent_windows.rs | 29 | MRU focus/preview UI is a relevant interaction surface; it is not the unrelated shortcut-help overlay. |

Total addition: 258 fields. All previous 318 rows remain. Source-indexed
appearance value types and the scoped audits remain valid, but those audits do
not automatically verify newly indexed container/consumer settings.

Overview's recovered public subtree is `overview { zoom ...; backdrop-color ...;
workspace-shadow ...; }`: zoom is FloatOrInt in [0,1] with resolved default 0.5,
backdrop-color uses Color with default RGBA `(0.15,0.15,0.15,1)`, workspace-shadow
uses the value/normalization already [audited](niri-effects-config-audit.md#workspace-shadows).
These three resolved fields and three optional part fields are Missing as a niri
overview facility in driftwm; arbitrary canvas zoom is not that facility.
Scoped zoom/entry/render/interaction consumers are now recorded in the
[layer/overview audit](niri-layer-overview-audit.md); complete interaction parity
and independent fixtures remain open.

LayerRule and its own Match are now visible separately from WindowRule.Match.
Its namespace/startup/layer matching, defaults, compositing placement, refresh
and scoped body/popup consumers are traced in the layer/overview audit. Shared
value types do not imply automatic window-rule equivalence or full layer/capture
acceptance.

## Explicit exclusions and mixed scopes

These exclusions concern the objective of niri tiling behavioral parity while
retaining driftwm canvas, not deletion of existing driftwm features:

- Startup process commands and child environment setup are Outside. Windows
  created by those programs still use the in-scope mapping/layout lifecycle.
- Screenshot filename/path policy is Outside; capture clipping, effect privacy
  and layout-visible capture behavior are separate concerns and are not excluded
  merely because the path is excluded.
- Clipboard selection policy, shortcut-help overlay and config-error notification
  presentation are Outside. The help overlay is distinct from workspace overview
  and MRU focus UI, which remain in scope.
- error.rs exposes four named diagnostic/result fields. They describe parse
  results and include errors, not public option nodes, so they are not in the
  576 schema rows. Config error/reload behavior can still be tested at consumers.
- macros/utils files are Infrastructure. Their merge/scalar/wrapper logic is
  audited through owning schemas; they do not add independent root option trees.

Input, debug, cursor, switch events, xwayland integration, output and layer rules
have mixed/integration scope and remain visible. They cannot be excluded wholesale
as unrelated compositor settings. A future field exclusion needs a concrete
layout-observability reason; Unverified is the conservative current label.
The generated root table records these decisions rather than hiding surfaces.

## Validation and continuation

Regenerate from the pinned checkout; compare byte-for-byte. Verify 18 coverage
entries, 14 indexed modules, 25 root nodes / 24 storage fields, 576 unique
module-qualified schema rows and all 186 action variants. Preserve all previously
indexed 318 rows and their scoped semantic links. Local targets/anchors and pinned
source paths must resolve. These checks validate source coverage, not accepted
runtime behavior, parser diagnostics or every possible public child/property.

LayerRule/Match and Overview/OverviewPart now have a scoped source audit.
The [input focus audit](niri-input-focus-audit.md) covers 18 selected input/debug
rows; other fields remain unverified. The [animation/gesture audit](niri-animation-gesture-audit.md)
adds 63 reviewed rows, including six explicit exclusions for unrelated UI motion.
The [input/output integration audit](niri-input-output-integration-audit.md) adds
45 review links for client delivery, output identity and remaining debug scope.
Only the DBus-export switch is newly Outside; retained Unverified backend/capture
rows remain open. The [device input audit](niri-device-input-audit.md) adds 89
links and completes all 103 named input fields; Unverified device/keymap/protocol
acceptance remains open. The [output/window rule audit](niri-output-window-rule-audit.md)
adds 42 links, completing named rows in those modules; DRM/protocol/render and
parser acceptance remains open. The [binding/address audit](niri-binding-address-audit.md)
adds 14 named bind fields, source-derived config exposure and 12 scoped action
consumers. The [size/transfer action audit](niri-size-transfer-action-audit.md) adds
16 selected enum-payload consumers without adding schema rows. The
[preset/topology action audit](niri-preset-topology-action-audit.md) adds 23 scoped
action consumers, retaining existing Partial status. The
[navigation/view action audit](niri-navigation-view-action-audit.md) adds 40 scoped
action consumers. The [local/state action audit](niri-local-state-action-audit.md)
adds 39 scoped consumers. The [workspace/policy action audit](niri-workspace-policy-action-audit.md)
adds 24, correcting seven opacity/cast generic Outside labels to integration.
The [remaining integration/action scope audit](niri-integration-action-scope-audit.md)
adds the final 32 action links and 16 explicit scope corrections. All 186 action
variants now have source-scope evidence; runtime/argument acceptance remains open.
The [recent-windows audit](niri-recent-windows-audit.md) adds 29 named field links
(525 at that step); MRU runtime/decoder/lifecycle acceptance remains open.
The [root/misc audit](niri-root-misc-audit.md) adds the final 51 row links, so all
576 named fields now have scoped source evidence. It includes explicit exclusions
and retained Unverified integration, not complete parser/consumer acceptance.
Next A0: independent executable pinned-reference parser/merge/error fixtures,
then nonzero-gap/fractional-frame geometry and operation traces.
Document concrete exclusions as needed. Independent decoder/geometry/operation
fixtures beyond zero gaps remain required; no Exact or overall-completion claim
follows from covering the source file list.


## First executed reference contracts

The [parser corpus](niri-parser-reference-fixtures.md) runs the actual pinned
niri-config parser in an archived workspace: the first 16 tests passed. It
corrected an earlier inferred rejection of explicit MRU filter="all". This is
bounded executable reference evidence, not driftwm equivalence or completion of
all 576 field /186 action contracts. Source inventory stays unchanged. Next:
remaining scalar/error cases and nonzero-gap/fractional-frame geometry/operation
fixtures; retained consumer/integration acceptance stays open.
