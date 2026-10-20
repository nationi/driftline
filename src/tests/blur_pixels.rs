//! Pixel-level conformance for the frost, rendered by a real GL context.
//!
//! The chrome scenarios in [`super::render_pixels`] go through the screenshot
//! path, which deliberately draws no blur. These drive `compose_frame` and read
//! the live frame back instead.
//!
//! `#[ignore]`d out of the default lane (needs Mesa's surfaceless EGL,
//! self-skipping when that is missing); each scenario takes [`gl::lock`] first.

use image::RgbaImage;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Physical, Scale, Size};

use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use super::client::{ClientId, LayerConfigureProps};
use super::real::TempDir;
use super::{Fixture, gl};

/// The output every scenario runs on, in physical pixels, and its scale. At 2x,
/// anything sized from the unscaled strength is short by half.
const OUTPUT: (u32, u32) = (1600, 1200);
const SCALE: f64 = 2.0;

/// The frosted window, logical and physical.
const WINDOW_LOGICAL: (u16, u16) = (300, 200);
const WINDOW_PHYSICAL: (i32, i32) = (600, 400);

/// What the frosted window paints: alpha 1/255 is above the mask shader's
/// `step(0.001, a)` threshold, so the whole window is frosted, and close enough
/// to invisible that the composite over the frost reads as the frost itself.
const GHOST_BGRA: [u8; 4] = [0, 0, 0, 1];

/// Hard black/white stripes inside a rectangle centred on the output, flat
/// white outside it. The stripes blur to one mid-grey, so the frost holds one
/// number wherever the blur sees only them, and the white beyond shows as a
/// deviation from it wherever the reach gets to it.
///
/// `margin` is how far the stripes extend past the window on every side, in
/// physical pixels.
fn framed_stripes_shader(margin: f32) -> String {
    let half_x = (WINDOW_PHYSICAL.0 as f32 / 2.0 + margin) / OUTPUT.0 as f32;
    let half_y = (WINDOW_PHYSICAL.1 as f32 / 2.0 + margin) / OUTPUT.1 as f32;
    format!(
        "precision highp float;\n\
         varying vec2 v_coords;\n\
         void main() {{\n\
         \x20   vec2 d = abs(v_coords - vec2(0.5));\n\
         \x20   float inner = step(d.x, {half_x}) * step(d.y, {half_y});\n\
         \x20   float s = step(0.5, fract(v_coords.x * 100.0));\n\
         \x20   gl_FragColor = mix(vec4(1.0), vec4(s, s, s, 1.0), inner);\n\
         }}\n"
    )
}

fn config_toml(shader_path: &str, radius: u32, strength: f64) -> String {
    format!(
        "[background]\n\
         type = \"shader\"\n\
         path = \"{shader_path}\"\n\
         \n\
         [effects]\n\
         blur_radius = {radius}\n\
         blur_strength = {strength}\n\
         animate_blur_fps = 0\n\
         \n\
         [decorations]\n\
         default_mode = \"client\"\n\
         border_width = 0\n\
         corner_radius = 0\n\
         shadow = false\n\
         \n\
         [[window_rules]]\n\
         app_id = \"frosted\"\n\
         blur = true\n"
    )
}

/// [`gl::install`] plus the blur trio it leaves alone.
fn install_with_blur(f: &mut Fixture, mut renderer: GlesRenderer) {
    let (down, up, mask) = crate::render::compile_blur_shaders(&mut renderer);
    gl::install(f, renderer);
    f.state().render.blur_down_shader = down;
    f.state().render.blur_up_shader = up;
    f.state().render.blur_mask_shader = mask;
    f.state().render.blur_wrap_mode = None;
}

/// Undo [`install_with_blur`] and drop every per-output GPU cache a live frame
/// created, so the fixture's teardown baseline holds.
fn uninstall_with_blur(f: &mut Fixture, output: &Output) {
    f.state().render.remove_output(&output.name());
    f.state().render.blur_down_shader = None;
    f.state().render.blur_up_shader = None;
    f.state().render.blur_mask_shader = None;
    f.state().render.blur_wrap_mode = None;
    gl::uninstall(f);
}

/// Compose one live frame for `output` and read it back.
fn live_frame(f: &mut Fixture, output: &Output) -> RgbaImage {
    let mut backend = f
        .state()
        .backend
        .take()
        .expect("the fixture has a renderer");
    let bytes = {
        let renderer = backend.renderer();
        let elements = crate::render::compose_frame(f.state(), renderer, output, Vec::new());
        let refs: Vec<&crate::render::OutputRenderElements> = elements.iter().collect();
        crate::render::render_elements_to_rgba(
            renderer,
            Size::<i32, Physical>::from((OUTPUT.0 as i32, OUTPUT.1 as i32)),
            Scale::from(output.current_scale().fractional_scale()),
            &refs,
        )
    };
    f.state().backend = Some(backend);
    RgbaImage::from_raw(OUTPUT.0, OUTPUT.1, bytes.expect("render the live frame"))
        .expect("the readback is the frame's own size")
}

/// Where the frost is. The backdrop is pure black or pure white everywhere, so
/// anything in between has been through the blur.
fn frost_bounds(img: &RgbaImage) -> (u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] == 255 && (24..=231).contains(&p[0]) {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    assert!(x0 <= x1 && y0 <= y1, "no frosted pixels in the frame");
    (x0, y0, x1, y1)
}

fn luma(img: &RgbaImage, x: u32, y: u32) -> i32 {
    let p = img.get_pixel(x, y);
    (p[0] as i32 + p[1] as i32 + p[2] as i32) / 3
}

