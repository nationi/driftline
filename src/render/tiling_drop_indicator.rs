use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Logical, Physical, Point};

use driftline::canvas::{CanvasPos, canvas_to_screen};
use driftline::config::FontWeight;

use super::elements::{OutputRenderElements, PixelSnapRescaleElement};
use crate::state::Driftline;

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
    let scale = output.current_scale().fractional_scale().ceil() as i32;
    let (badge_width, badge_height) = super::tiling_badge::logical_size(label, &font, weight);
    let key = CacheKey {
        label: label.to_owned(),
        scale,
        font: font.clone(),
        weight,
    };
    let cache = state
        .render
        .cached_tiling_drop_indicators
        .entry(output_name.clone())
        .or_insert_with(|| TilingDropIndicatorCache {
            key: key.clone(),
            buffer: super::tiling_badge::render_buffer(label, &font, weight, scale),
        });
    if cache.key != key {
        cache.buffer = super::tiling_badge::render_buffer(label, &font, weight, scale);
        cache.key = key;
    }

    let viewport = crate::state::output_logical_size(output);
    let pointer_screen = canvas_to_screen(CanvasPos(pointer), camera, zoom).0;
    let margin: f64 = 10.0;
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
