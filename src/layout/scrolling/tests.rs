use proptest::prelude::*;

use super::*;

#[derive(Clone, Copy)]
struct WindowInput {
    id: ElementId,
    width: ColumnWidth,
    constraints: Constraints,
    chrome: Chrome,
}

fn tile(input: &WindowInput) -> Tile {
    Tile {
        id: input.id,
        constraints: input.constraints,
        chrome: input.chrome,
    }
}

fn arrange(
    inputs: &[WindowInput],
    origin: Point<i32, Logical>,
    size: Size<i32, Logical>,
    gap: i32,
) -> Result<Vec<Placement>, LayoutError> {
    let columns: Vec<_> = inputs
        .iter()
        .map(|input| Column {
            width: input.width,
            tiles: vec![tile(input)],
        })
        .collect();
    arrange_columns(&columns, origin, size, gap)
}

fn arrange_groups(
    inputs: &[Vec<WindowInput>],
    origin: Point<i32, Logical>,
    size: Size<i32, Logical>,
    gap: i32,
) -> Result<Vec<Placement>, LayoutError> {
    let columns: Vec<_> = inputs
        .iter()
        .map(|group| {
            let width = group
                .first()
                .map_or(ColumnWidth::Proportion(0.5), |input| input.width);
            assert!(group.iter().all(|input| input.width == width));
            Column {
                width,
                tiles: group.iter().map(tile).collect(),
            }
        })
        .collect();
    arrange_columns(&columns, origin, size, gap)
}

fn column(id: u64, width: ColumnWidth) -> WindowInput {
    WindowInput {
        id: ElementId(id),
        width,
        constraints: Constraints::default(),
        chrome: Chrome::NONE,
    }
}

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

#[test]
fn column_width_is_shared_while_client_limits_remain_per_tile() {
    let columns = [
        Column {
            width: ColumnWidth::Fixed(300),
            tiles: vec![
                Tile {
                    id: ElementId(1),
                    constraints: Constraints {
                        min: (500, 0).into(),
                        ..Default::default()
                    },
                    chrome: Chrome { bar: 20, border: 2 },
                },
                Tile {
                    id: ElementId(2),
                    constraints: Constraints {
                        max: (200, 0).into(),
                        ..Default::default()
                    },
                    chrome: Chrome::NONE,
                },
            ],
        },
        Column {
            width: ColumnWidth::Fixed(100),
            tiles: vec![tile(&column(3, ColumnWidth::Fixed(100)))],
        },
    ];
    let result = arrange_columns(&columns, (0, 0).into(), (1000, 700).into(), 10).unwrap();
    assert_eq!(result[0].frame.size.w, 504);
    assert_eq!(result[1].frame.size.w, 200);
    assert_eq!(result[2].frame.loc.x, 514);
    assert_eq!(result[0].content.size.h, result[1].content.size.h);
}

#[test]
fn inserting_a_column_does_not_shrink_existing_columns() {
    let first = column(1, ColumnWidth::Proportion(0.5));
    let last = column(2, ColumnWidth::Fixed(300));
    let origin = (-200, 50).into();
    let size = (1000, 700).into();
    let before = arrange(&[first, last], origin, size, 12).unwrap();
    let after = arrange(
        &[first, column(3, ColumnWidth::Fixed(400)), last],
        origin,
        size,
        12,
    )
    .unwrap();
    assert_eq!(before[0], after[0]);
    assert_eq!(after[0].frame, rect(-200, 50, 500, 700));
    assert_eq!(after[1].frame, rect(312, 50, 400, 700));
    assert_eq!(after[2].frame, rect(724, 50, 300, 700));
    assert_eq!(before[1].content.size, after[2].content.size);
}

#[test]
fn decorations_are_inside_the_column_and_gaps_are_between_frames() {
    let mut decorated = column(1, ColumnWidth::Fixed(500));
    decorated.chrome = Chrome { bar: 30, border: 2 };
    let result = arrange(
        &[decorated, column(2, ColumnWidth::Fixed(200))],
        (100, 80).into(),
        (1000, 700).into(),
        8,
    )
    .unwrap();
    assert_eq!(result[0].frame, rect(100, 80, 500, 700));
    assert_eq!(result[0].content, rect(102, 112, 496, 666));
    assert_eq!(result[1].frame.loc.x, 608);
}