/// The pad the capture is grown by carries the output scale, as the Kawase
/// offset the shader is handed does. Sized from the unscaled strength, the
/// capture around a window on a 2x output is half as wide as the blur reaches.
///
/// The stripes stop 110 physical pixels past the window: outside a pad sized
/// from the strength alone (5 passes x 1.5 = 96), inside one that carries the
/// scale (192), and well inside the reach either way.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn the_frost_sees_backdrop_a_pad_without_the_output_scale_would_miss() {
    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("scaled blur pad") else {
        return;
    };
    let temp = TempDir::new();
    let shader = temp.path().join("framed_stripes.glsl");
    std::fs::write(&shader, framed_stripes_shader(110.0)).unwrap();

    let mut f = Fixture::with_config(super::config(&config_toml(
        shader.to_str().unwrap(),
        5,
        1.5,
    )));
    let output = f.add_output_scaled(1, (OUTPUT.0 as u16, OUTPUT.1 as u16), SCALE);
    install_with_blur(&mut f, renderer);

    let client = f.add_client();
    let pixels =
        super::render_pixels::solid_bgra(WINDOW_PHYSICAL.0, WINDOW_PHYSICAL.1, GHOST_BGRA, false);
    super::map_window_shm(
        &mut f,
        client,
        "frosted",
        WINDOW_LOGICAL,
        WINDOW_PHYSICAL,
        &pixels,
    );
    super::tick_until_settled(&mut f);

    let img = live_frame(&mut f, &output);
    let (x0, y0, x1, y1) = frost_bounds(&img);

    // The stripes are laid out around a centred window, so a placement change
    // has to fail here rather than quietly slide them out from under the frost.
    assert_eq!(
        (x1 - x0 + 1, y1 - y0 + 1),
        (WINDOW_PHYSICAL.0 as u32, WINDOW_PHYSICAL.1 as u32),
        "the frost covers the window"
    );
    assert_eq!(
        (x0 + x1 + 1, y0 + y1 + 1),
        (OUTPUT.0, OUTPUT.1),
        "the window is centred on the output"
    );

    let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
    let centre = luma(&img, mx, my);
    // Half black, half white: the stripes are all the centre's blur can reach
    // whatever the pad, which makes it the baseline the edges are read against.
    assert!(
        (126..=128).contains(&centre),
        "the frost's centre is the stripes' own average, got {centre}"
    );

    for (name, x, y) in [
        ("left", x0 + 4, my),
        ("right", x1 - 4, my),
        ("top", mx, y0 + 4),
        ("bottom", mx, y1 - 4),
    ] {
        let edge = luma(&img, x, y);
        assert!(
            edge - centre >= 5,
            "the {name} edge of the frost carries the white beyond the stripes: \
             {edge} against a centre of {centre}"
        );
    }

    uninstall_with_blur(&mut f, &output);
}

/// Map a layer surface painting a [`GHOST_BGRA`] buffer at `logical` size, and
/// return its `wl_surface`. `anchor` of `None` centres it on the output.
fn map_layer(
    f: &mut Fixture,
    id: ClientId,
    namespace: &str,
    layer: zwlr_layer_shell_v1::Layer,
    logical: (u32, u32),
    anchor: Option<zwlr_layer_surface_v1::Anchor>,
) -> wayland_client::protocol::wl_surface::WlSurface {
    map_layer_filled(f, id, namespace, layer, logical, anchor, GHOST_BGRA)
}

/// [`map_layer`] with the texel the first buffer is filled with.
fn map_layer_filled(
    f: &mut Fixture,
    id: ClientId,
    namespace: &str,
    layer: zwlr_layer_shell_v1::Layer,
    logical: (u32, u32),
    anchor: Option<zwlr_layer_surface_v1::Anchor>,
    bgra: [u8; 4],
) -> wayland_client::protocol::wl_surface::WlSurface {
    let texels = layer_texels(logical);
    let pixels = super::render_pixels::solid_bgra(texels.0, texels.1, bgra, false);

    let created = f.client(id).create_layer(None, layer, namespace);
    let surface = created.surface.clone();
    created.set_configure_props(LayerConfigureProps {
        size: Some(logical),
        anchor,
        exclusive_zone: Some(0),
        ..Default::default()
    });
    created.commit();
    f.roundtrip(id);

    let l = f.client(id).layer(&surface);
    l.set_size(logical.0 as u16, logical.1 as u16);
    l.attach_shm_buffer(texels, &pixels);
    l.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

/// A layer surface's buffer size in texels at the scenario scale.
fn layer_texels(logical: (u32, u32)) -> (i32, i32) {
    (
        (logical.0 as f64 * SCALE) as i32,
        (logical.1 as f64 * SCALE) as i32,
    )
}

/// The mid-grey a blurred stripe field settles at, and how far from it a
/// frosted pixel may sit. The tolerance is generous because a surface at an
/// output edge blurs over the mirror wrap, whose stripe phase does not match
/// the backdrop's; it only has to tell frost from bare black or white.
const STRIPE_GREY: i32 = 127;
const FROST_TOLERANCE: i32 = 12;

fn frosted(img: &RgbaImage, x: u32, y: u32) -> bool {
    (luma(img, x, y) - STRIPE_GREY).abs() <= FROST_TOLERANCE
}

/// Stripes everywhere: the layer scenarios only ask whether the frost is there
/// at all, and the blurred stripe field is a number the bare backdrop never is.
fn stripes_shader() -> String {
    "precision highp float;\n\
     varying vec2 v_coords;\n\
     void main() {\n\
     \x20   float s = step(0.5, fract(v_coords.x * 100.0));\n\
     \x20   gl_FragColor = vec4(s, s, s, 1.0);\n\
     }\n"
    .to_string()
}

/// A fixture over [`stripes_shader`] with the blur programs installed, or
/// `None` where there is no EGL to run them on. The `TempDir` is returned
/// because the background shader is read off disk on the first frame.
fn stripe_scene(name: &str) -> Option<(Fixture, Output, TempDir)> {
    let renderer = gl::surfaceless_renderer(name)?;
    let temp = TempDir::new();
    let shader = temp.path().join("stripes.glsl");
    std::fs::write(&shader, stripes_shader()).unwrap();

    let mut f = Fixture::with_config(super::config(&config_toml(
        shader.to_str().unwrap(),
        5,
        1.5,
    )));
    let output = f.add_output_scaled(1, (OUTPUT.0 as u16, OUTPUT.1 as u16), SCALE);
    install_with_blur(&mut f, renderer);
    Some((f, output, temp))
}

/// A layer surface's frost is built and spliced in the same pass that first
/// sees it, so the frame it maps on already carries it.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_layer_surface_is_frosted_on_the_frame_it_maps() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("layer first frame") else {
        return;
    };

    let client = f.add_client();
    map_layer(
        &mut f,
        client,
        "frosted",
        zwlr_layer_shell_v1::Layer::Top,
        (300, 200),
        None,
    );

    let img = live_frame(&mut f, &output);
    let (x0, y0, x1, y1) = frost_bounds(&img);
    assert_eq!(
        (x1 - x0 + 1, y1 - y0 + 1),
        (600, 400),
        "the frost covers the layer surface"
    );
    assert!(
        frosted(&img, (x0 + x1) / 2, (y0 + y1) / 2),
        "the layer surface's first frame is frosted"
    );

    uninstall_with_blur(&mut f, &output);
}

