use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::utils::Transform;

use driftline::config::FontWeight;

const HEIGHT: i32 = 28;
const PADDING_X: i32 = 12;
const DOT_GAP: i32 = 9;
const DOT_SIZE: i32 = 7;
const FONT_SIZE: f32 = 10.5;
const CORNER_RADIUS: f32 = 9.0;
const BORDER_WIDTH: f32 = 1.0;
const BACKGROUND: [u8; 4] = [14, 20, 31, 238];
const TEXT: [u8; 4] = [242, 247, 255, 255];
pub(crate) const ACCENT: [u8; 4] = [112, 190, 255, 240];

pub(crate) fn logical_size(label: &str, font: &str, weight: FontWeight) -> (i32, i32) {
    let text_width = driftline::text::measure(label, font, FONT_SIZE, weight);
    (
        (PADDING_X * 2 + DOT_SIZE + DOT_GAP + text_width).clamp(82, 220),
        HEIGHT,
    )
}

pub(crate) fn render_buffer(
    label: &str,
    font: &str,
    weight: FontWeight,
    buffer_scale: i32,
) -> MemoryRenderBuffer {
    let buffer_scale = buffer_scale.max(1);
    let (width, height) = logical_size(label, font, weight);
    let (pixel_width, pixel_height) = (width * buffer_scale, height * buffer_scale);
    let border = (BORDER_WIDTH * buffer_scale as f32).round();
    let corner_radius = CORNER_RADIUS * buffer_scale as f32;
    let mut pixels = vec![0; (pixel_width * pixel_height * 4) as usize];

    for y in 0..pixel_height {
        for x in 0..pixel_width {
            let outer = rounded_rect_coverage(x, y, pixel_width, pixel_height, corner_radius, 0.0);
            let inner = rounded_rect_coverage(
                x,
                y,
                pixel_width,
                pixel_height,
                corner_radius - border,
                border,
            );
            let border_alpha = (outer - inner).max(0.0) * f32::from(ACCENT[3]) / 255.0;
            let background_alpha = inner * f32::from(BACKGROUND[3]) / 255.0;
            let alpha = border_alpha + background_alpha * (1.0 - border_alpha);
            let offset = ((y * pixel_width + x) * 4) as usize;
            if alpha > 0.0 {
                for channel in 0..3 {
                    let color = f32::from(ACCENT[channel]) * border_alpha
                        + f32::from(BACKGROUND[channel]) * background_alpha * (1.0 - border_alpha);
                    pixels[offset + channel] = (color / alpha).round() as u8;
                }
                pixels[offset + 3] = (alpha * 255.0).round() as u8;
            }
        }
    }

    draw_dot(&mut pixels, pixel_width, buffer_scale);
    let text_origin = (PADDING_X + DOT_SIZE + DOT_GAP) * buffer_scale;
    driftline::text::rasterize_into(
        &mut pixels,
        pixel_width,
        pixel_height,
        label,
        font,
        FONT_SIZE * buffer_scale as f32,
        weight,
        TEXT,
        text_origin,
    );

    MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Abgr8888,
        (pixel_width, pixel_height),
        buffer_scale,
        Transform::Normal,
        None,
    )
}

fn rounded_rect_coverage(x: i32, y: i32, width: i32, height: i32, radius: f32, inset: f32) -> f32 {
    let mut covered = 0.0;
    for sample_y in [0.25_f32, 0.75] {
        for sample_x in [0.25_f32, 0.75] {
            let px = x as f32 + sample_x;
            let py = y as f32 + sample_y;
            let left = inset;
            let top = inset;
            let right = width as f32 - inset;
            let bottom = height as f32 - inset;
            let nearest_x = px.clamp(left + radius, right - radius);
            let nearest_y = py.clamp(top + radius, bottom - radius);
            let dx = px - nearest_x;
            let dy = py - nearest_y;
            if dx * dx + dy * dy <= radius * radius {
                covered += 0.25;
            }
        }
    }
    covered
}

fn draw_dot(pixels: &mut [u8], width: i32, scale: i32) {
    let center = (PADDING_X + DOT_SIZE / 2) * scale;
    let radius = DOT_SIZE as f32 * scale as f32 / 2.0;
    let center_y = HEIGHT as f32 * scale as f32 / 2.0;
    let dot_color = [ACCENT[0], ACCENT[1], ACCENT[2], 255];
    let highlight = [228, 244, 255, 220];
    for y in 0..(HEIGHT * scale) {
        for x in 0..(width * scale) {
            let px = x as f32 + 0.5 - center as f32;
            let py = y as f32 + 0.5 - center_y;
            if px * px + py * py > radius * radius {
                continue;
            }
            let color = if px * px + py * py < (radius * 0.40).powi(2) {
                highlight
            } else {
                dot_color
            };
            let offset = ((y * width * scale + x) * 4) as usize;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}
