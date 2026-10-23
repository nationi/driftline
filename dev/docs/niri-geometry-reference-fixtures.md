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

The base output is 1280×720 physical pixels, also logical pixels at scale 1. Struts reserve 40 on each
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
Scale transitions, broader stacking/consume/expel sequences, client constraints,
tabs, transactions, view physics and layer-shell exclusive zones remain open.

## Fractional-scale expectations

Three additional static-scale cases use the same 1280×720 physical mode, gap 3,
border 1 and struts left/right/top/bottom 10.1/9.9/5.1/4.9 logical pixels. The
selected width is one half of the usable strip. The client accepts integer
logical dimensions. Visual client size is then rounded to physical pixels before
adding the physically rounded border to obtain frame size.

| Scale | Gap (physical) | Border (physical) | Client size (logical) | Frame size (logical) | Frame origin (logical) |
| --- | --- | --- | --- | --- | --- |
| 1.25 | 4 | 1 | 495×558 | 496.8×560 | (13.6, 8.8) |
| 1.5 | 5 | 2 | 408×460 | (1232/3)×(1388/3) | (14, 26/3) |
| 2 | 6 | 2 | 303×342 | 305×344 | (13.5, 8.5) |

At 1.25, the logical view is 1024×576. The left strut begins at physical
12.625, so usable content starts at physical 13, or logical 10.4; the top starts
at physical 7, or logical 5.6. With opposite struts retained, usable dimensions
are 1003.7×565.5. Two half-width frame budgets plus three 3.2-logical gaps fill
the width: each budget is 497.05. Removing two 0.8 borders leaves 495.45; the
client receives width 495. Auto height instead rounds its 557.5 content budget
to nearest integer 558. Visual content is then aligned to physical pixels:
495×558 logical becomes 619×698 physical, or 495.2×558.4 logical. Adding the two
borders gives a 496.8×560 frame. Its height can exceed its allocation; width
flooring does not apply to auto-height allocation. Origin adds one rounded gap to each rounded work-area coordinate.

At 1.5, the configured gap and border land on physical half pixels and round
up to 5 and 2. Usable dimensions after strut-origin alignment are
832.766666…×469.766666…. Half-width client budget is 408.716666…, and full-height
budget is 460.433333…; width floors and auto height rounds to nearest. The 2× case
retains the fractional strut remainder even though border and gap already align
exactly: the auto-height budget 341.6 rounds to 342. Floating-point
assertions allow only 1e-9 logical pixels; client dimensions remain exact.
These static checks do not exercise output scale transitions or GPU rasterization.

## Fractional operation sequences

Three additional contracts use the 1.25-scale case above with fixed content
width 300. Every client resize is answered before advancing the reference clock;
there are no sleeps. Expected values remain independent constants.

| Settled state | Ordered client heights | Ordered visual frame heights | Ordered frame y positions |
| --- | --- | --- | --- |
| First consume (two windows in column) | 276, 277 | 277.6, 278.4 | 8.8, 289.6 |
| Second consume (three windows) | 183, 182, 183 | 184.8, 184, 184.8 | 8.8, 196.8, 384 |
| Middle window fixed at 100 | 224, 100, 224 | 225.6, 101.6, 225.6 | 8.8, 237.6, 342.4 |

With three windows, four gaps reserve 12.8 of the usable 565.5 height, leaving
552.7 for frames. Equal allocation starts with content budget 182.633333…,
rounded to 183. Accounting for its two borders leaves 368.1 to share, so the
second content budget is 182.45, rounded to 182; the final content budget is
182.9, rounded to 183. Physical visual alignment then gives the frame heights
in the table. Vertical positions accumulate visual frame sizes and the 3.2 gap,
not the pre-alignment allocation. Fixing the middle window reserves 101.6, leaving
451.1 for two Auto frames; their content budgets round to 224. Reset restores
the three-window case.

Expel runs while window 1 remains focused: it moves window 3 (the last tile)
to column 2, leaving window 1 focused in column 1. Windows 1/2 recover heights
276/277 and window 3 fills height 558 with frame height 560. The new column
retains content width 300 rather than its original client-chosen 100. Its x
position is 318.4, after the first frame at x=13.6, frame width 301.6 and gap 3.2.
Reconsuming restores ordered IDs, indices, dimensions and origins. The middle
fixed/reset sequence also retains focus. These contracts do not cover unequal
Auto weights, min/max constraints, asynchronous clients or animated trajectories.

## Recorded result

The [recorded report](../fixtures/niri-layout/geometry-contract-evidence.json) and
[log](../fixtures/niri-layout/geometry-contract-evidence.log) show eleven passed,
zero failed and zero ignored tests; the other 203 library tests were filtered out.
The reference checkout remained clean. Hashes identify the archived reference,
lockfile, maintained fixture and run log. A zero-test run is rejected even when
Cargo exits successfully. The parser suite also passed its 16 contracts after the
runner extension. No driftwm runtime code changed in this batch.