/// A fully transparent first buffer (GTK4 maps before its first paint)
/// captures an all-zero mask, and on a busy output the frame countdown that
/// retries it runs out before the client paints. The client's own commit has
/// to re-capture the mask, so the frame after it paints is frosted.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_layer_surface_that_paints_after_a_transparent_first_buffer_is_frosted() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("layer transparent first buffer") else {
        return;
    };

    let client = f.add_client();
    let logical = (300, 200);
    let surface = map_layer_filled(
        &mut f,
        client,
        "frosted",
        zwlr_layer_shell_v1::Layer::Top,
        logical,
        None,
        [0, 0, 0, 0],
    );
    let (cx, cy) = (OUTPUT.0 / 2, OUTPUT.1 / 2);

    // More composed frames than the frame countdown, all on the transparent
    // buffer: nothing to frost yet, and the countdown is gone.
    for _ in 0..4 {
        let img = live_frame(&mut f, &output);
        assert!(
            !frosted(&img, cx, cy),
            "a fully transparent surface shows the bare backdrop"
        );
    }

    let texels = layer_texels(logical);
    let pixels = super::render_pixels::solid_bgra(texels.0, texels.1, GHOST_BGRA, false);
    let l = f.client(client).layer(&surface);
    l.attach_shm_buffer(texels, &pixels);
    l.commit();
    f.double_roundtrip(client);

    let img = live_frame(&mut f, &output);
    assert!(
        frosted(&img, cx, cy),
        "the frame after the client paints is frosted"
    );

    uninstall_with_blur(&mut f, &output);
}

/// A launcher that opens and closes inside a couple of frames is frosted while
/// it is up, and takes the frost with it when it goes.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_layer_surface_that_unmaps_two_frames_later_was_frosted_the_whole_time() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("layer short life") else {
        return;
    };

    let client = f.add_client();
    let surface = map_layer(
        &mut f,
        client,
        "frosted",
        zwlr_layer_shell_v1::Layer::Top,
        (300, 200),
        None,
    );

    for frame in 1..=2 {
        let img = live_frame(&mut f, &output);
        let (x0, y0, x1, y1) = frost_bounds(&img);
        assert!(
            frosted(&img, (x0 + x1) / 2, (y0 + y1) / 2),
            "frame {frame} of the surface's life is frosted"
        );
    }

    let l = f.client(client).layer(&surface);
    l.attach_null();
    l.commit();
    f.double_roundtrip(client);

    let img = live_frame(&mut f, &output);
    assert!(
        !frosted(&img, OUTPUT.0 / 2 + 8, OUTPUT.1 / 2),
        "the frost goes with the surface it belonged to"
    );

    uninstall_with_blur(&mut f, &output);
}

/// A surface anchored to the output edge has a padded capture that hangs off
/// the output, so it goes through the clipped-scratch path with the mirror on
/// the output boundary rather than straight into the pad. It is frosted all the
/// same.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn an_edge_anchored_layer_surface_is_frosted_where_its_capture_clips() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("layer at the edge") else {
        return;
    };

    let client = f.add_client();
    map_layer(
        &mut f,
        client,
        "frosted",
        zwlr_layer_shell_v1::Layer::Top,
        (800, 40),
        Some(
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        ),
    );

    let img = live_frame(&mut f, &output);
    let (x0, y0, x1, y1) = frost_bounds(&img);
    assert_eq!(
        (x0, y0, x1, y1),
        (0, 0, OUTPUT.0 - 1, 79),
        "the bar spans the top edge of the output"
    );
    for x in [2u32, 8, 16, 32, 64, 128, 800, 1500, 1590, 1597] {
        assert!(frosted(&img, x, 40), "the bar is frosted at x={x}");
    }
    for y in [2u32, 8, 20, 40, 60, 77] {
        assert!(frosted(&img, 800, y), "the bar is frosted at y={y}");
    }

    uninstall_with_blur(&mut f, &output);
}

/// By design, and asserted rather than fixed: a fullscreen window conceals the
/// canvas, so an overlay layer above it frosts the fullscreen picture rather
/// than the canvas.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn an_overlay_layer_over_a_fullscreen_window_frosts_the_fullscreen_picture() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("layer over fullscreen") else {
        return;
    };

    // Mid-grey and opaque, so the frost over it is a number neither the stripes
    // nor an unrendered frost can produce.
    const FS_LUMA: i32 = 0x60;
    let client = f.add_client();
    let fs_logical = (
        (OUTPUT.0 as f64 / SCALE) as u16,
        (OUTPUT.1 as f64 / SCALE) as u16,
    );
    let fs_texels = (OUTPUT.0 as i32, OUTPUT.1 as i32);
    let opaque = super::render_pixels::solid_bgra(
        fs_texels.0,
        fs_texels.1,
        [FS_LUMA as u8, FS_LUMA as u8, FS_LUMA as u8, 0xff],
        false,
    );
    let window = super::map_window_shm(&mut f, client, "fs", fs_logical, fs_texels, &opaque);
    f.client(client).window(&window).set_fullscreen(None);
    f.double_roundtrip(client);
    // Not `adopt_last_configure`: that attaches a transparent single-pixel
    // buffer, and a fullscreen window that paints nothing conceals no canvas.
    let w = f.client(client).window(&window);
    w.set_size(fs_logical.0, fs_logical.1);
    w.attach_shm_buffer(fs_texels, &opaque);
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    super::tick_until_settled(&mut f);

    map_layer(
        &mut f,
        client,
        "frosted",
        zwlr_layer_shell_v1::Layer::Overlay,
        (300, 200),
        None,
    );

    let img = live_frame(&mut f, &output);
    let centre = luma(&img, OUTPUT.0 / 2, OUTPUT.1 / 2);
    assert_eq!(
        luma(&img, 100, 100),
        FS_LUMA,
        "the fullscreen window covers the output"
    );
    assert!(
        (centre - FS_LUMA).abs() <= 4,
        "the overlay's frost is the fullscreen picture, got {centre}"
    );

    uninstall_with_blur(&mut f, &output);
}

