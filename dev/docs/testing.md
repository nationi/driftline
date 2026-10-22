# Testing

The test-suite map and the few rules that keep it trustworthy. Everything runs
under plain `cargo test` — no display, GPU, or root — and CI runs exactly that.

## The map

- **Unit tests** — `#[cfg(test)]` modules across `src/` (canvas math, config
  parsing, membership, persistence change detection, …). Pure logic; smithay
  glue (handlers, delegates) is deliberately untested — see
  [caveats.md](caveats.md).
- **Integration tests** — `tests/*.rs`, linking the lib crate: config
  parse/docs-sync gates, canvas transforms, snap, window-rule matching.
- **Stage proptest harness** — `src/stage/tests.rs` (`mod harness`): random
  op-histories (map/close/crash/fullscreen/pin/fit/cluster/hotplug/auto-place)
  drive the real `Stage` alongside a model of `DriftWm` policy, with structural
  invariants checked after every op. This layer found the suspended-pin
  output-unplug bug.
- **Config robustness proptests** — `tests/config_robustness_test.rs`:
  arbitrary and hostile TOML through `Config::from_toml_collect` must never
  panic — the mechanical form of the hot-reload "a bad edit never crashes"
  promise.
- **In-process conformance fixture** — `src/tests/` (bin-crate unit tests): a
  real `DriftWm` runs headless (`backend = None`), real wayland-client test
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
tests too: drive `DriftWm` methods or the fixture. Details in
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
prefer running it alone (`cargo test --bin driftwm soak -- --ignored`) when
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
`~/.config/driftwm/config.toml` (that's what `DriftWm::new_with_config` is
for). Never read or write `$XDG_RUNTIME_DIR/driftwm`; the test pump
deliberately excludes the state-file writer. Don't set process env vars in
tests — the harness is multi-threaded.

For a direct udev test on a machine with multiple connected monitors, pass
`--drm-connector HDMI-A-1` (using the actual connector name) to enroll only that
connector, including after hotplug. This is useful for the experimental
single-output tiling mode. Omit the option for normal output discovery; it does
not implement independent multi-output tiling. Run from a separate TTY and use
an isolated config/session file to preserve the main desktop.

For this checkout's isolated launcher, keys and return path, see
[the local TTY guide](tty-preview.md). Direct udev is the visual acceptance
environment for the current NVIDIA blur work; nested winit still has
artifacts/lag. The surfaceless GLES helper self-skips without EGL; current local
pixel evidence comes from NVIDIA RTX 3060, not an assumed Mesa software renderer.
Record automated checks and manual observations separately in
[the maintained status](tiling-status.md), with detailed logs in
[history](tiling-history.md).


## Manual pinned-reference contracts

The [niri parser reference corpus](niri-parser-reference-fixtures.md) runs
maintained Rust fixtures against an archived clean pinned niri checkout using
`dev/scripts/niri-config-reference.py`. The default suite builds only the niri-config test target,
keeps separate dependency/cache/provenance and leaves driftwm's lockfile/runtime
unchanged. This manual external-reference gate is separate from ordinary driftwm
CI; it validates reference contracts before differential implementation fixtures.

Use `--suite geometry` for the [executable layout contracts](niri-geometry-reference-fixtures.md).
This builds the pinned root niri library test target without default features and runs
only the injected geometry module. Native build dependencies are still required;
the test runtime needs no display or GPU. Keep executable reference evidence
separate from the older source-derived Stage fixtures.
