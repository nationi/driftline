use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Logical, Physical, Point, Transform};

use driftline::config::FontWeight;

use crate::state::Driftline;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};

const WIDTH: i32 = 124;
const HEIGHT: i32 = 34;
const BORDER: i32 = 2;
const FONT_PX: f32 = 12.0;

#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
    tiling: bool,
    scale: i32,
    font: String,
    weight: FontWeight,
}

pub(crate) struct TilingModeIndicatorCache {
    key: CacheKey,
    buffer: MemoryRenderBuffer,
}

pub(crate) fn build_elements(
    state: &mut Driftline,
    renderer: &mut GlesRenderer,
    output: &Output,
) -> Vec<OutputRenderElements> {
    #[cfg(test)]
    if !state.render_mode_cues_in_tests {
        return Vec::new();
    }

    let output_name = output.name();
    let Some(cue) = state.tiling_mode_cues.get(&output_name) else {
        return Vec::new();
    };

    let buffer_scale = state.decoration_scale.max(1);
    let font = state.config.decorations.font.clone();
    let weight = state.config.decorations.font_weight;
    let key = CacheKey {
        tiling: cue.tiling,
        scale: buffer_scale,
        font: font.clone(),
        weight,
    };
    let cache = state
        .render
        .cached_tiling_mode_indicators
        .entry(output_name)
        .or_insert_with(|| TilingModeIndicatorCache {
            key: key.clone(),
            buffer: render_buffer(cue.tiling, buffer_scale, &font, weight),
        });
    if cache.key != key {
        cache.buffer = render_buffer(cue.tiling, buffer_scale, &font, weight);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let x = ((viewport.w - WIDTH).max(0) / 2) as f64;
    let y = 16.0_f64.min((viewport.h - HEIGHT).max(0) as f64);
    let loc =
        Point::<f64, Logical>::from((x, y)).to_physical(output.current_scale().fractional_scale());
    let Ok(element) = MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        loc,
        &cache.buffer,
        Some(cue.alpha()),
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

fn render_buffer(tiling: bool, scale: i32, font: &str, weight: FontWeight) -> MemoryRenderBuffer {
    let scale = scale.max(1);
    let (pixel_width, pixel_height) = (WIDTH * scale, HEIGHT * scale);
    let border = BORDER * scale;
    let accent = if tiling {
        [122, 170, 255, 255]
    } else {
        [185, 166, 216, 255]
    };
    let mut pixels = vec![0; (pixel_width * pixel_height * 4) as usize];
    for y in 0..pixel_height {
        for x in 0..pixel_width {
            let color = if x < border
                || y < border
                || x >= pixel_width - border
                || y >= pixel_height - border
            {
                accent
            } else {
                [22, 25, 35, 232]
            };
            let offset = ((y * pixel_width + x) * 4) as usize;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }

    let label = if tiling { "TILING MODE" } else { "CANVAS MODE" };
    let font_px = FONT_PX * scale as f32;
    driftline::text::rasterize_into(
        &mut pixels,
        pixel_width,
        pixel_height,
        label,
        font,
        font_px,
        weight,
        [244, 246, 252, 255],
        BORDER * scale + 10 * scale,
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