/// Two bars that between them cover most of the output claim the shared
/// backdrop: the background is blurred once and each slices its own rect out
/// of it. The other layer scenarios all take the per-window path.
#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn two_bars_that_pay_for_the_shared_backdrop_slice_their_frost_out_of_it() {
    let _gl = gl::lock();
    let Some((mut f, output, _temp)) = stripe_scene("shared backdrop slice") else {
        return;
    };

    let client = f.add_client();
    for anchor in [
        zwlr_layer_surface_v1::Anchor::Top,
        zwlr_layer_surface_v1::Anchor::Bottom,
    ] {
        map_layer(
            &mut f,
            client,
            "frosted",
            zwlr_layer_shell_v1::Layer::Top,
            (800, 220),
            Some(
                anchor | zwlr_layer_surface_v1::Anchor::Left | zwlr_layer_surface_v1::Anchor::Right,
            ),
        );
    }

    let img = live_frame(&mut f, &output);
    assert!(
        f.state()
            .render
            .shared_blur
            .get(&output.name())
            .is_some_and(|s| s.textures.is_some() && !s.stale),
        "the two bars paid for the shared backdrop"
    );
    for y in [80, OUTPUT.1 - 80] {
        for x in [64, OUTPUT.0 / 2, OUTPUT.0 - 64] {
            assert!(
                frosted(&img, x, y),
                "the bar at y={y} is frosted at x={x}, got {}",
                luma(&img, x, y)
            );
        }
    }

    uninstall_with_blur(&mut f, &output);
}

fn check_tiling_frost(framed: bool) {
    check_tiling_frost_at_scale(framed, SCALE);
}

fn check_tiling_frost_at_scale(framed: bool, scale: f64) {
    check_tiling_frost_scene(framed, scale, false);
}

#[derive(Clone, Copy)]
enum TilingFrostAnimation {
    None,
    Move,
    Open,
    PureFade,
    ActiveOpen,
    ActiveFade,
    RepeatResize,
    RepeatSsdResize,
}

fn check_tiling_frost_scene(framed: bool, scale: f64, translate: bool) {
    check_tiling_frost_capture(
        framed,
        scale,
        if translate {
            TilingFrostAnimation::Move
        } else {
            TilingFrostAnimation::None
        },
        false,
    );
}

