use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Logical, Physical, Point};

use driftline::config::FontWeight;

use crate::state::Driftline;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};

#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
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

    let buffer_scale = output.current_scale().fractional_scale().ceil() as i32;
    let font = state.config.decorations.font.clone();
    let weight = state.config.decorations.font_weight;
    let label = "TILING MODE";
    let (width, height) = super::tiling_badge::logical_size(label, &font, weight);
    let key = CacheKey {
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
            buffer: super::tiling_badge::render_buffer(label, &font, weight, buffer_scale),
        });
    if cache.key != key {
        cache.buffer = super::tiling_badge::render_buffer(label, &font, weight, buffer_scale);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let x = ((viewport.w - width).max(0) / 2) as f64;
    let y = 16.0_f64.min((viewport.h - height).max(0) as f64);
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