#[test]
fn minimum_constraints_can_overflow_without_overlapping_the_next_column() {
    let mut first = column(1, ColumnWidth::Fixed(100));
    first.constraints.min = (1200, 900).into();
    first.constraints.max = (200, 300).into();
    let result = arrange(
        &[first, column(2, ColumnWidth::Fixed(100))],
        (0, 0).into(),
        (1000, 700).into(),
        10,
    )
    .unwrap();
    assert_eq!(result[0].content.size, (1200, 900).into());
    assert_eq!(result[1].frame.loc.x, 1210);
}

#[test]
fn maximum_constraints_leave_unused_space_in_the_working_area() {
    let mut tile = column(1, ColumnWidth::Proportion(1.0));
    tile.constraints.max = (300, 250).into();
    let result = arrange(&[tile], (0, 0).into(), (1000, 700).into(), 0).unwrap();
    assert_eq!(result[0].content, rect(0, 0, 300, 250));
}

#[test]
fn working_area_changes_resize_proportional_columns_but_keep_fixed_widths() {
    let columns = [
        column(1, ColumnWidth::Proportion(0.5)),
        column(2, ColumnWidth::Fixed(350)),
    ];
    let before = arrange(&columns, (0, 0).into(), (1000, 700).into(), 10).unwrap();
    let after = arrange(&columns, (0, 0).into(), (800, 600).into(), 10).unwrap();
    assert_eq!(before[0].frame.size, (500, 700).into());
    assert_eq!(after[0].frame.size, (400, 600).into());
    assert_eq!(before[1].frame.size.w, after[1].frame.size.w);
    assert_eq!(after[1].frame.loc.x, 410);
}

#[test]
fn tiny_dimensions_still_request_positive_content() {
    let mut tile = column(1, ColumnWidth::Proportion(0.001));
    tile.chrome = Chrome { bar: 30, border: 2 };
    let result = arrange(&[tile], (0, 0).into(), (1, 1).into(), 0).unwrap();
    assert_eq!(result[0].content.size, (1, 1).into());
    assert_eq!(result[0].frame.size, (5, 35).into());
}

#[test]
fn invalid_input_and_unrepresentable_coordinates_are_rejected() {
    let origin = (0, 0).into();
    let size = (1000, 700).into();
    let tile = column(1, ColumnWidth::Fixed(500));
    assert_eq!(arrange(&[], origin, size, 0), Ok(vec![]));
    assert_eq!(
        arrange(&[tile, tile], origin, size, 0),
        Err(LayoutError::DuplicateWindow(tile.id))
    );
    assert_eq!(
        arrange(&[tile], origin, size, -1),
        Err(LayoutError::InvalidGap)
    );
    assert_eq!(
        arrange(&[tile], origin, (0, 1).into(), 0),
        Err(LayoutError::InvalidWorkingArea)
    );
    for fraction in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
        assert_eq!(
            arrange(
                &[column(1, ColumnWidth::Proportion(fraction))],
                origin,
                size,
                0
            ),
            Err(LayoutError::InvalidWidth)
        );
    }
    assert_eq!(
        arrange(&[column(1, ColumnWidth::Fixed(0))], origin, size, 0),
        Err(LayoutError::InvalidWidth)
    );
    assert_eq!(
        arrange(
            &[column(1, ColumnWidth::Proportion(f64::MAX))],
            origin,
            size,
            0
        ),
        Err(LayoutError::CoordinateOverflow)
    );
    assert_eq!(
        arrange(&[tile], (i32::MAX - 1, 0).into(), size, 0),
        Err(LayoutError::CoordinateOverflow)
    );
    let mut invalid = tile;
    invalid.chrome.border = -1;
    assert_eq!(
        arrange(&[invalid], origin, size, 0),
        Err(LayoutError::InvalidChrome)
    );
    invalid = tile;
    invalid.constraints.min.w = -1;
    assert_eq!(
        arrange(&[invalid], origin, size, 0),
        Err(LayoutError::InvalidConstraints)
    );
    invalid = tile;
    invalid.chrome.bar = i32::MAX;
    assert_eq!(
        arrange(&[invalid], origin, size, 0),
        Err(LayoutError::CoordinateOverflow)
    );
}

