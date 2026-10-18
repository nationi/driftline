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

/// One window per column for the initial scrolling layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub id: ElementId,
    pub width: ColumnWidth,
    pub constraints: Constraints,
    pub chrome: Chrome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: ElementId,
    pub frame: Rectangle<i32, Logical>,
    pub content: Rectangle<i32, Logical>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    InvalidWorkingArea,
    InvalidGap,
    InvalidWidth,
    InvalidChrome,
    InvalidConstraints,
    DuplicateWindow(ElementId),
    CoordinateOverflow,
}

/// Arrange columns from a fixed canvas origin, independent of camera position.
/// `usable_size` excludes outer gaps and layer-shell reservations. `gap` is
/// between visual frames. Constraints may make a frame exceed the usable area;
/// subsequent columns still start after its actual target frame.
pub fn arrange(
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
    let mut placements = Vec::with_capacity(columns.len());
    let mut x = i64::from(origin.x);
    for column in columns {
        if !seen.insert(column.id) {
            return Err(LayoutError::DuplicateWindow(column.id));
        }
        let Chrome { bar, border } = column.chrome;
        if bar < 0 || border < 0 {
            return Err(LayoutError::InvalidChrome);
        }
        let horizontal_chrome = 2 * i64::from(border);
        let vertical_chrome = horizontal_chrome + i64::from(bar);
        let width = match column.width {
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
        let constraints = column.constraints;
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
        placements.push(Placement {
            id: column.id,
            frame,
            content,
        });
        x += frame_width + i64::from(gap);
    }
    Ok(placements)
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