fn check_tiling_frost_capture(
    framed: bool,
    scale: f64,
    animation: TilingFrostAnimation,
    moving_capture: bool,
) {
    use super::{last_configured, map_window_shm, window_by_app_id};
    use driftwm::config::Action;
    use smithay::reexports::wayland_server::Resource;
    use smithay::wayland::compositor::{
        Barrier, add_blocker, add_pre_commit_hook, remove_pre_commit_hook,
    };
    use smithay::wayland::seat::WaylandFocus;

    let Some(renderer) = gl::surfaceless_renderer("tiling frost") else {
        return;
    };
    let temp = TempDir::new();
    let shader = temp.path().join("frozen_stripes.glsl");
    std::fs::write(&shader, framed_stripes_shader(110.0)).unwrap();
    let config = format!(
        "{}\n[snap]\ngap = 0\n",
        config_toml(shader.to_str().unwrap(), 3, 1.5)
    );
    let config = if matches!(
        animation,
        TilingFrostAnimation::PureFade | TilingFrostAnimation::ActiveFade
    ) {
        config.replace("[effects]", "[effects]\nanimation_scale = 1.0")
    } else {
        config
    };
    let config = if framed {
        config
            .replace("border_width = 0", "border_width = 2")
            .replace("corner_radius = 0", "corner_radius = 12")
            .replace("shadow = false", "shadow = true")
    } else {
        config
    };
    let config = if matches!(animation, TilingFrostAnimation::RepeatSsdResize) {
        format!(
            "{}\n[[window_rules]]\napp_id = \"normal\"\ndecoration = \"server\"\n",
            config.replace("blur = true", "blur = true\ndecoration = \"server\"")
        )
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (OUTPUT.0 as u16, OUTPUT.1 as u16), scale);
    install_with_blur(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let pixels =
        super::render_pixels::solid_bgra(WINDOW_PHYSICAL.0, WINDOW_PHYSICAL.1, GHOST_BGRA, false);
    let a = map_window_shm(
        &mut f,
        first,
        "frosted",
        WINDOW_LOGICAL,
        WINDOW_PHYSICAL,
        &pixels,
    );
    let b_pixels = super::render_pixels::solid_bgra(200, 100, [0, 0, 255, 255], false);
    let b = map_window_shm(&mut f, second, "normal", (200, 100), (200, 100), &b_pixels);
    super::tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "frosted").unwrap();
    let wb = window_by_app_id(&mut f, "normal").unwrap();
    f.state().stage.set_position(&wa, (250, 200).into());
    f.state().stage.set_position(&wb, (650, 450).into());
    f.state().set_camera((0.0, 0.0).into());
    if moving_capture {
        f.state().execute_action(&Action::ToggleTiling);
        for (client, surface, size, data) in [
            (first, &a, WINDOW_PHYSICAL, &pixels),
            (second, &b, (200, 100), &b_pixels),
        ] {
            last_configured(&mut f, client, surface);
            let w = f.client(client).window(surface);
            w.attach_shm_buffer(size, data);
            w.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        assert_eq!(f.state().tiling_snapshot_count(), 0);
        f.state().cancel_window_animation(&wa);
        f.state().cancel_window_animation(&wb);
        f.state().stage.set_position(&wa, (250, 200).into());
        f.state().stage.set_position(&wb, (650, 450).into());
        f.state().set_camera((0.0, 0.0).into());
        f.state()
            .animate_window_move_from(&wa, (300, 250).into(), None);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
    }
    let translated_before = if matches!(animation, TilingFrostAnimation::Move) {
        f.state()
            .animate_window_move_from(&wa, (300, 250).into(), None);
        let expected = live_frame(&mut f, &output);
        f.state().cancel_window_animation(&wa);
        Some(expected)
    } else {
        None
    };
    let opaque_bounds = frost_bounds(&live_frame(&mut f, &output));
    let opening_frames = if matches!(
        animation,
        TilingFrostAnimation::Open
            | TilingFrostAnimation::PureFade
            | TilingFrostAnimation::ActiveOpen
            | TilingFrostAnimation::ActiveFade
    ) {
        f.state().start_window_open_animation(&wa);
        if matches!(
            animation,
            TilingFrostAnimation::ActiveOpen | TilingFrostAnimation::ActiveFade
        ) {
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
        }
        let mut frames = vec![live_frame(&mut f, &output)];
        for _ in 0..2 {
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            // Rebuild the reference mask even for an opacity-only live frame.
            f.state().render.blur_geometry_generation += 1;
            frames.push(live_frame(&mut f, &output));
        }
        f.state().cancel_window_animation(&wa);
        f.state().render.blur_geometry_generation += 1;
        Some(frames)
    } else {
        None
    };
    if matches!(
        animation,
        TilingFrostAnimation::ActiveOpen | TilingFrostAnimation::ActiveFade
    ) {
        f.state().start_window_open_animation(&wa);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        f.state().render.blur_geometry_generation += 1;
    }
    let before = live_frame(&mut f, &output);
    let before_bounds = frost_bounds(&before);
    if moving_capture {
        f.state()
            .execute_action(&Action::MoveIntoColumn(driftwm::config::Direction::Left));
    } else {
        f.state().execute_action(&Action::ToggleTiling);
    }
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    let size_a = last_configured(&mut f, first, &a);
    let size_b = last_configured(&mut f, second, &b);
    let surface = wb.wl_surface().unwrap().into_owned();
    let barrier = Barrier::new(false);
    let blocker = barrier.clone();
    let hook = add_pre_commit_hook::<crate::state::DriftWm, _>(&surface, move |_, _, s| {
        add_blocker(s, blocker.clone());
    });
    for (client, surface, size) in [(first, &a, size_a), (second, &b, size_b)] {
        let pixels = super::render_pixels::solid_bgra(size.0, size.1, [0, 128, 0, 128], false);
        let w = f.client(client).window(surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &pixels);
        w.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, size_a.into());
    let pending = live_frame(&mut f, &output);
    assert_eq!(frost_bounds(&pending), before_bounds);
    let max_delta = before
        .as_raw()
        .iter()
        .zip(pending.as_raw())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    let differences: Vec<_> = before
        .enumerate_pixels()
        .zip(pending.pixels())
        .filter(|((_, _, a), b)| a.0.iter().zip(b.0.iter()).any(|(a, b)| a.abs_diff(*b) > 2))
        .take(8)
        .map(|((x, y, a), b)| (x, y, a.0, b.0))
        .collect();
    assert!(
        max_delta <= if framed { 2 } else { 1 },
        "frozen frost differs by {max_delta} bytes: {differences:?}"
    );
    if let Some(expected) = translated_before {
        let id = f.state().stage.id_of(&wa).unwrap();
        let resting_mask = f
            .state()
            .tiling_snapshot_elements(id, &output, (0.0, 0.0).into(), 1.0, scale)
            .unwrap()
            .blur
            .unwrap()
            .screen_rect;
        f.state()
            .animate_window_move_from(&wa, (300, 250).into(), None);
        assert_eq!(f.state().tiling_snapshot_count(), 2);
        let moved = live_frame(&mut f, &output);
        assert_eq!(frost_bounds(&moved), frost_bounds(&expected));
        let delta = moved
            .as_raw()
            .iter()
            .zip(expected.as_raw())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(
            delta <= if framed { 2 } else { 1 },
            "translated frost differs by {delta}"
        );
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        let advanced = live_frame(&mut f, &output);
        assert_ne!(frost_bounds(&advanced), frost_bounds(&moved));
        let visual = f
            .state()
            .window_animations
            .geometry_visual_rect(id)
            .unwrap();
        let delta = visual.loc - f.state().stage.position_of(&wa).unwrap().to_f64();
        let physical_delta: smithay::utils::Point<i32, smithay::utils::Physical> =
            delta.to_physical_precise_round(scale);
        let animated_mask = f
            .state()
            .tiling_snapshot_elements(id, &output, (0.0, 0.0).into(), 1.0, scale)
            .unwrap()
            .blur
            .unwrap()
            .screen_rect;
        assert_eq!(animated_mask.loc, resting_mask.loc + physical_delta);
        assert_eq!(animated_mask.size, resting_mask.size);
        f.state().cancel_window_animation(&wa);
        let restored = live_frame(&mut f, &output);
        assert_eq!(frost_bounds(&restored), opaque_bounds);
        assert_eq!(f.state().tiling_snapshot_count(), 2);
    }
    if let Some(frames) = opening_frames {
        if !matches!(
            animation,
            TilingFrostAnimation::ActiveOpen | TilingFrostAnimation::ActiveFade
        ) {
            f.state().start_window_open_animation(&wa);
        }
        assert_eq!(f.state().tiling_snapshot_count(), 2);
        for (tick, expected) in frames.iter().enumerate() {
            if tick != 0 {
                let generation = f.state().render.blur_geometry_generation;
                f.state()
                    .tick_window_animations(std::time::Duration::from_millis(16));
                assert!(
                    f.state().render.blur_geometry_generation > generation,
                    "fade must invalidate the mask even at constant size"
                );
            }
            let actual = live_frame(&mut f, &output);
            let delta = actual
                .as_raw()
                .iter()
                .zip(expected.as_raw())
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                delta <= if framed { 2 } else { 1 },
                "{scale}x framed={framed} opening frost tick {tick} differs by {delta}"
            );
        }
        f.state().cancel_window_animation(&wa);
        let restored = live_frame(&mut f, &output);
        assert_eq!(frost_bounds(&restored), opaque_bounds);
        assert_eq!(f.state().tiling_snapshot_count(), 2);
    }
    barrier.signal();
    let owner = surface.client().unwrap();
    let client_state = owner.get_data::<crate::state::ClientState>().unwrap();
    let dh = f.state().display_handle.clone();
    client_state
        .compositor_state
        .blocker_cleared(f.state(), &dh);
    f.double_roundtrip(second);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    if matches!(
        animation,
        TilingFrostAnimation::RepeatResize | TilingFrostAnimation::RepeatSsdResize
    ) {
        let released = live_frame(&mut f, &output);
        let delta = before
            .as_raw()
            .iter()
            .zip(released.as_raw())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        let differences: Vec<_> = before
            .enumerate_pixels()
            .zip(released.pixels())
            .filter(|((_, _, a), b)| a.0.iter().zip(b.0.iter()).any(|(a, b)| a.abs_diff(*b) > 2))
            .take(8)
            .map(|((x, y, a), b)| (x, y, a.0, b.0))
            .collect();
        assert!(
            delta <= 2,
            "{scale}x SSD={} initial frost release differs by {delta}: {differences:?}",
            matches!(animation, TilingFrostAnimation::RepeatSsdResize)
        );
    }
    // The release frame intentionally retains the old content. The following
    // animation tick, rather than release itself, must advance the picture.
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(16));
    assert!(before != live_frame(&mut f, &output));
    if matches!(
        animation,
        TilingFrostAnimation::RepeatResize | TilingFrostAnimation::RepeatSsdResize
    ) {
        f.state()
            .execute_action(&Action::FocusColumn(driftwm::config::Direction::Left));
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        let mut current = live_frame(&mut f, &output);
        for tick_ms in [16, 48, 112] {
            f.state().execute_action(&Action::CycleColumnWidth);
            assert!(f.state().tiling_snapshot_count() > 0);
            let held = live_frame(&mut f, &output);
            let delta = current
                .as_raw()
                .iter()
                .zip(held.as_raw())
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(delta <= 2, "{scale}x frost recapture differs by {delta}");
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert_eq!(held, live_frame(&mut f, &output));
            let size = last_configured(&mut f, first, &a);
            let pixels = super::render_pixels::solid_bgra(size.0, size.1, [0, 128, 0, 128], false);
            let window = f.client(first).window(&a);
            window.set_size(size.0 as u16, size.1 as u16);
            window.attach_shm_buffer(size, &pixels);
            window.ack_last_and_commit();
            f.double_roundtrip(first);
            f.double_roundtrip(second);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            let resumed = live_frame(&mut f, &output);
            let delta = held
                .as_raw()
                .iter()
                .zip(resumed.as_raw())
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                delta <= 2,
                "{scale}x tick {tick_ms} frost release differs by {delta}"
            );
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(tick_ms));
            current = live_frame(&mut f, &output);
        }
    }
    remove_pre_commit_hook(&surface, &hook);
    f.state().execute_action(&Action::ToggleTiling);
    uninstall_with_blur(&mut f, &output);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn tiling_keeps_frost_under_frozen_content_until_buffers_apply() {
    let _gl = gl::lock();
    check_tiling_frost(false);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn frozen_tiling_frost_excludes_shadows_in_rounded_corner_cutouts() {
    let _gl = gl::lock();
    check_tiling_frost(true);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn frozen_tiling_frost_preserves_pixels_at_fractional_output_scales() {
    let _gl = gl::lock();
    for scale in [1.25, 1.5] {
        check_tiling_frost_at_scale(false, scale);
        check_tiling_frost_at_scale(true, scale);
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn position_animation_moves_frozen_tiling_frost_and_shadow_together() {
    let _gl = gl::lock();
    for scale in [SCALE, 1.25, 1.5] {
        for framed in [false, true] {
            check_tiling_frost_scene(framed, scale, true);
        }
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn tiling_captures_frost_during_an_active_translation() {
    let _gl = gl::lock();
    for scale in [SCALE, 1.25, 1.5] {
        for framed in [false, true] {
            check_tiling_frost_capture(framed, scale, TilingFrostAnimation::None, true);
        }
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn opening_frozen_tiling_content_preserves_frost_and_shadow() {
    let _gl = gl::lock();
    for scale in [SCALE, 1.25, 1.5] {
        for framed in [false, true] {
            check_tiling_frost_capture(framed, scale, TilingFrostAnimation::Open, false);
        }
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn fading_frozen_tiling_content_refreshes_an_unchanged_blur_mask_rect() {
    let _gl = gl::lock();
    check_tiling_frost_capture(true, SCALE, TilingFrostAnimation::PureFade, false);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn incremental_frost_frames_match_full_redraw_with_output_flip() {
    use super::{map_window_shm, window_by_app_id};
    use smithay::backend::allocator::Fourcc;
    use smithay::backend::renderer::damage::OutputDamageTracker;
    use smithay::backend::renderer::gles::GlesTexture;
    use smithay::backend::renderer::{Bind, ExportMem, Offscreen};
    use smithay::utils::{Rectangle, Transform};

    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("incremental frost") else {
        return;
    };
    let temp = TempDir::new();
    let shader = temp.path().join("incremental-stripes.glsl");
    std::fs::write(&shader, framed_stripes_shader(110.0)).unwrap();
    let config = config_toml(shader.to_str().unwrap(), 2, 1.1)
        .replace("[decorations]", "[decorations]\nblur = true");
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (OUTPUT.0 as u16, OUTPUT.1 as u16), SCALE);
    install_with_blur(&mut f, renderer);
    let client = f.add_client();
    let pixels =
        super::render_pixels::solid_bgra(WINDOW_PHYSICAL.0, WINDOW_PHYSICAL.1, GHOST_BGRA, false);
    for name in ["frosted", "middle", "front"] {
        map_window_shm(
            &mut f,
            client,
            name,
            WINDOW_LOGICAL,
            WINDOW_PHYSICAL,
            &pixels,
        );
    }
    super::tick_until_settled(&mut f);
    let windows: Vec<_> = ["frosted", "middle", "front"]
        .into_iter()
        .map(|name| window_by_app_id(&mut f, name).unwrap())
        .collect();
    for (i, window) in windows.iter().enumerate() {
        f.state()
            .stage
            .set_position(window, (100 + 60 * i as i32, 90 + 45 * i as i32).into());
    }
    f.state().set_camera((0.0, 0.0).into());
    let size: Size<i32, Physical> = (OUTPUT.0 as i32, OUTPUT.1 as i32).into();
    let mut backend = f.state().backend.take().unwrap();
    {
        let renderer = backend.renderer();
        let gpu = renderer
            .with_context(|gl| unsafe {
                std::ffi::CStr::from_ptr(
                    gl.GetString(smithay::backend::renderer::gles::ffi::RENDERER)
                        .cast(),
                )
                .to_string_lossy()
                .into_owned()
            })
            .unwrap();
        eprintln!("incremental frost renderer: {gpu}");
        for transform in [Transform::Normal, Transform::Flipped180] {
            let mut incremental = Offscreen::<GlesTexture>::create_buffer(
                renderer,
                Fourcc::Abgr8888,
                size.to_logical(1).to_buffer(1, Transform::Normal),
            )
            .unwrap();
            let mut reference = Offscreen::<GlesTexture>::create_buffer(
                renderer,
                Fourcc::Abgr8888,
                size.to_logical(1).to_buffer(1, Transform::Normal),
            )
            .unwrap();
            let mut tracker = OutputDamageTracker::new(size, SCALE, transform);
            for step in 0..6 {
                let window = &windows[step % windows.len()];
                f.state().stage.set_position(
                    window,
                    (100 + 25 * step as i32, 80 + 17 * step as i32).into(),
                );
                f.state().render.blur_geometry_generation += 1;
                let elements =
                    crate::render::compose_frame(f.state(), renderer, &output, Vec::new());
                let mut results = Vec::new();
                for full in [false, true] {
                    let texture = if full {
                        &mut reference
                    } else {
                        &mut incremental
                    };
                    let mut fresh = OutputDamageTracker::new(size, SCALE, transform);
                    let dt = if full { &mut fresh } else { &mut tracker };
                    let mapping = {
                        let mut target = renderer.bind(texture).unwrap();
                        dt.render_output(
                            renderer,
                            &mut target,
                            if full || step == 0 { 0 } else { 1 },
                            &elements,
                            [0.0, 0.0, 0.0, 1.0],
                        )
                        .unwrap()
                        .sync
                        .wait()
                        .unwrap();
                        renderer
                            .copy_framebuffer(
                                &target,
                                Rectangle::from_size(
                                    size.to_logical(1).to_buffer(1, Transform::Normal),
                                ),
                                Fourcc::Abgr8888,
                            )
                            .unwrap()
                    };
                    results.push(renderer.map_texture(&mapping).unwrap().to_vec());
                }
                let delta = results[0]
                    .iter()
                    .zip(&results[1])
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert_eq!(
                    delta, 0,
                    "{transform:?} step {step}: incremental rendering differs"
                );
            }
        }
    }
    f.state().backend = Some(backend);
    uninstall_with_blur(&mut f, &output);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_captures_frost_during_an_active_open_or_fade() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for animation in [
            TilingFrostAnimation::ActiveOpen,
            TilingFrostAnimation::ActiveFade,
        ] {
            check_tiling_frost_capture(true, scale, animation, false);
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn repeated_tiling_resize_preserves_frost_and_chrome() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftwm::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for animation in [
            TilingFrostAnimation::RepeatResize,
            TilingFrostAnimation::RepeatSsdResize,
        ] {
            check_tiling_frost_capture(true, scale, animation, false);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ClientBlurMotion {
    Opening,
    Move,
    Resize,
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn client_blur_regions_follow_motion_opening_and_grouped_capture() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftwm::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for motion in [
            ClientBlurMotion::Opening,
            ClientBlurMotion::Move,
            ClientBlurMotion::Resize,
        ] {
            for ssd in [false, true] {
                check_client_blur_motion(scale, motion, ssd);
            }
        }
    }
}

fn check_client_blur_motion(scale: f64, motion: ClientBlurMotion, ssd: bool) {
    use super::{last_configured, map_window_shm, window_by_app_id};
    use driftwm::config::Action;
    let Some(renderer) = gl::surfaceless_renderer("client blur motion") else {
        return;
    };
    let temp = TempDir::new();
    let shader = temp.path().join("client-regions.glsl");
    std::fs::write(&shader, "precision highp float; varying vec2 v_coords; void main() { float s=step(0.5,fract(v_coords.x*80.0)); gl_FragColor=vec4(vec3(s),1.0); }").unwrap();
    let config = config_toml(shader.to_str().unwrap(), 3, 1.5)
        .replace(
            "blur = true",
            if ssd {
                "blur = false\ndecoration = \"server\""
            } else {
                "blur = false"
            },
        )
        .replace("border_width = 0", "border_width = 2")
        .replace("corner_radius = 0", "corner_radius = 8")
        .replace("shadow = false", "shadow = true")
        .replace(
            "[decorations]",
            "[decorations]\nopacity = 0.6\nopacity_focused = 0.6",
        );
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (OUTPUT.0 as u16, OUTPUT.1 as u16), scale);
    install_with_blur(&mut f, renderer);
    let client = f.add_client();
    let pixels = super::render_pixels::solid_bgra(
        WINDOW_PHYSICAL.0,
        WINDOW_PHYSICAL.1,
        [0, 64, 0, 128],
        false,
    );
    let surface = map_window_shm(
        &mut f,
        client,
        "frosted",
        WINDOW_LOGICAL,
        WINDOW_PHYSICAL,
        &pixels,
    );
    super::tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "frosted").unwrap();
    f.state().stage.set_position(&window, (250, 200).into());
    f.state().set_camera((0.0, 0.0).into());
    let effect = f.client(client).background_effect(&surface);
    let regions = [(15, 10, 90, 70), (160, 80, 70, 100), (-50, -30, 60, 40)];
    match motion {
        ClientBlurMotion::Opening => f.state().start_window_open_animation(&window),
        ClientBlurMotion::Move => {
            f.state()
                .animate_window_move_from(&window, (300, 250).into(), None)
        }
        ClientBlurMotion::Resize => f.state().begin_geometry_animation_seeded(
            &window,
            smithay::utils::Rectangle::new((280.0, 220.0).into(), (200.0, 120.0).into()),
            crate::state::window_animation::AnimSpace::Canvas,
            None,
            crate::state::window_animation::GeometryRole::Normal,
            crate::state::window_animation::ContentPolicy::Stretch,
            Some((250, 200).into()),
        ),
    }
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(16));
    let pending = live_frame(&mut f, &output);
    f.client(client).set_blur_region(&effect, Some(&regions));
    f.double_roundtrip(client);
    assert_eq!(
        pending,
        live_frame(&mut f, &output),
        "the first pending blur request must not change sampling"
    );
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    let enabled = live_frame(&mut f, &output);
    f.client(client).set_blur_region(&effect, None);
    f.double_roundtrip(client);
    assert_eq!(
        enabled,
        live_frame(&mut f, &output),
        "uncommitted blur state must not apply"
    );
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    let disabled = live_frame(&mut f, &output);
    assert!(
        enabled != disabled,
        "{scale}x {motion:?} SSD={ssd}: the partial regions must actually blur the backdrop"
    );
    let id = f.state().stage.id_of(&window).unwrap();
    let amplitude = f.state().config.effects.animation_scale;
    let visual = f.state().window_animations.animated_visual(
        id,
        (250.0, 200.0).into(),
        (300.0, 200.0).into(),
        amplitude,
    );
    for (x, y, pixel) in enabled.enumerate_pixels() {
        if pixel == disabled.get_pixel(x, y) {
            continue;
        }
        let in_region = regions.iter().any(|&(rx, ry, rw, rh)| {
            let left = rx.max(0) as f64 / 300.0;
            let right = (rx + rw).min(300) as f64 / 300.0;
            let top = ry.max(0) as f64 / 200.0;
            let bottom = (ry + rh).min(200) as f64 / 200.0;
            f64::from(x) >= (visual.loc.x + left * visual.size.w) * scale - 2.0
                && f64::from(x) <= (visual.loc.x + right * visual.size.w) * scale + 2.0
                && f64::from(y) >= (visual.loc.y + top * visual.size.h) * scale - 2.0
                && f64::from(y) <= (visual.loc.y + bottom * visual.size.h) * scale + 2.0
        });
        assert!(
            in_region,
            "{scale}x {motion:?} SSD={ssd}: frost escaped the animated client regions at ({x}, {y})"
        );
    }
    for rects in [&[][..], &[(500, 500, 20, 20)][..]] {
        f.client(client).set_blur_region(&effect, Some(rects));
        f.client(client).window(&surface).commit();
        f.double_roundtrip(client);
        assert_eq!(
            disabled,
            live_frame(&mut f, &output),
            "empty/clipped regions must not become whole-window blur"
        );
    }
    f.client(client).set_blur_region(&effect, Some(&regions));
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    let before = live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().tiling_snapshot_count(), 1);
    let captured = live_frame(&mut f, &output);
    let delta = before
        .as_raw()
        .iter()
        .zip(captured.as_raw())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x motion {motion:?} SSD={ssd}: region capture differs by {delta}"
    );
    last_configured(&mut f, client, &surface);
    let w = f.client(client).window(&surface);
    w.attach_shm_buffer(WINDOW_PHYSICAL, &pixels);
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    let released = live_frame(&mut f, &output);
    let delta = captured
        .as_raw()
        .iter()
        .zip(released.as_raw())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x motion {motion:?} SSD={ssd}: region release differs by {delta}"
    );
    f.state().execute_action(&Action::CycleColumnWidth);
    assert_eq!(f.state().tiling_snapshot_count(), 1);
    let held = live_frame(&mut f, &output);
    let doubled: Vec<_> = regions
        .iter()
        .map(|&(x, y, w, h)| (x * 2, y * 2, w * 2, h * 2))
        .collect();
    f.client(client).set_blur_region(&effect, Some(&doubled));
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    assert_eq!(
        held,
        live_frame(&mut f, &output),
        "an applied region-only commit must not mutate the held mask"
    );
    last_configured(&mut f, client, &surface);
    let resized = super::render_pixels::solid_bgra(
        WINDOW_PHYSICAL.0 * 2,
        WINDOW_PHYSICAL.1 * 2,
        [0, 64, 0, 128],
        false,
    );
    let w = f.client(client).window(&surface);
    w.set_size(WINDOW_LOGICAL.0 * 2, WINDOW_LOGICAL.1 * 2);
    w.attach_shm_buffer((WINDOW_PHYSICAL.0 * 2, WINDOW_PHYSICAL.1 * 2), &resized);
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    let resized = live_frame(&mut f, &output);
    let delta = held
        .as_raw()
        .iter()
        .zip(resized.as_raw())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x motion {motion:?} SSD={ssd}: resized region release differs by {delta}"
    );
    f.state().execute_action(&Action::ToggleTiling);
    uninstall_with_blur(&mut f, &output);
}