#[test]
fn reveal_respects_reserved_screen_edges_and_preserves_a_visible_frame() {
    let usable = rect(20, 40, 960, 640);
    let camera = (-100, -200).into();
    assert_eq!(reveal(rect(0, 0, 300, 300), usable, camera), Ok(camera));
    assert_eq!(
        reveal(rect(800, 300, 300, 300), usable, camera),
        Ok((120, -80).into())
    );
    assert_eq!(
        reveal(rect(-200, -300, 300, 300), usable, camera),
        Ok((-220, -340).into())
    );
}

#[test]
fn oversized_frames_align_the_leading_edge_and_settle() {
    let frame = rect(-300, 100, 2000, 1500);
    let usable = rect(20, 40, 960, 640);
    let target = reveal(frame, usable, (800, 600).into()).unwrap();
    assert_eq!(target, (-320, 60).into());
    assert_eq!(reveal(frame, usable, target), Ok(target));
    assert_eq!(
        reveal(frame, rect(0, 0, 0, 100), target),
        Err(LayoutError::InvalidWorkingArea)
    );
    assert_eq!(
        reveal(rect(i32::MIN, 0, 100, 100), usable, target),
        Err(LayoutError::CoordinateOverflow)
    );
}

proptest! {
    #[test]
    fn bounded_height_allocation_conserves_the_available_budget(
        limits in prop::collection::vec((1i64..200, 0i64..500), 1..20),
        budget in 0i64..5000,
    ) {
        let limits: Vec<_> = limits.into_iter().map(|(min, extra)| (min, min + extra)).collect();
        let heights = distribute_height(&limits, budget);
        let minimum: i64 = limits.iter().map(|(min, _)| min).sum();
        let maximum: i64 = limits.iter().map(|(_, max)| max).sum();
        prop_assert_eq!(heights.iter().sum::<i64>(), budget.clamp(minimum, maximum));
        for (height, (min, max)) in heights.iter().zip(limits) {
            prop_assert!((min..=max).contains(height));
        }
    }

    #[test]
    fn grouped_layout_has_no_overlaps_and_is_deterministic(
        sizes in prop::collection::vec((1i32..800, prop::collection::vec((0i32..900, 0i32..900), 1..8)), 0..8),
        working_height in 1i32..1200,
        gap in 0i32..30,
    ) {
        let mut id = 0;
        let groups: Vec<Vec<_>> = sizes.iter().map(|(width, group)| group.iter().map(|(min, max)| {
            id += 1;
            let mut tile = column(id, ColumnWidth::Fixed(*width));
            tile.constraints.min.h = *min;
            tile.constraints.max.h = *max;
            tile.chrome = Chrome { bar: 20, border: 2 };
            tile
        }).collect()).collect();
        let result = arrange_groups(&groups, (-100, -200).into(), (1000, working_height).into(), gap).unwrap();
        prop_assert_eq!(&result, &arrange_groups(&groups, (-100, -200).into(), (1000, working_height).into(), gap).unwrap());
        for (i, placement) in result.iter().enumerate() {
            prop_assert!(placement.content.size.w > 0 && placement.content.size.h > 0);
            for other in &result[i + 1..] {
                prop_assert!(!placement.frame.overlaps(other.frame));
            }
        }
        for (spec, placement) in groups.iter().flatten().zip(&result) {
            prop_assert!(placement.content.size.h >= spec.constraints.min.h.max(1));
            if spec.constraints.max.h > 0 {
                prop_assert!(placement.content.size.h <= spec.constraints.max.h.max(spec.constraints.min.h).max(1));
            }
        }
    }

    #[test]
    fn arbitrary_inputs_never_panic(
        values in any::<[i32; 11]>(),
        fraction in any::<f64>(),
    ) {
        let [x, y, w, h, gap, bar, border, min_w, min_h, max_w, max_h] = values;
        let (w, h) = (w.max(0), h.max(0));
        let (min_w, min_h) = (min_w.max(0), min_h.max(0));
        let (max_w, max_h) = (max_w.max(0), max_h.max(0));
        let mut tile = column(1, ColumnWidth::Proportion(fraction));
        tile.chrome = Chrome { bar, border };
        tile.constraints = Constraints {
            min: (min_w, min_h).into(),
            max: (max_w, max_h).into(),
        };
        let _ = arrange(&[tile], (x, y).into(), (w, h).into(), gap);
        let _ = reveal(rect(x, y, w, h), rect(bar, border, min_w, min_h), (max_w, max_h).into());
    }

    #[test]
    fn columns_remain_ordered_and_revealing_a_tile_settles(
        widths in prop::collection::vec(1i32..4000, 0..50),
        usable_width in 1i32..4000,
        usable_height in 1i32..2000,
        gap in 0i32..100,
        origin_x in -100_000i32..100_000,
        origin_y in -100_000i32..100_000,
        bar in 0i32..60,
        border in 0i32..10,
    ) {
        let columns: Vec<_> = widths.iter().enumerate().map(|(id, width)| {
            let mut tile = column(id as u64, ColumnWidth::Fixed(*width));
            tile.chrome = Chrome { bar, border };
            tile
        }).collect();
        let size = (usable_width, usable_height).into();
        let origin = (origin_x, origin_y).into();
        let result = arrange(&columns, origin, size, gap).unwrap();
        prop_assert_eq!(&result, &arrange(&columns, origin, size, gap).unwrap());
        prop_assert_eq!(result.len(), widths.len());
        for pair in result.windows(2) {
            prop_assert_eq!(pair[1].frame.loc.x, pair[0].frame.loc.x + pair[0].frame.size.w + gap);
        }
        for (tile, placement) in columns.iter().zip(&result) {
            prop_assert!(placement.content.size.w > 0 && placement.content.size.h > 0);
            prop_assert_eq!(placement.frame.size, tile.chrome.frame_size(placement.content.size));
            prop_assert_eq!(placement.content.loc, tile.chrome.content_loc(placement.frame.loc));
            let usable = Rectangle::new((15, 30).into(), size);
            let target = reveal(placement.frame, usable, (0, 0).into()).unwrap();
            prop_assert_eq!(reveal(placement.frame, usable, target), Ok(target));
            if placement.frame.size.w <= usable_width {
                prop_assert!(placement.frame.loc.x >= target.x + usable.loc.x);
                prop_assert!(placement.frame.loc.x + placement.frame.size.w <= target.x + usable.loc.x + usable_width);
            }
        }
    }
}

