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

use crate::state::Driftline;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};

const WIDTH: i32 = 132;
const HEIGHT: i32 = 34;
const BORDER: i32 = 2;
const PAD_X: i32 = 8;
const FONT_PX: f32 = 12.0;

#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
    width: i32,
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
    let buffer_scale = state.decoration_scale.max(1);
    let displayed_width = (width.saturating_add(5) / 10 * 10).max(10);
    let font = state.config.decorations.font.clone();
    let weight = state.config.decorations.font_weight;
    let key = CacheKey {
        width: displayed_width,
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
            buffer: render_buffer(displayed_width, buffer_scale, &font, weight),
        });
    if cache.key != key {
        cache.buffer = render_buffer(displayed_width, buffer_scale, &font, weight);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let pointer = state.seat.get_pointer().unwrap().current_location();
    let pointer_screen = canvas_to_screen(CanvasPos(pointer), camera, zoom).0;
    let margin: f64 = 8.0;
    let max_x = (viewport.w - WIDTH).max(0) as f64;
    let max_y = (viewport.h - HEIGHT).max(0) as f64;
    let x = (pointer_screen.x + 20.0).clamp(margin.min(max_x), max_x);
    let above = pointer_screen.y - HEIGHT as f64 - 16.0;
    let y = if above >= margin {
        above.min(max_y)
    } else {
        (pointer_screen.y + 24.0).min(max_y)
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

fn render_buffer(width: i32, scale: i32, font: &str, weight: FontWeight) -> MemoryRenderBuffer {
    let scale = scale.max(1);
    let (pixel_width, pixel_height) = (WIDTH * scale, HEIGHT * scale);
    let border = BORDER * scale;
    let mut pixels = vec![0; (pixel_width * pixel_height * 4) as usize];
    for y in 0..pixel_height {
        for x in 0..pixel_width {
            let color = if x < border
                || y < border
                || x >= pixel_width - border
                || y >= pixel_height - border
            {
                [122, 170, 255, 255]
            } else {
                [22, 25, 35, 232]
            };
            let offset = ((y * pixel_width + x) * 4) as usize;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }

    let label = format!("Column {width} px");
    let available = pixel_width - (PAD_X + BORDER) * scale * 2;
    let font_px = FONT_PX * scale as f32;
    let (label, _) = driftline::text::fit_text(&label, font, font_px, weight, available);
    driftline::text::rasterize_into(
        &mut pixels,
        pixel_width,
        pixel_height,
        &label,
        font,
        font_px,
        weight,
        [244, 246, 252, 255],
        (PAD_X + BORDER) * scale,
    );

    MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Abgr8888,
        (pixel_width, pixel_height),
        scale,
        Transform::Normal,
        None,
    )
}
