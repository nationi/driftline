use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Logical, Physical, Point};

use driftline::canvas::{CanvasPos, canvas_to_screen};
use driftline::config::FontWeight;

use crate::state::Driftline;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};

#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
    label: String,
    scale: i32,
    font: String,
    weight: FontWeight,
}

pub(crate) struct TilingResizeIndicatorCache {
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
    let Some((column, output_name, width)) = state.tiling_width_indicator.as_ref() else {
        return Vec::new();
    };
    if output.name() != *output_name || state.stage.column(*column).is_none() {
        return Vec::new();
    }

    let output_scale = output.current_scale().fractional_scale();
    let displayed_width = (width.saturating_add(5) / 10 * 10).max(10);
    let label = format!("COLUMN {displayed_width} PX");
    let buffer_scale = (output_scale.ceil() as i32).max(1);
    let font = state.config.decorations.font.clone();
    let weight = state.config.decorations.font_weight;
    let key = CacheKey {
        label: label.clone(),
        scale: buffer_scale,
        font: font.clone(),
        weight,
    };
    let cache = state
        .render
        .cached_tiling_resize_indicators
        .entry(output_name.clone())
        .or_insert_with(|| TilingResizeIndicatorCache {
            key: key.clone(),
            buffer: super::tiling_badge::render_buffer(&label, &font, weight, buffer_scale),
        });
    if cache.key != key {
        cache.buffer = super::tiling_badge::render_buffer(&label, &font, weight, buffer_scale);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let pointer = state.seat.get_pointer().unwrap().current_location();
    let pointer_screen = canvas_to_screen(CanvasPos(pointer), camera, zoom).0;
    let (badge_width, badge_height) = super::tiling_badge::logical_size(&label, &font, weight);
    let margin: f64 = 8.0;
    let max_x = (viewport.w - badge_width).max(0) as f64;
    let max_y = (viewport.h - badge_height).max(0) as f64;
    let x = (pointer_screen.x + 16.0).clamp(margin.min(max_x), max_x);
    let above = pointer_screen.y - badge_height as f64 - 12.0;
    let y = if above >= margin {
        above.min(max_y)
    } else {
        (pointer_screen.y + 18.0).min(max_y)
    };
    let loc = Point::<i32, Logical>::from((x.round() as i32, y.round() as i32))
        .to_f64()
        .to_physical(output_scale);
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
