# Pinned niri root assembly, cursor and remaining integration

Reference: `12de51d6a60baad8896366920278a24f17921f52`. This A0/P0 source review
adds 51 indexed review links: 24 Config storage fields, 25 remaining misc fields
and two Workspace fields already traced by the identity audit but not previously
linked per row. With earlier audits, all 576 named schema rows and all 186 action
variants now have scoped source evidence. This completes source triage, not public
parser/consumer execution, full acceptance or driftwm feature implementation.
No niri/parser/client/hardware execution or public config change is claimed.

## Pinned sources

- [niri-config/src/lib.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/lib.rs)
- [niri-config/src/misc.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/misc.rs)
- [niri-config/src/utils.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/utils.rs)
- [niri-config/src/error.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/error.rs)
- [niri-config/src/workspace.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/niri-config/src/workspace.rs)
- [src/niri.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/niri.rs)
- [src/input/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/input/mod.rs)
- [src/utils/mod.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/mod.rs)
- [src/utils/watcher.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/watcher.rs)
- [src/utils/xwayland/satellite.rs](https://github.com/niri-wm/niri/blob/12de51d6a60baad8896366920278a24f17921f52/src/utils/xwayland/satellite.rs)

## Root storage, scope and ordering

ConfigPart manually decodes 25 root node names onto 24 Config fields; include is
a directive without storage field. Root default assembly uses Config::default,
not loading the bundled example as an implicit override. Each Config::parse starts
a new default value and contextual include/error/MRU-bind tracking, so successful
reload reconstruction is distinct from merging new text into old live config.
The [audit catalog](niri-config-schema-coverage.md) links each owning field tree.

| Config field | Public root node | Retained scope / status |
| --- | --- | --- |
| `input` | `input` | Integration / Unverified |
| `cursor` | `cursor` | Integration / Unverified |
| `clipboard` | `clipboard` | Outside / Outside |
| `hotkey_overlay` | `hotkey-overlay` | Outside / Outside |
| `config_notification` | `config-notification` | Outside / Outside |
| `animations` | `animations` | Layout / Divergent |
| `blur` | `blur` | Layout/integration / Partial |
| `gestures` | `gestures` | Layout/integration / Missing |
| `overview` | `overview` | Layout / Missing |
| `xwayland_satellite` | `xwayland-satellite` | Integration / Unverified |
| `switch_events` | `switch-events` | Integration / Unverified |
| `debug` | `debug` | Integration / Unverified |
| `outputs` | `output` | Layout/integration / Partial |
| `spawn_at_startup` | `spawn-at-startup` | Outside / Outside |
| `spawn_sh_at_startup` | `spawn-sh-at-startup` | Outside / Outside |
| `window_rules` | `window-rule` | Layout/integration / Partial |
| `layer_rules` | `layer-rule` | Integration / Missing |
| `workspaces` | `workspace` | Layout / Missing |
| `binds` | `binds` | Layout/integration / Unverified |
| `environment` | `environment` | Outside / Outside |
| `prefer_no_csd` | `prefer-no-csd` | Integration / Unverified |
| `screenshot_path` | `screenshot-path` | Outside / Outside |
| `layout` | `layout` | Layout / Partial |
| `recent_windows` | `recent-windows` | Layout/integration / Missing |

Input/cursor/clipboard/help/notification/animation/blur/gesture/overview/satellite/
switch/debug roots merge their owning Part type. Output, startup commands,
window/layer rules and workspaces append records. Environment appends variables;
prefer-no-csd assigns a Flag value, screenshot-path replaces its wrapper, and
bind/layout/recent-windows roots have the earlier audited custom handling.
Root table statuses describe driftwm compatibility, not whether parsing succeeded.

Within one file, repeated single-section roots are diagnosed and the duplicate
is skipped. Exceptions are output, spawn-at-startup, spawn-sh-at-startup,
window-rule, layer-rule, workspace and include. Thus later binds/layout/input/
recent-windows Parts can merge through positional includes, but two such roots
in the same file are not an accepted override syntax. Unknown roots error.
Earlier local-merge audits do not waive this outer duplicate rule.

## Include paths, diagnostics and reload

include takes one path argument and optional boolean property optional, default
false. Extra arguments/properties/children are diagnosed. Home-component expansion
uses the home directory; otherwise path is joined to current file's BasePath,
which changes for nested files. Absolute paths retain normal PathBuf::join behavior.
No glob expansion or shell environment interpolation is implemented by this helper.

Each include is parsed at its source position into shared Config state. Recursion
starts at zero; increment reaching limit 10 is diagnosed before reading that
include. A cloned path stack detects repeated lexical paths along the recursion
chain; it is not a canonicalized global visited set, so repeated nonrecursive
includes and symlink aliases must be tested separately. Include paths are
recorded before read/parse failure after preceding validation, allowing repair
of a missing or invalid included file to trigger reload. Relative diagnostic
names are derived from the root base when possible.

optional=true tolerates only read ErrorKind::NotFound (warning). Other I/O errors
and syntax/decoder errors still fail. Nested parse failures are accumulated into
ConfigIncludeError and a parent diagnostic. Construction can have applied earlier
parts internally, but ConfigParseResult.config is Err, not a partially accepted
live configuration. Its includes list is returned independently on failure.

ConfigPath Explicit never creates a file. Regular selects existing user path,
otherwise existing system path; load errors do not automatically try another
existing file. Startup load-or-create has a separate missing-file path using a
bundled config, while ordinary load reports missing configuration. This path/UI
policy is not the layout behavior contract, but parse/reload failure recovery is.

The watcher parses on its worker thread, polls at 500 ms and sends Result<Config>
to the compositor. It tracks canonical path plus mtime, and includes missing-path
state; an include disappearing/reappearing can trigger a reload. A missing main
config yields Missing and skips that load iteration. Manual load(path) can replace
the watched path. Includes are refreshed after processing even when parsing fails;
source notes a race between reading config and updating include metadata.

reload_config(Err) shows error notification and returns, preserving old live
config. Success applies scoped layout/output/window/layer/input/shader/cursor/MRU
updates and hides notification; those consumers retain their own timing/cache
limits. No universal atomic rollback/retroactive initial-rule claim follows.
Executable parser/error/watcher fixtures remain required.

## Cursor defaults and consumers

Cursor defaults theme=default, size=24, hide-when-typing=false and no inactivity
timeout. CursorPart theme/size are optional string/u8 arguments, hide-when-typing
uses optional Flag (explicit false allowed), and inactivity is optional u32
milliseconds. Present parts preserve absent values; a fresh root parse resets
removed settings through defaults. Zero timeout is a value, not schema absence.

Startup constructs CursorManager; any changed Cursor value reloads theme/size and
clears cursor texture cache. Changed inactivity setting resets its timer. Key-driven
hide honors hide-when-typing and skips active tablet interaction, then sets Hidden
and redraws. The inactivity timer only changes a currently visible pointer to
Hidden, avoiding a one-frame reappearance for an already invisible state. Pointer
motion, hidden-versus-disabled contents, constraints and programmatic warp have
separate policy, already partly traced in the input audit. Theme assets, cursor
capture pixels, fractional scales and lock/tablet/reload behavior need fixtures;
these rows remain Unverified integration rather than automatic exact matching.

## Decoration preference and satellite lifecycle

prefer-no-csd is a root Flag, default false; a bare node means true and explicit
false assigns false. Client insertion snapshots whether decoration globals are
visible from this preference. Changing it on reload is not a guarantee those
globals are exposed/retracted for existing clients. Regular window refresh and
rule refresh separately call the earlier update_tiled_state helper, whose default
can change protocol Tiled hints from negotiated decoration or preference. Those
hints are distinct from structural column membership. Driftwm CSD-first policy
and advertised tiled-state workaround remain a documented difference, not parity.

XwaylandSatellite defaults off=false and path=xwayland-satellite. Part off/on are
flags; on wins within a part. Omitted path preserves earlier value. Setup returns
if a Satellite already exists or configured off. Otherwise it probes executable
listenfd support, creates X11 sockets and installs readiness watchers; later
connections start the satellite. Home path expansion is separate from shell
command parsing. Socket/process failure and reconnect paths remain integration.

Reload attempts setup when settings changed, but a working satellite is not killed
or its watchers stopped merely by off/path change. Child DISPLAY selection is
updated based on enabled state and available satellite; source explicitly does
not update systemd environment there. Driftwm currently starts satellite eagerly
(src/xwayland.rs), so its enabled/path settings are not proof of the same on-demand
lifecycle. X11 app mapping remains in layout acceptance despite process-launch
UX exclusions. No satellite process was started by this source audit.

## Remaining exclusions and workspace row links

These misc fields have concrete Outside scope: SpawnAtStartup.command is a vector
of arguments, SpawnShAtStartup.command one shell string; EnvironmentVariable uses
node name plus optional value and feeds child environment; screenshot destination
is a tuple Option<String> wrapper without named fields; clipboard disable-primary,
help skip-at-startup/hide-not-bound and notification disable-failed are false by
default and Part flags merge present values. Their process/selection/help/error-UI
presentation is excluded; map/activation, capture privacy, MRU/Overview, reload
failure and input consequences remain in scope. This is not a blanket exclusion
of the misc module.

Workspace.name/open_on_output now link the previously delivered
[identity/placement audit](niri-input-output-integration-audit.md#workspace-names-and-placement).
They retain Missing status; repeated output preferences, ASCII-insensitive names,
creation fallback and restricted WorkspaceLayoutPart are not new implementation.
The two new row links do not represent discovering a new consumer this session.

## Continuation and independent acceptance

The generator now checks complete named-field evidence rather than silently
leaving uncovered source rows. A link is scoped review, not Exact, parser success,
model agreement or hardware behavior. Root and exclusion links must not inflate
behavioral completion claims. Detailed existing Partial/Unverified/deviation
acceptance remains in the owning audits and roadmap.

Next A0: run independent pinned-reference parser/merge/error fixtures, then
nonzero-gap/fractional-frame geometry and operation traces against the reference.
Start with ordered includes, duplicate roots, optional missing versus malformed
files, first MRU binds and on/off precedence. Retain pure fixture provenance,
separate parser construction from acceptance and update status with actual runs.
No feature expansion or full niri parity claim follows merely from 576/186 links.