#[test]
fn grouped_windows_share_height_and_the_next_column_starts_after_the_widest_tile() {
    let groups = vec![
        vec![
            column(1, ColumnWidth::Fixed(500)),
            column(2, ColumnWidth::Fixed(500)),
        ],
        vec![column(3, ColumnWidth::Fixed(200))],
    ];
    let result = arrange_groups(&groups, (0, 0).into(), (1000, 701).into(), 10).unwrap();
    assert_eq!(result[0].frame, rect(0, 0, 500, 346));
    assert_eq!(result[1].frame, rect(0, 356, 500, 345));
    assert_eq!(result[2].frame, rect(510, 0, 200, 701));
}

#[test]
fn height_distribution_respects_limits_and_redistributes_remaining_space() {
    let mut a = column(1, ColumnWidth::Fixed(300));
    a.constraints.min.h = 450;
    let mut b = column(2, ColumnWidth::Fixed(300));
    b.constraints.max.h = 50;
    let c = column(3, ColumnWidth::Fixed(300));
    let result = arrange_groups(&[vec![a, b, c]], (0, 0).into(), (1000, 600).into(), 0).unwrap();
    assert_eq!(
        result.iter().map(|p| p.content.size.h).collect::<Vec<_>>(),
        vec![450, 50, 100]
    );
    assert_eq!(result[2].frame.loc.y, 500);
}

#[test]
fn minimum_heights_overflow_downward_and_decorations_consume_height() {
    let mut a = column(1, ColumnWidth::Fixed(300));
    a.constraints.min.h = 500;
    a.chrome = Chrome { bar: 20, border: 2 };
    let mut b = a;
    b.id = ElementId(2);
    let result = arrange_groups(&[vec![a, b]], (0, 0).into(), (1000, 700).into(), 10).unwrap();
    assert_eq!(result[0].content.size.h, 500);
    assert_eq!(result[0].frame.size.h, 524);
    assert_eq!(result[1].frame.loc.y, 534);
}

#[test]
fn grouped_layout_rejects_empty_columns_and_duplicate_membership() {
    let tile = column(1, ColumnWidth::Fixed(300));
    assert_eq!(
        arrange_groups(&[vec![]], (0, 0).into(), (1000, 700).into(), 0),
        Err(LayoutError::EmptyColumn)
    );
    assert_eq!(
        arrange_groups(
            &[vec![tile], vec![tile]],
            (0, 0).into(),
            (1000, 700).into(),
            0
        ),
        Err(LayoutError::DuplicateWindow(tile.id))
    );
}
