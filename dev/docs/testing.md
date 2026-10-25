# Testing

The test-suite map and the few rules that keep it trustworthy. The standard
tests need no display or root. A subset of ignored rendering tests attempts to
use surfaceless GLES and skips itself when the required EGL/GLES support is not
available. CI runs the complete command sequence from
[`ci-local`](../scripts/ci-local), including those ignored cases and the
isolated soak test.

## The map

- **Unit tests** — `#[cfg(test)]` modules across `src/` (canvas math, config
  parsing, membership, persistence change detection, …). Pure logic; smithay
  glue (handlers, delegates) is deliberately untested — see
  [caveats.md](caveats.md).
- **Integration tests** — `tests/*.rs`, linking the lib crate: config
  parse/docs-sync gates, canvas transforms, snap, window-rule matching.
- **Stage proptest harness** — `src/stage/tests.rs` (`mod harness`): random
  op-histories (map/close/crash/fullscreen/pin/fit/cluster/hotplug/auto-place)
  drive the real `Stage` alongside a model of `Driftline` policy, with structural
  invariants checked after every op. This layer found the suspended-pin
  output-unplug bug.
- **Config robustness proptests** — `tests/config_robustness_test.rs`:
  arbitrary and hostile TOML through `Config::from_toml_collect` must never
  panic — the mechanical form of the hot-reload "a bad edit never crashes"
  promise.
- **In-process conformance fixture** — `src/tests/` (bin-crate unit tests): a
  real `Driftline` runs headless (`backend = None`), real wayland-client test
  clients connect over socketpairs, and one `Fixture::dispatch()` pumps the
  whole graph deterministically. `verify_stage_invariants` runs on every
  dispatch, so a state leak aborts the test and an in-process deadlock becomes
  a hang you can't miss. Covers the layer nothing else reaches: protocol
  wiring, configure sequences as the client sees them, crash paths, focus
  timing. Start reading at `src/tests/window_opening.rs` — the idiom is
  map → roundtrip → snapshot/assert, and new scenarios should read as
  specification sentences. Every scenario is also leak-checked at teardown: the
  fixture's `Drop` tears down all clients and asserts `debug-counters` return to
  the construction-time baseline (opt out with `skip_baseline_check`).

## Rules

**Config options end in the reference, and the generated doc is committed.**
`config.reference.toml` is the single source of truth for every option
(grammar: [reference-config-format.md](reference-config-format.md)). Adding or
changing an option means documenting it there, regenerating with
`UPDATE_CONFIG_DOCS=1 cargo test docs_config_md_is_up_to_date`, and committing
both files. Never hand-edit `docs/config.md`. Tests gate the loop: reference
examples must parse warning-free, reference defaults must equal code defaults,
and the generated doc must be current.

**Never touch `Space` window APIs — go through the stage.** `clippy.toml`
rejects every `Space` element read and write via `disallowed-methods`; the
stage is the sole window store and `Space` is output-registry only. Applies to
tests too: drive `Driftline` methods or the fixture. Details in
[caveats.md](caveats.md).

**Proptests run 256 cases in CI; go deeper before merging.** Plain
`cargo test` uses proptest's default 256 cases per property. Work that touches
the stage, layout, or config parsing deserves a deeper local pass:
`PROPTEST_CASES=4000 cargo test` (seconds, not minutes). When proptest finds a
counterexample it writes a seed under `proptest-regressions/` — commit it;
seeds replay first on every subsequent run, pinning the bug forever.

**Non-hermetic tests are `#[ignore]`d, not env-gated.** The default
`cargo test` lane stays hermetic and deterministic; anything that steps
outside it carries `#[ignore = "reason"]` and runs via
`cargo test -- --include-ignored` (or `-- --ignored` for just those). CI runs
the full include-ignored lane. Three exist today: the soak/leak scenario
(`src/tests/soak.rs`) churns hundreds of windows and asserts counters, file
descriptors, and RSS all plateau — the fd/RSS assertions are process-wide, so
prefer running it alone (`cargo test --bin driftline soak -- --ignored`) when
debugging a failure; the real-client scenario
(`src/tests/real_clients.rs`) spawns an actual `foot`/`weston-terminal`
against a private wayland + IPC socket and drives it over the wire,
self-skipping when neither is on `PATH`; and the pixel scenarios
(`src/tests/render_pixels.rs`) give the fixture a real GLES renderer on Mesa's
surfaceless EGL platform (`src/tests/gl.rs`) and assert on the bytes the chrome
shaders produce, self-skipping where there is no EGL/GLES. smithay keeps
surfaceless displays in a process-global set and terminates one when its last
handle drops, so those scenarios take `gl::lock()` as their first statement and
run one at a time whatever the harness's thread count — a new GL scenario must
do the same. Mark any future non-hermetic or genuinely slow test the same way.

**Fixture tests never touch the real session.** Construct configs with
`Config::from_toml` + `Fixture::with_config` — never read
`~/.config/driftline/config.toml` (that's what `Driftline::new_with_config` is
for). Never read or write `$XDG_RUNTIME_DIR/driftline`; the test pump
deliberately excludes the state-file writer. Don't set process env vars in
tests — the harness is multi-threaded.

## Hardware acceptance

Run from a separate TTY with an existing isolated test config and a separate
session file, so the normal desktop state is preserved:

```bash
cargo build --release --locked
./target/release/driftline --backend udev --config /path/to/test-config.toml --session-file /path/to/test-session.json
```

Omit `--drm-connector` to use every connected output, or add
`--drm-connector HDMI-A-1` with an actual connector name to isolate one output.
The default exit binding is `Mod+Ctrl+Shift+Q`. See the
[tiling guide](../../docs/tiling.md) for controls.

On a multi-monitor setup, check independent focus, reorder, stack and resize;
new windows; keyboard and mouse transfer; fullscreen on one output; hotplug
with pending client resizes; and return to saved canvas geometry and views.
Restart with the same session file to check restored output ownership, column
and stack order, widths and selection. Use the actual hardware for visual
acceptance: surfaceless GLES tests do not prove driver or monitor behavior.
Record remaining checks in [tiling status](tiling-status.md).

## Container build checks

For example, check Arch dependencies from the repository root with Podman:

```bash
podman run --rm -it --security-opt label=disable -v "$PWD":/src archlinux:latest bash
pacman -Syu --noconfirm rust cargo pkg-config libdisplay-info libinput seatd mesa libxkbcommon
cd /src
cargo build
```

The volume is writable so Cargo can build in `target/`; on Fedora,
`--security-opt label=disable` avoids SELinux blocking the container's access.
