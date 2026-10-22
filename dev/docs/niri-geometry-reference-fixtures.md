# Executable pinned niri geometry contracts

The maintained fixture in
[`dev/fixtures/niri-layout/geometry_contract.rs`](../fixtures/niri-layout/geometry_contract.rs)
runs inside the real layout test harness of niri
`12de51d6a60baad8896366920278a24f17921f52`. This is an external reference
contract, separate from driftwm's source-derived Stage snapshots and its CI.

Run from the repository root with a clean reference checkout:

```sh
python3 dev/scripts/niri-config-reference.py /tmp/niri --suite geometry \
  --report /tmp/niri-geometry.json --log /tmp/niri-geometry.log
```

The runner archives the pinned tree and injects a child test module in that
temporary copy. The original checkout stays untouched. The root niri library test
target builds with `--locked --no-default-features -j 2`, using a separate cache.
It still needs niri's native build dependencies. Tests use its headless layout
and simulated window responses; they do not start a compositor or access a GPU.
Default D-Bus/systemd/screencast features are outside this contract.

## Independent expectations

The output is 1280×720 logical pixels at scale 1. Struts reserve 40 on each
horizontal side and 20 on each vertical side, leaving 1200×680. Gaps are 16;
borders are enabled at width 4. Windows accept requested sizes and have no
constraints, titlebars or tab indicators. The helper explicitly passes the
selected initial width into `Layout::add_window`, as its caller must do. Passing
`None` preserves the test client's current width; layout enrollment alone does
not resolve `default-column-width`. This fixture does not test the Wayland
initial-configure caller.

- Fixed width 300 means 300 content pixels and a 308-pixel frame.
- Two half-width frames fit into 1200 pixels with three 16-pixel gaps:
  each frame is 576 wide and its content is 568 wide.
- Full-height content is 640: 680 minus two 16-pixel gaps and two 4-pixel borders.
- Explicit fixed resize to 400 gives a 408-pixel frame.
- Consuming the next window gives two 316-pixel frames with 308-pixel content
  heights: three 16-pixel gaps leave 632 pixels to share. Their origins are
  (56, 36) and (56, 368).
- The first frame starts at (56, 36), after struts plus the outer gap.

Expected constants come from these dimensioned examples, rather than a copy of
niri's implementation formula. Assertions inspect actual client geometry and
IPC tile dimensions/decoration offsets; the origin assertion uses settled render
positions. These are reference assertions, not a driftwm differential comparison.
Fractional scales, broader stacking/consume/expel sequences, client constraints,
tabs, transactions,
view physics and layer-shell exclusive zones remain open.

## Recorded result

The [recorded report](../fixtures/niri-layout/geometry-contract-evidence.json) and
[log](../fixtures/niri-layout/geometry-contract-evidence.log) show five passed,
zero failed and zero ignored tests; the other 203 library tests were filtered out.
The reference checkout remained clean. Hashes identify the archived reference,
lockfile, maintained fixture and run log. A zero-test run is rejected even when
Cargo exits successfully. The parser suite also passed its 16 contracts after the
runner extension. No driftwm runtime code changed in this batch.
