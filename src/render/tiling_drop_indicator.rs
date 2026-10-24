use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Logical, Physical, Point, Transform};

use driftline::canvas::{CanvasPos, canvas_to_screen};
use driftline::config::FontWeight;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};
use crate::state::Driftline;

const WIDTH: i32 = 164;
const HEIGHT: i32 = 34;
const SCALE: i32 = 2;

#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
    label: String,
    scale: i32,
    font: String,
    weight: FontWeight,
}

pub(crate) struct TilingDropIndicatorCache {
    key: CacheKey,
    buffer: MemoryRenderBuffer,
}

pub(crate) fn build_elements(
    state: &mut Driftline,
    renderer: &mut GlesRenderer,
    output: &Output,
    camera: Point<f64, Logical>,
    zoom: f64,
) -> Vec<OutputRenderElements> {
    let transfer_output = state
        .tiling_drag_visual
        .as_ref()
        .and_then(|(source, name, _)| {
            (state.stage.tiling_output_for_window(*source) != Some(name.as_str())).then_some(name)
        });
    let Some(output_name) = state
        .tiling_drop_target
        .as_ref()
        .map(|(_, name)| name)
        .or(transfer_output)
    else {
        return Vec::new();
    };
    if output.name() != *output_name {
        return Vec::new();
    }
    let Some(pointer) = state
        .seat
        .get_pointer()
        .map(|pointer| pointer.current_location())
    else {
        return Vec::new();
    };
    let transfer = transfer_output.is_some_and(|name| name == output_name);
    let target_bbox = state
        .tiling_drop_target
        .as_ref()
        .filter(|(_, name)| name == output_name)
        .and_then(|(target, _)| state.stage.window_by_id(*target))
        .and_then(|entry| entry.client())
        .and_then(|window| state.window_bbox_with_popups(window));
    let pointer_over_target = target_bbox.is_some_and(|bbox| {
        pointer.x >= f64::from(bbox.loc.x)
            && pointer.x < f64::from(bbox.loc.x + bbox.size.w)
            && pointer.y >= f64::from(bbox.loc.y)
            && pointer.y < f64::from(bbox.loc.y + bbox.size.h)
    });
    let target_relative_x =
        target_bbox.map(|bbox| (pointer.x - f64::from(bbox.loc.x)) / f64::from(bbox.size.w.max(1)));
    let source_is_stacked = state
        .tiling_drag_visual
        .as_ref()
        .filter(|(_, source_output, _)| source_output == output_name)
        .and_then(|(source, _, _)| state.stage.column_of(*source))
        .and_then(|column| state.stage.column(column))
        .is_some_and(|column| column.windows.len() > 1);
    let label = if transfer {
        "MOVE WINDOW HERE"
    } else if pointer_over_target && target_relative_x.is_some_and(|x| x < 0.12) {
        if source_is_stacked {
            "NEW COLUMN LEFT"
        } else {
            "MOVE COLUMN LEFT"
        }
    } else if pointer_over_target && target_relative_x.is_some_and(|x| x > 0.88) {
        if source_is_stacked {
            "NEW COLUMN RIGHT"
        } else {
            "MOVE COLUMN RIGHT"
        }
    } else if !pointer_over_target
        && target_bbox.is_some()
        && let Some((source, source_output, _)) = state.tiling_drag_visual.as_ref()
        && source_output == output_name
        && let Some(source_window) = state
            .stage
            .window_by_id(*source)
            .and_then(|entry| entry.client())
        && let Some(source_bbox) = state.window_bbox_with_popups(source_window)
        && state.stage.column_of(*source).is_some_and(|column| {
            state
                .stage
                .column(column)
                .is_some_and(|column| column.windows.len() > 1)
        })
        && (pointer.x < f64::from(source_bbox.loc.x)
            || pointer.x > f64::from(source_bbox.loc.x + source_bbox.size.w))
    {
        if pointer.x < f64::from(source_bbox.loc.x) {
            "NEW COLUMN LEFT"
        } else {
            "NEW COLUMN RIGHT"
        }
    } else {
        let Some(target_bbox) = target_bbox else {
            return Vec::new();
        };
        let relative_y =
            (pointer.y - f64::from(target_bbox.loc.y)) / f64::from(target_bbox.size.h.max(1));
        if relative_y < 0.24 {
            "INSERT ABOVE"
        } else if relative_y > 0.76 {
            "INSERT BELOW"
        } else {
            "SWAP WINDOWS"
        }
    };

    let font = state.config.decorations.font.clone();
    let weight = state.config.decorations.font_weight;
    let key = CacheKey {
        label: label.to_owned(),
        scale: SCALE,
        font: font.clone(),
        weight,
    };
    let cache = state
        .render
        .cached_tiling_drop_indicators
        .entry(output_name.clone())
        .or_insert_with(|| TilingDropIndicatorCache {
            key: key.clone(),
            buffer: render_buffer(label, &font, weight),
        });
    if cache.key != key {
        cache.buffer = render_buffer(label, &font, weight);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let pointer_screen = canvas_to_screen(CanvasPos(pointer), camera, zoom).0;
    let margin: f64 = 8.0;
    let max_x = (viewport.w - WIDTH).max(0) as f64;
    let max_y = (viewport.h - HEIGHT).max(0) as f64;
    let x = (pointer_screen.x + 20.0).clamp(margin.min(max_x), max_x);
    let above = pointer_screen.y - HEIGHT as f64 - 14.0;
    let y = if above >= margin {
        above.min(max_y)
    } else {
        (pointer_screen.y + 24.0).min(max_y)
    };
    let loc = Point::<i32, Logical>::from((x.round() as i32, y.round() as i32))
        .to_f64()
        .to_physical(output.current_scale().fractional_scale());
    let Ok(element) = MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        loc,
        &cache.buffer,
        Some(1.0),
        None,
        None,
        Kind::Unspecified,
    ) else {
        return Vec::new();
    };
    vec![OutputRenderElements::Decoration(
        PixelSnapRescaleElement::from_element(element, Point::<i32, Physical>::from((0, 0)), 1.0),
    )]
}

fn render_buffer(label: &str, font: &str, weight: FontWeight) -> MemoryRenderBuffer {
    let (pixel_width, pixel_height) = (WIDTH * SCALE, HEIGHT * SCALE);
    let border = 2 * SCALE;
    let mut pixels = vec![0; (pixel_width * pixel_height * 4) as usize];
    for y in 0..pixel_height {
        for x in 0..pixel_width {
            let color = if x < border
                || y < border
                || x >= pixel_width - border
                || y >= pixel_height - border
            {
                [68, 211, 255, 255]
            } else {
                [18, 23, 32, 240]
            };
            let offset = ((y * pixel_width + x) * 4) as usize;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
    driftline::text::rasterize_into(
        &mut pixels,
        pixel_width,
        pixel_height,
        label,
        font,
        12.0 * SCALE as f32,
        weight,
        [248, 250, 255, 255],
        10 * SCALE,
    );
    MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Abgr8888,
        (pixel_width, pixel_height),
        SCALE,
        Transform::Normal,
        None,
    )
}
