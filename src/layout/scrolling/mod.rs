//! Scrolling-column geometry, independent of client commits and viewport motion.
//! The caller supplies stage-owned column order; this module never owns windows.

use std::collections::HashSet;

use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::canvas::Chrome;
use crate::stage::ElementId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// Visual frame width in logical pixels.
    Fixed(i32),
    /// Fraction of the usable width, before client constraints are applied.
    Proportion(f64),
}

/// Client content constraints; zero means unconstrained, minimum wins conflicts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Constraints {
    pub min: Size<i32, Logical>,
    pub max: Size<i32, Logical>,
}

/// Per-window content limits and compositor decorations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub id: ElementId,
    pub constraints: Constraints,
    pub chrome: Chrome,
}

/// One shared width policy and an ordered, nonempty set of windows.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub width: ColumnWidth,
    pub tiles: Vec<Tile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: ElementId,
    pub frame: Rectangle<i32, Logical>,
    pub content: Rectangle<i32, Logical>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    EmptyColumn,
    InvalidWorkingArea,
    InvalidGap,
    InvalidWidth,
    InvalidChrome,
    InvalidConstraints,
    DuplicateWindow(ElementId),
    CoordinateOverflow,
}

/// Arrange vertical groups, sharing height equally except where client limits
/// require a different allocation. If minimum heights do not fit, overflow
/// downward rather than overlap windows or violate their minimum sizes.
pub fn arrange_columns(
    columns: &[Column],
    origin: Point<i32, Logical>,
    usable_size: Size<i32, Logical>,
    gap: i32,
) -> Result<Vec<Placement>, LayoutError> {
    if usable_size.w <= 0 || usable_size.h <= 0 {
        return Err(LayoutError::InvalidWorkingArea);
    }
    if gap < 0 {
        return Err(LayoutError::InvalidGap);
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let mut x = i64::from(origin.x);
    for column in columns {
        if column.tiles.is_empty() {
            return Err(LayoutError::EmptyColumn);
        }
        let mut width = 1;
        let mut chrome_heights = Vec::with_capacity(column.tiles.len());
        let mut limits = Vec::with_capacity(column.tiles.len());
        for tile in &column.tiles {
            if !seen.insert(tile.id) {
                return Err(LayoutError::DuplicateWindow(tile.id));
            }
            let placement = arrange_tile(tile, column.width, (0, 0).into(), usable_size)?;
            width = width.max(placement.frame.size.w);
            chrome_heights.push(i64::from(tile.chrome.bar) + 2 * i64::from(tile.chrome.border));
            let min = constrained(0, tile.constraints.min.h, tile.constraints.max.h)?;
            let max = constrained(
                i64::from(i32::MAX),
                tile.constraints.min.h,
                tile.constraints.max.h,
            )?;
            limits.push((min, max));
        }
        let budget = i64::from(usable_size.h)
            - chrome_heights.iter().sum::<i64>()
            - i64::from(gap) * (column.tiles.len() as i64 - 1);
        let heights = distribute_height(&limits, budget);
        let mut y = i64::from(origin.y);
        for ((tile, height), chrome_height) in column.tiles.iter().zip(heights).zip(chrome_heights)
        {
            let content_width = constrained(
                i64::from(width) - 2 * i64::from(tile.chrome.border),
                tile.constraints.min.w,
                tile.constraints.max.w,
            )?;
            let frame_width = content_width + 2 * i64::from(tile.chrome.border);
            result.push(Placement {
                id: tile.id,
                frame: rectangle(x, y, frame_width, height + chrome_height)?,
                content: rectangle(
                    x + i64::from(tile.chrome.border),
                    y + i64::from(tile.chrome.bar) + i64::from(tile.chrome.border),
                    content_width,
                    height,
                )?,
            });
            y += height + chrome_height + i64::from(gap);
        }
        x += i64::from(width) + i64::from(gap);
    }
    Ok(result)
}

fn distribute_height(limits: &[(i64, i64)], budget: i64) -> Vec<i64> {
    let minimum: i64 = limits.iter().map(|(min, _)| min).sum();
    let maximum: i64 = limits.iter().map(|(_, max)| max).sum();
    let budget = budget.clamp(minimum, maximum);
    let mut low = 0;
    let mut high = limits.iter().map(|(_, max)| *max).max().unwrap_or(0);
    // Find the common height before rounding. Freezing minima or maxima first
    // can leave usable space stranded when both kinds of limits are present.
    while low < high {
        let middle = low + (high - low + 1) / 2;
        let total: i64 = limits
            .iter()
            .map(|(min, max)| middle.clamp(*min, *max))
            .sum();
        if total <= budget {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let mut heights: Vec<_> = limits
        .iter()
        .map(|(min, max)| low.clamp(*min, *max))
        .collect();
    let mut remaining = budget - heights.iter().sum::<i64>();
    let mut count = limits
        .iter()
        .filter(|(min, max)| *min <= low && low < *max)
        .count() as i64;
    for (height, (min, max)) in heights.iter_mut().zip(limits) {
        if *min <= low && low < *max {
            // Round each share from the remaining budget, matching sequential
            // rounding of integer client sizes rather than front-loading pixels.
            if remaining * 2 >= count {
                *height += 1;
                remaining -= 1;
            }
            count -= 1;
        }
    }
    heights
}

fn arrange_tile(
    tile: &Tile,
    width_policy: ColumnWidth,
    origin: Point<i32, Logical>,
    usable_size: Size<i32, Logical>,
) -> Result<Placement, LayoutError> {
    let x = i64::from(origin.x);
    let Chrome { bar, border } = tile.chrome;
    if bar < 0 || border < 0 {
        return Err(LayoutError::InvalidChrome);
    }
    let horizontal_chrome = 2 * i64::from(border);
    let vertical_chrome = horizontal_chrome + i64::from(bar);
    let width = match width_policy {
        ColumnWidth::Fixed(width) if width > 0 => i64::from(width),
        ColumnWidth::Proportion(fraction) if fraction.is_finite() && fraction > 0.0 => {
            let width = (f64::from(usable_size.w) * fraction).round();
            if !width.is_finite() || width > f64::from(i32::MAX) {
                return Err(LayoutError::CoordinateOverflow);
            }
            (width as i64).max(1)
        }
        _ => return Err(LayoutError::InvalidWidth),
    };
    let constraints = tile.constraints;
    let content_width = constrained(
        width - horizontal_chrome,
        constraints.min.w,
        constraints.max.w,
    )?;
    let content_height = constrained(
        i64::from(usable_size.h) - vertical_chrome,
        constraints.min.h,
        constraints.max.h,
    )?;
    let frame_width = content_width + horizontal_chrome;
    let frame_height = content_height + vertical_chrome;
    let frame = rectangle(x, i64::from(origin.y), frame_width, frame_height)?;
    let content = rectangle(
        x + i64::from(border),
        i64::from(origin.y) + i64::from(bar) + i64::from(border),
        content_width,
        content_height,
    )?;
    Ok(Placement {
        id: tile.id,
        frame,
        content,
    })
}

/// Minimal camera movement to reveal a frame at zoom 1. Oversized frames align
/// their leading edge with the usable area instead of alternating between edges.
/// `usable` is output-local, including its layer-shell/outer-gap offset.
pub fn reveal(
    frame: Rectangle<i32, Logical>,
    usable: Rectangle<i32, Logical>,
    camera: Point<i32, Logical>,
) -> Result<Point<i32, Logical>, LayoutError> {
    if usable.size.w <= 0 || usable.size.h <= 0 || frame.size.w <= 0 || frame.size.h <= 0 {
        return Err(LayoutError::InvalidWorkingArea);
    }
    let axis = |start: i32, size: i32, inset: i32, extent: i32, camera: i32| {
        let start = i64::from(start);
        let inset = i64::from(inset);
        let visible_start = i64::from(camera) + inset;
        let visible_end = visible_start + i64::from(extent);
        let end = start + i64::from(size);
        let target = if size >= extent || start < visible_start {
            start - inset
        } else if end > visible_end {
            end - i64::from(extent) - inset
        } else {
            i64::from(camera)
        };
        coordinate(target)
    };
    Ok(Point::from((
        axis(
            frame.loc.x,
            frame.size.w,
            usable.loc.x,
            usable.size.w,
            camera.x,
        )?,
        axis(
            frame.loc.y,
            frame.size.h,
            usable.loc.y,
            usable.size.h,
            camera.y,
        )?,
    )))
}

fn constrained(requested: i64, min: i32, max: i32) -> Result<i64, LayoutError> {
    if min < 0 || max < 0 {
        return Err(LayoutError::InvalidConstraints);
    }
    let min = i64::from(min).max(1);
    let max = if max == 0 {
        i64::from(i32::MAX)
    } else {
        i64::from(max).max(min)
    };
    Ok(requested.clamp(min, max))
}

fn coordinate(value: i64) -> Result<i32, LayoutError> {
    i32::try_from(value).map_err(|_| LayoutError::CoordinateOverflow)
}

fn rectangle(x: i64, y: i64, w: i64, h: i64) -> Result<Rectangle<i32, Logical>, LayoutError> {
    // Smithay rectangle operations add origin and size in i32.
    coordinate(x + w)?;
    coordinate(y + h)?;
    Ok(Rectangle::new(
        Point::from((coordinate(x)?, coordinate(y)?)),
        Size::from((coordinate(w)?, coordinate(h)?)),
    ))
}

#[cfg(test)]
mod tests;
