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
constraints (except in the height-bound cases below), titlebars or tab indicators.
The helper explicitly passes the
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
Scale transitions, broader stacking/consume/expel sequences, broader constraints,
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
fixed/reset sequence also retains focus. These three sequence contracts do not
cover asynchronous clients or animated trajectories.

## Height constraints and Auto weights

Four more contracts retain the same fractional three-window layout. Only the
middle window advertises height constraints; width remains unconstrained.
The simulated clients accept requested sizes, so these assert reference layout
allocation rather than real toolkits' independent constraint enforcement.

| Middle height constraints/action | Client heights | Visual frame heights | Frame y positions |
| --- | --- | --- | --- |
| min 300, max unspecified | 124, 300, 124 | 125.6, 301.6, 125.6 | 8.8, 137.6, 442.4 |
| min unspecified, max 100, Auto | 183, 182, 183 | 184.8, 184, 184.8 | 8.8, 196.8, 384 |
| same max 100, explicit fixed request 500 | 224, 100, 224 | 225.6, 101.6, 225.6 | 8.8, 237.6, 342.4 |
| min=max=100, Auto | 224, 100, 224 | 225.6, 101.6, 225.6 | 8.8, 237.6, 342.4 |

The min-300 case reserves a 301.6 frame, leaving 251.1 for two Auto frames;
content budgets round to 124. Niri deliberately ignores a nonfixed maximum in
Auto allocation, while explicit sizing clamps to it. An equal min/max constraint
is treated as a fixed height even without an explicit height operation. This
bounded contract is not a claim that all max constraints apply to all sizing paths.

The fourth contract starts unconstrained, sets window 1 to 100, then window 2 to
100, then resets window 2. Each new fixed-height target converts existing visual
frame heights into Auto weights before replacing that target's height:

| Action | Client heights | Visual frame heights | Frame y positions |
| --- | --- | --- | --- |
| Window 1 fixed 100 | 100, 223, 225 | 101.6, 224.8, 226.4 | 8.8, 113.6, 341.6 |
| Window 2 fixed 100 | 138, 100, 310 | 140, 101.6, 312 | 8.8, 152, 256.8 |
| Reset window 2 | 100, 223, 225 | 101.6, 224.8, 226.4 | 8.8, 113.6, 341.6 |

The first redistribution retains the previous frame ratio 184:184.8 for windows
2/3, not equal weights: the first content budget is 223.460737…, rounded to 223,
and the last rounds to 225. On the second resize the remaining frame weights are
101.6:226.4. Of the remaining 451.1, window 1 receives budget 139.730975…,
leaving content 138.130975… rounded to 138; the final window rounds to 310.
Reset replaces only the target's weight with 1, preserving the other Auto weights;
it does not globally reset all three windows to equal heights. All states also
check IDs, focus, indices, widths, borders and horizontal positions. Dynamic constraint changes, tabs and client refusal remain outside these
height/weight contracts.

## Column width bounds and overconstrained height

Five further contracts use the same three-window 1.25-scale fixture, initially
requesting fixed content width 300. Constraints belong to the middle window,
except the conflict case where window 3 advertises a separate maximum.

| Width constraints | Content width of all three clients | Visual frame width |
| --- | --- | --- |
| Middle min 500, no max | 500 | 501.6 |
| Middle max 200, no min | 200 | 201.6 |
| Middle min=max=240 | 240 | 241.6 |
| Middle min 500 and window 3 max 200 | 500 | 501.6 |

The shared column width uses the largest tile minimum and smallest nonzero tile
maximum; incompatible bounds resolve in favor of the largest minimum. These
clients accept the resulting requests, including a request exceeding window 3's
maximum. This is the reference allocation policy, not simulated toolkit refusal.
All four cases retain client heights 183/182/183 and visual frame heights
184.8/184/184.8. Frame x=13.6 and y=8.8/196.8/384, order, indices and focus on
window 1 remain unchanged. Frame widths add two 0.8 borders to content widths
that already align to physical pixels.

The fifth case sets the middle minimum height to 600, exceeding usable 565.5.
It preserves that minimum and requests one logical content pixel for each other
window: client heights 1/600/1. At 1.25 those one-pixel contents visually round
to 0.8 logical, so visual frame heights are 2.4/601.6/2.4. Their y positions
are 8.8/14.4/619.2; the final window lies below the physical output. Geometry is
not shrunk to fit. This contract measures layout coordinates, not rendering or
visibility/clipping of the offscreen window. Contradictory bounds within a single
client, dynamic changes, asynchronous response order and width refusal remain open.

## Recorded result

The [recorded report](../fixtures/niri-layout/geometry-contract-evidence.json) and
[log](../fixtures/niri-layout/geometry-contract-evidence.log) show twenty passed,
zero failed and zero ignored tests; the other 203 library tests were filtered out.
The reference checkout remained clean. Hashes identify the archived reference,
lockfile, maintained fixture and run log. A zero-test run is rejected even when
Cargo exits successfully. The parser suite also passed its 16 contracts after the
runner extension. No driftwm runtime code changed in this batch.
