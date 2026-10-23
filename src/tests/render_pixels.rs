//! Pixel-level conformance for window chrome, rendered by a real GL context.
//!
//! The fixture gets a surfaceless GLES renderer (see [`super::gl`]), a client
//! maps a window with a real `wl_shm` texture, and the compositor's own capture
//! path writes a PNG the test reads back. Everything the shaders decide —
//! where the ring lands, how wide it is, whether its corners are square, what
//! the shadow masks — is then a fact about bytes rather than about geometry
//! arithmetic.
//!
//! `#[ignore]`d out of the default lane (needs Mesa's surfaceless EGL,
//! self-skipping when that is missing); each scenario takes [`gl::lock`] first.

use std::path::Path;

use image::{Rgba, RgbaImage};
use smithay::utils::{Physical, Scale, Size};

use super::real::{TempDir, ipc_request};
use super::{Fixture, gl, map_window_shm};
use crate::ipc::protocol::{Request, Response, ScreenshotTarget, WindowSelector};

/// The ring colour every scenario configures, focused and unfocused alike — the
/// focused default is independent, and the mapped window is focused.
const RING_RGB: [u8; 3] = [0x57, 0x52, 0x79];
/// Opaque red, as the client paints it: premultiplied little-endian BGRA.
const CONTENT_BGRA: [u8; 4] = [0, 0, 255, 255];
const CONTENT_RGB: [u8; 3] = [0xFF, 0x00, 0x00];

fn config_toml(shadow: bool) -> String {
    format!(
        "[background]\n\
         type = \"none\"\n\
         \n\
         [decorations]\n\
         default_mode = \"client\"\n\
         border_width = 2\n\
         border_color = \"#575279\"\n\
         border_color_focused = \"#575279\"\n\
         corner_radius = 0\n\
         shadow = {shadow}\n"
    )
}

/// The odd-sized window both the capture and the live-frame scenario use:
/// 1693x1053 logical lands on a half physical pixel at 1.5x, so the buffer the
/// fractional-scale protocol has the client allocate is a texel wider and
/// taller than the window paints. `ODD_*` are what its chrome measures there.
const ODD_LOGICAL: (u16, u16) = (1693, 1053);
const ODD_TEXELS: (i32, i32) = (2540, 1580);
const ODD_RING_PX: usize = 3;
const ODD_CONTENT_W: usize = 2539;
const ODD_CONTENT_H: usize = 1579;

/// Map [`ODD_LOGICAL`] with an [`ODD_TEXELS`] texture, its last column and row
/// left blank per [`solid_bgra`]'s `blank_last_col_row`.
fn map_odd_window(f: &mut Fixture, client: super::client::ClientId) {
    let pixels = solid_bgra(ODD_TEXELS.0, ODD_TEXELS.1, CONTENT_BGRA, true);
    map_window_shm(f, client, "pixels", ODD_LOGICAL, ODD_TEXELS, &pixels);
}

/// A premultiplied little-endian BGRA buffer of `w`x`h` texels filled with
/// `color`. `blank_last_col_row` leaves the far column and row transparent —
/// what a client paints into the extra texel `wp_fractional_scale_v1` rounds
/// its buffer up to when the logical size lands on a half physical pixel.
pub(super) fn solid_bgra(w: i32, h: i32, color: [u8; 4], blank_last_col_row: bool) -> Vec<u8> {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            if blank_last_col_row && (x == w - 1 || y == h - 1) {
                continue;
            }
            let i = ((y * w + x) * 4) as usize;
            buf[i..i + 4].copy_from_slice(&color);
        }
    }
    buf
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Px {
    Clear,
    /// Black at any alpha: the drop shadow, which no other element paints.
    Shadow,
    Ring,
    Content,
    Other,
}

/// Whether `p` is `rgb` premultiplied by its own alpha — true of a fully
/// covered pixel and of a partially covered one alike, so the classification
/// survives anti-aliasing and the alpha is left to be asserted separately.
fn premultiplied(p: Rgba<u8>, rgb: [u8; 3]) -> bool {
    let a = f32::from(p[3]);
    (0..3).all(|i| (f32::from(p[i]) - a * f32::from(rgb[i]) / 255.0).abs() <= 2.0)
}

fn classify(p: Rgba<u8>) -> Px {
    if p[3] == 0 {
        Px::Clear
    } else if p[0] == 0 && p[1] == 0 && p[2] == 0 {
        Px::Shadow
    } else if premultiplied(p, RING_RGB) {
        Px::Ring
    } else if premultiplied(p, CONTENT_RGB) {
        Px::Content
    } else {
        Px::Other
    }
}

/// Inclusive `(start, end)` index pairs of every maximal run of `want`.
fn runs_of(line: &[Px], want: Px) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, px) in line.iter().enumerate() {
        match (*px == want, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                runs.push((s, i - 1));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        runs.push((s, line.len() - 1));
    }
    runs
}

fn row(img: &RgbaImage, y: u32) -> Vec<Rgba<u8>> {
    (0..img.width()).map(|x| *img.get_pixel(x, y)).collect()
}

fn column(img: &RgbaImage, x: u32) -> Vec<Rgba<u8>> {
    (0..img.height()).map(|y| *img.get_pixel(x, y)).collect()
}

/// What a line through the middle of a bordered window owes: a ring run of
/// `ring_px` on each side, `content_px` of client content between them with
/// nothing in between on either side, and no half-covered ring pixel — the
/// stroke's edges are integral, so its anti-aliasing band never widens past
/// the pixel it belongs to.
fn assert_ring_hugs_content(line: &[Rgba<u8>], ring_px: usize, content_px: usize, axis: &str) {
    let classes: Vec<Px> = line.iter().map(|p| classify(*p)).collect();

    if let Some(i) = classes
        .iter()
        .position(|c| !matches!(c, Px::Clear | Px::Ring | Px::Content))
    {
        panic!(
            "{axis}: pixel {i} is neither clear, ring, nor content: {:?}",
            line[i]
        );
    }

    let rings = runs_of(&classes, Px::Ring);
    assert_eq!(rings.len(), 2, "{axis}: ring runs are {rings:?}");
    for (start, end) in &rings {
        assert_eq!(end - start + 1, ring_px, "{axis}: ring run {start}..={end}");
        for (i, px) in line.iter().enumerate().take(*end + 1).skip(*start) {
            assert_eq!(
                px[3], 255,
                "{axis}: ring pixel {i} is only partly covered: {px:?}"
            );
        }
    }

    let content = runs_of(&classes, Px::Content);
    assert_eq!(content.len(), 1, "{axis}: content runs are {content:?}");
    let (start, end) = content[0];
    assert_eq!(end - start + 1, content_px, "{axis}: content run");
    assert_eq!(
        start,
        rings[0].1 + 1,
        "{axis}: background shows between the ring and the content"
    );
    assert_eq!(
        rings[1].0,
        end + 1,
        "{axis}: background shows between the content and the ring"
    );
}

/// The id of the fixture's only window, over the same IPC the capture uses.
fn only_window_id(f: &mut Fixture, ipc: &Path) -> u64 {
    let reply = ipc_request(f, ipc, &Request::State);
    let Ok(Response::State(info)) = reply else {
        panic!("expected a State reply, got {reply:?}");
    };
    assert_eq!(info.windows.len(), 1, "one window is mapped");
    info.windows[0].id
}

/// Capture window `id` in isolation at `scale` and read the PNG back.
fn capture_window(
    f: &mut Fixture,
    ipc: &Path,
    dir: &Path,
    name: &str,
    id: u64,
    scale: f64,
) -> RgbaImage {
    let path = dir.join(name);
    let reply = ipc_request(
        f,
        ipc,
        &Request::Screenshot {
            target: ScreenshotTarget::Window {
                window: Some(WindowSelector::Id(id)),
            },
            scale,
            path: path.to_string_lossy().into_owned(),
        },
    );
    let Ok(Response::Screenshot { width, height, .. }) = reply else {
        panic!("expected a Screenshot reply, got {reply:?}");
    };
    let img = image::open(&path)
        .expect("open the written capture")
        .to_rgba8();
    assert_eq!(
        (img.width(), img.height()),
        (width, height),
        "the reply's dimensions describe the PNG"
    );
    img
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn odd_window_at_fractional_capture_scale_has_no_gap_before_the_ring() {
    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("odd window") else {
        return;
    };

    let temp = TempDir::new();
    let mut f = Fixture::with_config(super::config(&config_toml(false)));
    f.add_output(1, (2560, 1600));
    gl::install(&mut f, renderer);
    let ipc = f.start_ipc(temp.path());

    let client = f.add_client();
    map_odd_window(&mut f, client);
    let id = only_window_id(&mut f, &ipc);

    let img = capture_window(&mut f, &ipc, temp.path(), "odd-1.5.png", id, 1.5);
    // 1697x1057 logical (the window plus its 2px ring) at 1.5, rounded up.
    assert_eq!((img.width(), img.height()), (2546, 1586));

    assert_ring_hugs_content(
        &row(&img, img.height() / 2),
        ODD_RING_PX,
        ODD_CONTENT_W,
        "middle row at 1.5x",
    );
    assert_ring_hugs_content(
        &column(&img, img.width() / 2),
        ODD_RING_PX,
        ODD_CONTENT_H,
        "middle column at 1.5x",
    );

    // The frame's own half pixel: the last column and row are all the capture
    // has left over, and nothing else in it is transparent.
    for y in 0..img.height() {
        for x in 0..img.width() {
            let edge = x == img.width() - 1 || y == img.height() - 1;
            let clear = img.get_pixel(x, y)[3] == 0;
            assert_eq!(clear, edge, "pixel ({x}, {y}) is {:?}", img.get_pixel(x, y));
        }
    }

    // At an integral scale nothing is rounded up in the first place, so the
    // ring lands on the same edges the plain conversion gives and the capture
    // has no spare half pixel to leave clear.
    let img = capture_window(&mut f, &ipc, temp.path(), "odd-1.0.png", id, 1.0);
    assert_eq!((img.width(), img.height()), (1697, 1057));
    assert_ring_hugs_content(&row(&img, img.height() / 2), 2, 1693, "middle row at 1x");
    assert_ring_hugs_content(
        &column(&img, img.width() / 2),
        2,
        1053,
        "middle column at 1x",
    );

    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn square_ring_at_radius_zero_and_the_shadow_stays_outside() {
    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("square ring") else {
        return;
    };

    let temp = TempDir::new();
    let mut f = Fixture::with_config(super::config(&config_toml(true)));
    f.add_output(1, (2560, 1600));
    gl::install(&mut f, renderer);
    let ipc = f.start_ipc(temp.path());

    // Even on both axes, so the chrome is integral at either capture scale and
    // only the corner shape is under test.
    let pixels = solid_bgra(504, 304, CONTENT_BGRA, false);
    let client = f.add_client();
    let _surface = map_window_shm(&mut f, client, "pixels", (504, 304), (504, 304), &pixels);
    let id = only_window_id(&mut f, &ipc);

    for (scale, file) in [(1.0, "square-1.0.png"), (1.5, "square-1.5.png")] {
        let img = capture_window(&mut f, &ipc, temp.path(), file, id, scale);
        let mid_row: Vec<Px> = row(&img, img.height() / 2)
            .iter()
            .map(|p| classify(*p))
            .collect();
        let mid_col: Vec<Px> = column(&img, img.width() / 2)
            .iter()
            .map(|p| classify(*p))
            .collect();
        let horizontal = runs_of(&mid_row, Px::Ring);
        let vertical = runs_of(&mid_col, Px::Ring);
        assert_eq!(horizontal.len(), 2, "{scale}x: ring runs across the row");
        assert_eq!(vertical.len(), 2, "{scale}x: ring runs down the column");
        let (left, right) = (horizontal[0].0 as u32, horizontal[1].1 as u32);
        let (top, bottom) = (vertical[0].0 as u32, vertical[1].1 as u32);

        // A quarter circle would leave the outermost corner pixel outside the
        // stroke, partly covered at best.
        for (x, y) in [(left, top), (right, bottom)] {
            let p = *img.get_pixel(x, y);
            assert_eq!(
                classify(p),
                Px::Ring,
                "{scale}x: corner ({x}, {y}) is {p:?}"
            );
            assert_eq!(p[3], 255, "{scale}x: corner ({x}, {y}) is {p:?}");
        }

        // The shadow grades from a square perimeter: diagonally past the corner
        // it is never stronger than straight out from the edge beside it.
        let diagonal = img.get_pixel(left - 1, top - 1)[3];
        let beside = img.get_pixel(left - 1, top + 1)[3];
        assert!(
            diagonal <= beside,
            "{scale}x: shadow bulges at the corner ({diagonal} > {beside})"
        );
        assert!(beside > 0, "{scale}x: no shadow outside the ring at all");
    }

    // Half-opaque red over nothing else: a mask that also shaded the window's
    // own interior would show up as extra alpha here.
    let reply = ipc_request(
        &mut f,
        &ipc,
        &Request::Opacity {
            window: Some(WindowSelector::Id(id)),
            value: Some(0.5),
        },
    );
    assert_eq!(reply, Ok(Response::Opacity(0.5)), "the opacity round-trips");

    let img = capture_window(&mut f, &ipc, temp.path(), "square-half.png", id, 1.0);
    let centre = *img.get_pixel(img.width() / 2, img.height() / 2);
    assert!(
        centre[0].abs_diff(128) <= 2
            && centre[1] == 0
            && centre[2] == 0
            && centre[3].abs_diff(128) <= 2,
        "the window centre at opacity 0.5 is {centre:?}"
    );

    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn live_frame_agrees_with_the_capture_at_fractional_output_scale() {
    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("live frame") else {
        return;
    };

    let mut f = Fixture::with_config(super::config(&config_toml(false)));
    let output = f.add_output_scaled(1, (2560, 1600), 1.5);
    gl::install(&mut f, renderer);

    let client = f.add_client();
    map_odd_window(&mut f, client);
    // A live frame draws the open animation, which the capture path skips by
    // design — so this is the only test that has to run it out first.
    super::tick_until_settled(&mut f);

    // The frame the udev backend would submit, rendered offscreen: no damage
    // tracker, no cursor, no backend surface.
    let mut backend = f
        .state()
        .backend
        .take()
        .expect("the fixture has a renderer");
    let bytes = {
        let renderer = backend.renderer();
        let elements = crate::render::compose_frame(f.state(), renderer, &output, Vec::new());
        let refs: Vec<&crate::render::OutputRenderElements> = elements.iter().collect();
        crate::render::render_elements_to_rgba(
            renderer,
            Size::<i32, Physical>::from((2560, 1600)),
            Scale::from(1.5),
            &refs,
        )
    };
    f.state().backend = Some(backend);
    let img = RgbaImage::from_raw(2560, 1600, bytes.expect("render the live frame"))
        .expect("the frame fills the buffer");

    // The camera puts the window wherever it puts it; find it by its content.
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..img.height() {
        for x in 0..img.width() {
            if classify(*img.get_pixel(x, y)) == Px::Content {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    assert!(x0 < x1 && y0 < y1, "the live frame drew no window content");

    assert_ring_hugs_content(
        &row(&img, (y0 + y1) / 2),
        ODD_RING_PX,
        ODD_CONTENT_W,
        "live middle row",
    );
    assert_ring_hugs_content(
        &column(&img, (x0 + x1) / 2),
        ODD_RING_PX,
        ODD_CONTENT_H,
        "live middle column",
    );

    // Only a rendered frame creates per-output GPU state, so the fixture's
    // teardown baseline knows nothing about it; drop it the way an output
    // disconnect does.
    let output_name = output.name();
    f.state().render.remove_output(&output_name);
    gl::uninstall(&mut f);
}

fn tiling_live_frame(f: &mut Fixture, output: &smithay::output::Output) -> Vec<u8> {
    let mut backend = f.state().backend.take().expect("fixture renderer");
    let renderer = backend.renderer();
    let elements = crate::render::compose_frame(f.state(), renderer, output, Vec::new());
    let refs: Vec<_> = elements.iter().collect();
    let mode = output.current_mode().expect("output mode").size;
    let scale = Scale::from(output.current_scale().fractional_scale());
    let transform = output.current_transform();
    let raw =
        crate::render::render_elements_to_rgba_transformed(renderer, mode, scale, transform, &refs)
            .expect("render transformed tiling frame");
    let bytes = if transform == smithay::utils::Transform::Normal {
        raw
    } else {
        let size = transform.transform_size(mode);
        upright_scanout(
            &raw,
            mode.w as usize,
            size.w as usize,
            size.h as usize,
            transform,
        )
    };
    f.state().backend = Some(backend);
    bytes
}

fn upright_scanout(
    raw: &[u8],
    stride: usize,
    width: usize,
    height: usize,
    transform: smithay::utils::Transform,
) -> Vec<u8> {
    use smithay::utils::Transform;
    let mut upright = vec![0; raw.len()];
    for y in 0..height {
        for x in 0..width {
            let (rx, ry) = match transform {
                Transform::Normal => (x, y),
                Transform::_90 => (y, width - 1 - x),
                Transform::_180 => (width - 1 - x, height - 1 - y),
                Transform::_270 => (height - 1 - y, x),
                Transform::Flipped => (width - 1 - x, y),
                Transform::Flipped90 => (y, x),
                Transform::Flipped180 => (x, height - 1 - y),
                Transform::Flipped270 => (height - 1 - y, width - 1 - x),
            };
            let source = (ry * stride + rx) * 4;
            let destination = (y * width + x) * 4;
            upright[destination..destination + 4].copy_from_slice(&raw[source..source + 4]);
        }
    }
    upright
}

#[derive(Clone, Copy)]
enum TilingFinish {
    Buffers,
    ReleaseAnimation,
    ReleaseSsdFrame,
    ReleaseCsdFrame,
    ReleaseTranslucent,
    ReleasePixelAlpha,
    ReleaseRetarget,
    Deadline,
    Exit,
    OpenAnimation,
    ActiveOpen,
    UnresolvedHold,
    LateReply,
    OpenDeadline,
    OpenExit,
    MoveAnimation,
    ResizeAnimation,
    Supersede,
}

#[derive(Clone, Copy)]
enum TilingPopupScene {
    None,
    Menu,
    Nested,
    Dismiss,
    Unmap,
    Appears,
    Update,
}

fn colored_tiling_popup(
    f: &mut Fixture,
    client: super::client::ClientId,
    parent: &wayland_client::protocol::wl_surface::WlSurface,
    offset: (i32, i32),
    color: [u8; 4],
) -> wayland_client::protocol::wl_surface::WlSurface {
    let props = super::client::PopupProps {
        size: (80, 60),
        offset,
        ..Default::default()
    };
    let surface = f
        .client(client)
        .create_popup_with(parent, props)
        .surface
        .clone();
    f.client(client).popup(&surface).commit();
    f.roundtrip(client);
    let popup = f.client(client).popup(&surface);
    popup.set_size(100, 80);
    popup.xdg_surface.set_window_geometry(7, 9, 80, 60);
    popup.attach_shm_buffer((100, 80), &solid_bgra(100, 80, color, false));
    popup.ack_last_and_commit();
    f.double_roundtrip(client);
    surface
}

fn check_pending_tiling_pixels(finish: TilingFinish, shadow: bool, scene: TilingPopupScene) {
    check_pending_tiling_pixels_at_scale(finish, shadow, scene, 1.0);
}

fn check_pending_tiling_pixels_at_scale(
    finish: TilingFinish,
    shadow: bool,
    scene: TilingPopupScene,
    scale: f64,
) {
    check_pending_tiling_capture(finish, shadow, scene, scale, None);
}

fn check_pending_tiling_capture(
    finish: TilingFinish,
    shadow: bool,
    scene: TilingPopupScene,
    scale: f64,
    moving_before_capture: Option<i32>,
) {
    use super::{last_configured, window_by_app_id};
    use driftline::config::Action;
    use smithay::reexports::wayland_server::Resource;
    use smithay::wayland::compositor::{
        Barrier, add_blocker, add_pre_commit_hook, remove_pre_commit_hook,
    };
    use smithay::wayland::seat::WaylandFocus;

    let Some(renderer) = gl::surfaceless_renderer("tiling freeze") else {
        return;
    };
    let config = format!("{}\n[snap]\ngap = 0\n", config_toml(shadow));
    let config = if matches!(
        finish,
        TilingFinish::ReleaseTranslucent
            | TilingFinish::ReleasePixelAlpha
            | TilingFinish::ReleaseRetarget
    ) {
        config.replace(
            "[decorations]",
            "[decorations]\nopacity = 0.6\nopacity_focused = 0.6",
        )
    } else {
        config
    };
    let config = if matches!(
        finish,
        TilingFinish::ReleaseAnimation
            | TilingFinish::ReleaseTranslucent
            | TilingFinish::ReleasePixelAlpha
            | TilingFinish::ReleaseRetarget
    ) {
        config.replace("default_mode = \"client\"", "default_mode = \"none\"")
    } else {
        config
    };
    let config = if matches!(finish, TilingFinish::ReleaseSsdFrame) {
        format!(
            "{}\n[[window_rules]]\napp_id = \"a\"\ndecoration = \"server\"\n\n[[window_rules]]\napp_id = \"b\"\ndecoration = \"server\"\n",
            config.replace("corner_radius = 0", "corner_radius = 8")
        )
    } else {
        config
    };
    let config = if matches!(finish, TilingFinish::ReleaseCsdFrame) {
        config.replace("corner_radius = 0", "corner_radius = 8")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let old_color = if matches!(
        finish,
        TilingFinish::ReleasePixelAlpha | TilingFinish::ReleaseRetarget
    ) {
        [0, 0, 96, 96]
    } else {
        CONTENT_BGRA
    };
    let old = solid_bgra(200, 100, old_color, false);
    let a = map_window_shm(&mut f, first, "a", (200, 100), (200, 100), &old);
    let b = map_window_shm(&mut f, second, "b", (200, 100), (200, 100), &old);
    if matches!(
        finish,
        TilingFinish::ReleaseAnimation | TilingFinish::ReleaseTranslucent
    ) {
        for (client, surface) in [(first, &a), (second, &b)] {
            f.client(client)
                .set_opaque_region(surface, &[(0, 0, 10000, 10000)]);
            f.client(client)
                .window(surface)
                .attach_shm_buffer((200, 100), &old);
            f.client(client).window(surface).commit();
            f.double_roundtrip(client);
        }
    }
    super::tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().stage.set_position(&wa, (50, 50).into());
    f.state().stage.set_position(&wb, (350, 50).into());
    f.state().set_camera((0.0, 0.0).into());
    let popup = if matches!(scene, TilingPopupScene::None | TilingPopupScene::Appears) {
        None
    } else {
        Some(colored_tiling_popup(
            &mut f,
            first,
            &a,
            (180, 50),
            [255, 0, 0, 255],
        ))
    };
    if matches!(scene, TilingPopupScene::Nested) {
        colored_tiling_popup(
            &mut f,
            first,
            popup.as_ref().unwrap(),
            (35, 20),
            [255, 0, 255, 255],
        );
    }
    if let Some(destination) = moving_before_capture {
        f.state().execute_action(&Action::ToggleTiling);
        for (client, surface) in [(first, &a), (second, &b)] {
            last_configured(&mut f, client, surface);
            let w = f.client(client).window(surface);
            w.attach_shm_buffer((200, 100), &old);
            w.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        assert_eq!(f.state().tiling_snapshot_count(), 0);
        f.state().cancel_window_animation(&wa);
        f.state().cancel_window_animation(&wb);
        f.state().stage.set_position(&wa, (destination, 50).into());
        f.state().stage.set_position(&wb, (350, 50).into());
        f.state().set_camera((0.0, 0.0).into());
        let seed_x = if destination < 0 { 600 } else { 100 };
        f.state()
            .animate_window_move_from(&wa, (seed_x, 150).into(), None);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
    }
    let held_serial = if matches!(finish, TilingFinish::UnresolvedHold) {
        f.state()
            .animate_window_geometry(&wa, (400, 300).into(), None);
        f.state().send_size_configure(&wa, (400, 300).into());
        last_configured(&mut f, first, &a);
        let id = f.state().stage.id_of(&wa).unwrap();
        assert!(f.state().window_animations.start_held(id));
        Some(
            f.client(first)
                .window(&a)
                .configures_received
                .last()
                .unwrap()
                .0,
        )
    } else {
        None
    };
    let translated_before = if matches!(finish, TilingFinish::MoveAnimation) {
        f.state()
            .animate_window_move_from(&wa, (100, 150).into(), None);
        let frame = tiling_live_frame(&mut f, &output);
        f.state().cancel_window_animation(&wa);
        Some(frame)
    } else {
        None
    };
    let opaque_before = tiling_live_frame(&mut f, &output);
    let opening_frames = if matches!(
        finish,
        TilingFinish::OpenAnimation | TilingFinish::ActiveOpen
    ) {
        f.state().start_window_open_animation(&wa);
        if matches!(finish, TilingFinish::ActiveOpen) {
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
        }
        let mut frames = vec![tiling_live_frame(&mut f, &output)];
        for _ in 0..2 {
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            frames.push(tiling_live_frame(&mut f, &output));
        }
        f.state().cancel_window_animation(&wa);
        Some(frames)
    } else {
        None
    };
    if matches!(finish, TilingFinish::ActiveOpen) {
        f.state().start_window_open_animation(&wa);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
    }
    let before = tiling_live_frame(&mut f, &output);
    if popup.is_some() && !matches!(finish, TilingFinish::ActiveOpen) {
        assert!(
            before.as_chunks::<4>().0.contains(&[0, 0, 255, 255]),
            "popup is visible outside the parent"
        );
    }
    if moving_before_capture.is_some() {
        f.state()
            .execute_action(&Action::MoveIntoColumn(driftline::config::Direction::Left));
    } else {
        f.state().execute_action(&Action::ToggleTiling);
    }
    let images = match scene {
        TilingPopupScene::None | TilingPopupScene::Appears => 2,
        TilingPopupScene::Nested => 4,
        _ => 3,
    };
    assert_eq!(f.state().tiling_snapshot_count(), images);
    let target_a = last_configured(&mut f, first, &a);
    let target_b = last_configured(&mut f, second, &b);
    if let Some(serial) = held_serial {
        let id = f.state().stage.id_of(&wa).unwrap();
        assert!(
            f.state()
                .window_animations
                .geometry_visual_rect(id)
                .is_none(),
            "the group owns the held picture now"
        );
        let w = f.client(first).window(&a);
        w.set_size(400, 300);
        w.attach_shm_buffer((400, 300), &solid_bgra(400, 300, [0, 255, 0, 255], false));
        w.xdg_surface.ack_configure(serial);
        w.commit();
        f.double_roundtrip(first);
        assert_eq!(wa.geometry().size, (400, 300).into());
        assert_eq!(f.state().tiling_snapshot_count(), images);
        assert!(
            before == tiling_live_frame(&mut f, &output),
            "late superseded buffers must stay hidden"
        );
    }
    let surface = wb.wl_surface().unwrap().into_owned();
    let barrier = Barrier::new(false);
    let blocker = barrier.clone();
    let hook = add_pre_commit_hook::<crate::state::Driftline, _>(&surface, move |_, _, s| {
        add_blocker(s, blocker.clone());
    });
    for (client, surface, size) in [(first, &a, target_a), (second, &b, target_b)] {
        if matches!(finish, TilingFinish::LateReply) && client == second {
            continue;
        }
        let color = if matches!(
            finish,
            TilingFinish::ReleasePixelAlpha | TilingFinish::ReleaseRetarget
        ) {
            [0, 160, 0, 160]
        } else {
            [0, 255, 0, 255]
        };
        let pixels = solid_bgra(size.0, size.1, color, false);
        let w = f.client(client).window(surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &pixels);
        w.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    f.double_roundtrip(first);
    assert_eq!(
        wa.geometry().size,
        if matches!(finish, TilingFinish::LateReply) {
            (200, 100).into()
        } else {
            target_a.into()
        }
    );
    assert_eq!(wb.geometry().size, (200, 100).into());
    let pending = tiling_live_frame(&mut f, &output);
    let differences: Vec<_> = before
        .as_chunks::<4>()
        .0
        .iter()
        .zip(pending.as_chunks::<4>().0.iter())
        .enumerate()
        .filter(|(_, (a, b))| {
            a.iter()
                .zip(b.iter())
                .any(|(a, b)| a.abs_diff(*b) > u8::from(shadow))
        })
        .take(8)
        .map(|(i, (a, b))| (i % 1200, i / 1200, a, b))
        .collect();
    assert!(
        differences.is_empty(),
        "{scale}x changed pixels: {differences:?}"
    );
    if matches!(scene, TilingPopupScene::Dismiss | TilingPopupScene::Unmap) {
        let menu = f.client(first).popup(popup.as_ref().unwrap());
        if matches!(scene, TilingPopupScene::Unmap) {
            menu.surface.attach(None, 0, 0);
            menu.commit();
        } else {
            menu.destroy();
        }
        f.double_roundtrip(first);
        let dismissed = tiling_live_frame(&mut f, &output);
        assert!(
            !dismissed.as_chunks::<4>().0.contains(&[0, 255, 0, 255]),
            "closing a menu must not expose the early parent buffer"
        );
        assert!(
            !dismissed.as_chunks::<4>().0.contains(&[0, 0, 255, 255]),
            "dismissed menu must not survive in the snapshot"
        );
    }
    if matches!(scene, TilingPopupScene::Appears) {
        colored_tiling_popup(&mut f, first, &a, (180, 50), [255, 0, 0, 255]);
        let opened = tiling_live_frame(&mut f, &output);
        assert!(
            !opened.as_chunks::<4>().0.contains(&[0, 255, 0, 255]),
            "opening a menu must not expose the early parent buffer"
        );
        assert!(
            opened.as_chunks::<4>().0.contains(&[0, 0, 255, 255]),
            "new menus must appear while the parent is waiting"
        );
    }
    if matches!(scene, TilingPopupScene::Update) {
        let menu = f.client(first).popup(popup.as_ref().unwrap());
        menu.attach_shm_buffer((100, 80), &solid_bgra(100, 80, [255, 0, 255, 255], false));
        menu.commit();
        f.double_roundtrip(first);
        let changed = tiling_live_frame(&mut f, &output);
        assert!(
            changed.as_chunks::<4>().0.contains(&[255, 0, 255, 255]),
            "menu updates must remain live"
        );
        assert!(
            !changed.as_chunks::<4>().0.contains(&[0, 0, 255, 255]),
            "the retired menu image must disappear"
        );
        assert!(
            !changed.as_chunks::<4>().0.contains(&[0, 255, 0, 255]),
            "menu updates must not expose the early parent buffer"
        );
    }
    match finish {
        TilingFinish::Buffers
        | TilingFinish::ReleaseSsdFrame
        | TilingFinish::ReleaseCsdFrame
        | TilingFinish::UnresolvedHold
        | TilingFinish::ReleaseAnimation
        | TilingFinish::ReleaseTranslucent
        | TilingFinish::ReleasePixelAlpha
        | TilingFinish::ReleaseRetarget => {}
        TilingFinish::Supersede => {
            f.state().execute_action(&Action::CycleColumnWidth);
            assert_eq!(f.state().tiling_snapshot_count(), images);
            assert!(
                tiling_live_frame(&mut f, &output) == pending,
                "supersession preserves the first picture"
            );
            for (client, surface) in [(first, &a), (second, &b)] {
                let size = last_configured(&mut f, client, surface);
                let pixels = solid_bgra(size.0, size.1, [255, 255, 0, 255], false);
                let w = f.client(client).window(surface);
                w.set_size(size.0 as u16, size.1 as u16);
                w.attach_shm_buffer(size, &pixels);
                w.ack_last_and_commit();
                f.double_roundtrip(client);
            }
            f.double_roundtrip(first);
            assert!(
                tiling_live_frame(&mut f, &output) == pending,
                "newer applied buffers stay hidden too"
            );
        }
        TilingFinish::OpenAnimation
        | TilingFinish::ActiveOpen
        | TilingFinish::MoveAnimation
        | TilingFinish::ResizeAnimation => {
            let generation = f.state().render.blur_geometry_generation;
            match finish {
                TilingFinish::OpenAnimation => f.state().start_window_open_animation(&wa),
                TilingFinish::ActiveOpen => {}
                TilingFinish::MoveAnimation => {
                    f.state()
                        .animate_window_move_from(&wa, (100, 150).into(), None)
                }
                TilingFinish::ResizeAnimation => f.state().animate_window_geometry(
                    &wa,
                    (target_a.0 + 50, target_a.1 + 50).into(),
                    None,
                ),
                _ => unreachable!(),
            }
            let retained = !matches!(finish, TilingFinish::ResizeAnimation);
            let expected_images = if retained { images } else { 1 };
            assert_eq!(f.state().tiling_snapshot_count(), expected_images);
            if !retained {
                assert!(f.state().render.blur_geometry_generation > generation);
            }
            let animated = tiling_live_frame(&mut f, &output);
            if matches!(finish, TilingFinish::ActiveOpen) {
                assert!(
                    animated == pending,
                    "capture preserves the active animation"
                );
            } else {
                assert!(animated != pending, "the new animation must be visible");
            }
            if let Some(frames) = &opening_frames {
                assert!(
                    animated == frames[0],
                    "opening starts from the same transparent picture"
                );
            }
            if let Some(expected) = translated_before {
                let max_delta = animated
                    .iter()
                    .zip(&expected)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    max_delta <= u8::from(shadow),
                    "translated chrome/popups differ by {max_delta}"
                );
            }
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            if !matches!(finish, TilingFinish::ResizeAnimation) {
                assert_ne!(
                    tiling_live_frame(&mut f, &output),
                    animated,
                    "animation advances while waiting"
                );
            }
            if retained {
                let interrupted = tiling_live_frame(&mut f, &output);
                if let Some(frames) = &opening_frames {
                    let delta = interrupted
                        .iter()
                        .zip(&frames[1])
                        .map(|(a, b)| a.abs_diff(*b))
                        .max()
                        .unwrap();
                    let differences: Vec<_> = interrupted
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(frames[1].as_chunks::<4>().0)
                        .enumerate()
                        .filter(|(_, (a, b))| {
                            a.iter()
                                .zip(*b)
                                .any(|(a, b)| a.abs_diff(*b) > u8::from(shadow))
                        })
                        .take(8)
                        .map(|(i, (a, b))| (i % 1200, i / 1200, a, b))
                        .collect();
                    assert!(
                        delta <= u8::from(shadow),
                        "{scale}x shadow={shadow} opening differs by {delta}: {differences:?}"
                    );
                }
                assert!(
                    !interrupted.as_chunks::<4>().0.contains(&[0, 255, 0, 255]),
                    "new green buffers must stay hidden"
                );
                let seed = if matches!(
                    finish,
                    TilingFinish::OpenAnimation | TilingFinish::ActiveOpen
                ) {
                    (50, 50)
                } else {
                    (150, 200)
                };
                f.state().animate_window_move_from(&wa, seed.into(), None);
                assert_eq!(
                    tiling_live_frame(&mut f, &output),
                    interrupted,
                    "retargeting preserves the current picture"
                );
                if let Some(frames) = &opening_frames {
                    f.state()
                        .tick_window_animations(std::time::Duration::from_millis(16));
                    let inherited = tiling_live_frame(&mut f, &output);
                    let delta = inherited
                        .iter()
                        .zip(&frames[2])
                        .map(|(a, b)| a.abs_diff(*b))
                        .max()
                        .unwrap();
                    assert!(
                        delta <= u8::from(shadow),
                        "inherited opening differs by {delta}"
                    );
                }
            }
            f.state().cancel_window_animation(&wa);
            if !matches!(
                finish,
                TilingFinish::OpenAnimation | TilingFinish::ActiveOpen
            ) {
                f.pump(2);
            }
            assert_eq!(f.state().tiling_snapshot_count(), expected_images);
            if retained {
                let restored = tiling_live_frame(&mut f, &output);
                let max_delta = restored
                    .iter()
                    .zip(if matches!(finish, TilingFinish::ActiveOpen) {
                        &opaque_before
                    } else {
                        &before
                    })
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    max_delta <= u8::from(shadow),
                    "cancelled picture differs by {max_delta}"
                );
            }
            assert!(f.state().tiling_view.is_some());
        }
        TilingFinish::OpenDeadline | TilingFinish::OpenExit => {
            f.state().start_window_open_animation(&wa);
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert_eq!(f.state().tiling_snapshot_count(), images);
            if matches!(finish, TilingFinish::OpenExit) {
                f.state().execute_action(&Action::ToggleTiling);
            } else {
                let expired = std::time::Instant::now() + std::time::Duration::from_secs(1);
                f.state().refresh_tiling_at(expired);
                f.pump(4);
                f.state().refresh_tiling_at(expired);
            }
            assert_eq!(f.state().tiling_snapshot_count(), 0);
        }
        TilingFinish::LateReply => {
            let expired = std::time::Instant::now() + std::time::Duration::from_secs(1);
            f.state().refresh_tiling_at(expired);
            f.pump(4);
            f.state().refresh_tiling_at(expired);
            super::tick_until_settled(&mut f);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            let old_rect = smithay::utils::Rectangle::new(
                f.state().stage.position_of(&wb).unwrap().to_f64(),
                wb.geometry().size.to_f64(),
            );
            let resting = tiling_live_frame(&mut f, &output);
            barrier.signal();
            let w = f.client(second).window(&b);
            w.set_size(target_b.0 as u16, target_b.1 as u16);
            w.attach_shm_buffer(
                target_b,
                &solid_bgra(target_b.0, target_b.1, [255, 0, 0, 255], false),
            );
            w.ack_last_and_commit();
            f.double_roundtrip(second);
            let id = f.state().stage.id_of(&wb).unwrap();
            assert_eq!(
                f.state().window_animations.geometry_visual_rect(id),
                Some(old_rect)
            );
            assert!(f.state().resize_crossfades.contains_key(&id));
            let resumed = tiling_live_frame(&mut f, &output);
            let x = ((old_rect.loc.x + 50.0) * scale).round() as usize;
            let y = ((old_rect.loc.y + 50.0) * scale).round() as usize;
            let index = (y * 1200 + x) * 4;
            assert_eq!(
                &resumed[index..index + 4],
                &resting[index..index + 4],
                "late buffers must start from the old picture"
            );
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert_ne!(
                f.state().window_animations.geometry_visual_rect(id),
                Some(old_rect)
            );
            super::tick_until_settled(&mut f);
            let w = f.client(second).window(&b);
            w.attach_shm_buffer(
                target_b,
                &solid_bgra(target_b.0, target_b.1, [255, 0, 0, 255], false),
            );
            w.commit();
            f.double_roundtrip(second);
            assert!(
                f.state()
                    .window_animations
                    .geometry_visual_rect(id)
                    .is_none(),
                "later repaint must not restart the expired resize"
            );
            assert!(!f.state().resize_crossfades.contains_key(&id));
        }
        TilingFinish::Deadline => {
            let expired = std::time::Instant::now() + std::time::Duration::from_secs(1);
            f.state().refresh_tiling_at(expired);
            f.pump(4);
            f.state().refresh_tiling_at(expired);
            f.double_roundtrip(first);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert!(before != tiling_live_frame(&mut f, &output));
        }
        TilingFinish::Exit => {
            f.state().execute_action(&Action::ToggleTiling);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert!(before != tiling_live_frame(&mut f, &output));
        }
    }
    barrier.signal();
    let owner = surface.client().unwrap();
    let client_state = owner.get_data::<crate::state::ClientState>().unwrap();
    let dh = f.state().display_handle.clone();
    client_state
        .compositor_state
        .blocker_cleared(f.state(), &dh);
    f.double_roundtrip(second);
    f.double_roundtrip(first);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    let released = tiling_live_frame(&mut f, &output);
    if matches!(
        finish,
        TilingFinish::ReleaseAnimation
            | TilingFinish::ReleaseSsdFrame
            | TilingFinish::ReleaseCsdFrame
            | TilingFinish::ReleaseTranslucent
            | TilingFinish::ReleasePixelAlpha
            | TilingFinish::ReleaseRetarget
    ) {
        if matches!(
            finish,
            TilingFinish::ReleaseSsdFrame | TilingFinish::ReleaseCsdFrame
        ) {
            let differences: Vec<_> = before
                .as_chunks::<4>()
                .0
                .iter()
                .zip(released.as_chunks::<4>().0)
                .enumerate()
                .filter(|(_, (a, b))| {
                    a.iter()
                        .zip(b.iter())
                        .any(|(a, b)| a.abs_diff(*b) > u8::from(shadow))
                })
                .take(8)
                .map(|(i, (a, b))| (i % 1200, i / 1200, *a, *b))
                .collect();
            assert!(
                differences.is_empty(),
                "{scale}x release frame changed: {differences:?}"
            );
        }
        let id = f.state().stage.id_of(&wa).unwrap();
        let visual = f
            .state()
            .window_animations
            .geometry_visual_rect(id)
            .unwrap();
        assert_eq!(visual.loc, (50.0, 50.0).into());
        assert_eq!(visual.size, (200.0, 100.0).into());
        assert!(f.state().resize_crossfades.contains_key(&id));
        let popup_size = |frame: &[u8]| {
            let points: Vec<_> = frame
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, pixel)| **pixel == [0, 0, 255, 255])
                .map(|(i, _)| (i % 1200, i / 1200))
                .collect();
            let width = points.iter().map(|p| p.0).max().unwrap()
                - points.iter().map(|p| p.0).min().unwrap()
                + 1;
            let height = points.iter().map(|p| p.1).max().unwrap()
                - points.iter().map(|p| p.1).min().unwrap()
                + 1;
            (width, height)
        };
        if matches!(scene, TilingPopupScene::Menu) {
            assert_eq!(
                popup_size(&released),
                popup_size(&before),
                "resize must not stretch menus"
            );
        }
        let x = (100.0 * scale) as usize;
        let y = (90.0 * scale) as usize;
        let sample = (y * 1200 + x) * 4;
        assert_eq!(&released[sample..sample + 4], &before[sample..sample + 4]);
        if matches!(
            finish,
            TilingFinish::ReleaseTranslucent
                | TilingFinish::ReleasePixelAlpha
                | TilingFinish::ReleaseRetarget
        ) {
            assert_eq!(
                released[sample + 3],
                before[sample + 3],
                "release must not increase opacity"
            );
        }
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        assert_ne!(
            f.state()
                .window_animations
                .geometry_visual_rect(id)
                .unwrap(),
            visual
        );
        let advanced = tiling_live_frame(&mut f, &output);
        assert!(released != advanced);
        if matches!(
            finish,
            TilingFinish::ReleaseSsdFrame | TilingFinish::ReleaseCsdFrame
        ) {
            let visual = f
                .state()
                .window_animations
                .geometry_visual_rect(id)
                .unwrap();
            let camera = f.state().camera();
            let x = ((visual.loc.x - camera.x + visual.size.w / 2.0) * scale).round() as usize;
            let content_top = ((visual.loc.y - camera.y) * scale).round() as usize;
            let bar = if matches!(finish, TilingFinish::ReleaseSsdFrame) {
                (f64::from(f.state().config.decorations.title_bar_height) * scale).round() as usize
            } else {
                0
            };
            let expected = content_top - bar;
            let ring: Vec<_> = (0..content_top)
                .filter(|y| {
                    let start = (*y * 1200 + x) * 4;
                    classify(Rgba(advanced[start..start + 4].try_into().unwrap())) == Px::Ring
                })
                .collect();
            assert!(
                ring.last().unwrap().abs_diff(expected) <= 1,
                "{scale}x title-bar band moved: ring={ring:?}, expected edge={expected}"
            );
            assert!(
                ring.len().abs_diff((2.0 * scale).round() as usize) <= 1,
                "{scale}x border stroke stretched: {ring:?}"
            );
            let mut current = advanced.clone();
            for tick_ms in [16, 48, 112] {
                f.state().execute_action(&Action::CycleColumnWidth);
                assert!(f.state().tiling_snapshot_count() > 0);
                let held = tiling_live_frame(&mut f, &output);
                let delta = current
                    .iter()
                    .zip(&held)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(delta <= 2, "{scale}x recaptured frame differs by {delta}");
                f.state()
                    .tick_window_animations(std::time::Duration::from_millis(16));
                assert_eq!(held, tiling_live_frame(&mut f, &output));
                {
                    let size = last_configured(&mut f, second, &b);
                    let pixels = solid_bgra(size.0, size.1, [255, 0, 255, 255], false);
                    let window = f.client(second).window(&b);
                    window.set_size(size.0 as u16, size.1 as u16);
                    window.attach_shm_buffer(size, &pixels);
                    window.ack_last_and_commit();
                    f.double_roundtrip(second);
                }
                f.double_roundtrip(first);
                assert_eq!(f.state().tiling_snapshot_count(), 0);
                let resumed = tiling_live_frame(&mut f, &output);
                let delta = held
                    .iter()
                    .zip(&resumed)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    delta <= 2,
                    "{scale}x tick {tick_ms} recapture release differs by {delta}"
                );
                f.state()
                    .tick_window_animations(std::time::Duration::from_millis(tick_ms));
                current = tiling_live_frame(&mut f, &output);
            }
        }

        if matches!(scene, TilingPopupScene::Menu) {
            assert_eq!(
                popup_size(&advanced),
                popup_size(&before),
                "menus keep their native size during the chase"
            );
        }
        if matches!(finish, TilingFinish::ReleasePixelAlpha) {
            let old_weight = (1.0_f64 - 0.5).powf(0.016 * 60.0).powi(2);
            let expected = [
                96.0 * old_weight * 0.6,
                160.0 * (1.0 - old_weight) * 0.6,
                0.0,
                (96.0 * old_weight + 160.0 * (1.0 - old_weight)) * 0.6,
            ];
            for (actual, expected) in advanced[sample..sample + 4].iter().zip(expected) {
                assert!(
                    actual.abs_diff(expected.round() as u8) <= 2,
                    "premultiplied blend {actual} != {expected}"
                );
            }
            let w = f.client(first).window(&a);
            let blue = solid_bgra(target_a.0, target_a.1, [160, 0, 0, 160], false);
            w.attach_shm_buffer(target_a, &blue);
            w.commit();
            f.double_roundtrip(first);
            let updated = tiling_live_frame(&mut f, &output);
            assert!(
                updated[sample + 1] <= 1,
                "new client pixels must invalidate the cached bake"
            );
            assert!(updated[sample + 2] > 60);
            assert!(updated[sample + 3].abs_diff(advanced[sample + 3]) <= 1);
        }
        if matches!(finish, TilingFinish::ReleaseRetarget) {
            f.state().execute_action(&Action::CycleColumnWidth);
            assert_eq!(f.state().tiling_snapshot_count(), 2);
            assert!(
                f.state()
                    .window_animations
                    .geometry_visual_rect(id)
                    .is_none()
            );
            let held = tiling_live_frame(&mut f, &output);
            let delta = held
                .iter()
                .zip(&advanced)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                delta <= 2,
                "retarget capture differs by {delta} at {scale}x"
            );
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
            assert!(
                held == tiling_live_frame(&mut f, &output),
                "the transaction holds its captured picture"
            );
            {
                let (client, surface) = (second, &b);
                let size = last_configured(&mut f, client, surface);
                let w = f.client(client).window(surface);
                w.set_size(size.0 as u16, size.1 as u16);
                let pixels = solid_bgra(size.0, size.1, [255, 0, 255, 255], false);
                w.attach_shm_buffer(size, &pixels);
                w.ack_last_and_commit();
                f.double_roundtrip(client);
            }
            f.double_roundtrip(first);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            let resumed = tiling_live_frame(&mut f, &output);
            let delta = resumed
                .iter()
                .zip(&held)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                delta <= 2,
                "retarget release differs by {delta} at {scale}x"
            );
            f.state().execute_action(&Action::CycleColumnWidth);
            assert_eq!(f.state().tiling_snapshot_count(), 2);
            let expired = std::time::Instant::now() + std::time::Duration::from_secs(1);
            f.state().refresh_tiling_at(expired);
            f.pump(4);
            f.state().refresh_tiling_at(expired);
            assert_eq!(f.state().tiling_snapshot_count(), 0);
            f.state().execute_action(&Action::ToggleTiling);
            assert!(f.state().resize_crossfades.is_empty());
        }
        for _ in 0..30 {
            f.state()
                .tick_window_animations(std::time::Duration::from_millis(16));
        }
        assert!(!f.state().resize_crossfades.contains_key(&id));
        assert!(
            f.state()
                .window_animations
                .geometry_visual_rect(id)
                .is_none()
        );
    } else {
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        assert!(before != tiling_live_frame(&mut f, &output));
    }
    remove_pre_commit_hook(&surface, &hook);
    if f.state().tiling_view.is_some() {
        f.state().execute_action(&Action::ToggleTiling);
    }
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_keeps_the_old_pixels_until_all_buffers_apply() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::None);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn multi_output_tiling_snapshots_render_on_their_own_output() {
    use super::{last_configured, window_by_app_id};

    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("multi-output tiling snapshots") else {
        return;
    };
    let mut f = Fixture::with_config(super::config(&config_toml(false)));
    let first_output = f.add_output(1, (1200, 800));
    let second_output = f.add_output(2, (1200, 800));
    crate::state::output_state(&second_output).camera = (5000.0, 5000.0).into();
    f.state().focused_output = Some(first_output.clone());
    gl::install(&mut f, renderer);

    let first_client = f.add_client();
    let second_client = f.add_client();
    let first_pixels = solid_bgra(400, 300, CONTENT_BGRA, false);
    let second_color = [0, 255, 0, 255];
    let second_pixels = solid_bgra(400, 300, second_color, false);
    let first_surface = map_window_shm(
        &mut f,
        first_client,
        "first-output",
        (400, 300),
        (400, 300),
        &first_pixels,
    );
    let first_window = window_by_app_id(&mut f, "first-output").unwrap();
    f.state()
        .stage
        .set_position(&first_window, (100, 100).into());
    f.state().focused_output = Some(second_output.clone());
    map_window_shm(
        &mut f,
        second_client,
        "second-output",
        (400, 300),
        (400, 300),
        &second_pixels,
    );
    let second_window = window_by_app_id(&mut f, "second-output").unwrap();
    f.state()
        .stage
        .set_position(&second_window, (5100, 5100).into());

    super::tick_until_settled(&mut f);
    f.state().focused_output = Some(first_output.clone());
    let first_before = tiling_live_frame(&mut f, &first_output);
    let second_before = tiling_live_frame(&mut f, &second_output);
    f.state()
        .execute_action(&driftline::config::Action::ToggleTiling);
    assert_eq!(f.state().tiling_snapshot_count(), 2);

    assert_eq!(
        first_before,
        tiling_live_frame(&mut f, &first_output),
        "output one must render its own retained image"
    );
    assert_eq!(
        second_before,
        tiling_live_frame(&mut f, &second_output),
        "output two must render its own retained image"
    );

    let first_target = last_configured(&mut f, first_client, &first_surface);
    let first = f.client(first_client).window(&first_surface);
    first.set_size(first_target.0 as u16, first_target.1 as u16);
    first.attach_shm_buffer(
        first_target,
        &solid_bgra(first_target.0, first_target.1, second_color, false),
    );
    first.ack_last_and_commit();
    f.double_roundtrip(first_client);
    assert_eq!(f.state().tiling_snapshot_count(), 1);
    assert!(
        second_before == tiling_live_frame(&mut f, &second_output),
        "completing output one's configure must leave output two's image held"
    );

    for output in [&first_output, &second_output] {
        f.state().render.remove_output(&output.name());
    }
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_pixels_are_released_at_the_group_deadline() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Deadline, false, TilingPopupScene::None);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn exiting_tiling_drops_the_pending_pixel_snapshot() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Exit, false, TilingPopupScene::None);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_snapshot_keeps_the_shadow_aligned() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, true, TilingPopupScene::None);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_keeps_popup_overhangs_and_geometry_offsets() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::Menu);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_keeps_nested_popups_in_their_original_order() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::Nested);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_dismissed_popup_is_not_kept_by_the_tiling_snapshot() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::Dismiss);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn an_unmapped_popup_is_not_kept_by_the_tiling_snapshot() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::Unmap);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_popup_opened_while_tiling_waits_is_visible() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(TilingFinish::Buffers, false, TilingPopupScene::Appears);
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_preserves_pixels_at_fractional_output_scales() {
    let _gl = gl::lock();
    for scale in [1.25, 1.5] {
        check_pending_tiling_pixels_at_scale(
            TilingFinish::Buffers,
            false,
            TilingPopupScene::None,
            scale,
        );
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_preserves_popup_trees_at_fractional_output_scales() {
    let _gl = gl::lock();
    for scale in [1.25, 1.5] {
        check_pending_tiling_pixels_at_scale(
            TilingFinish::Buffers,
            false,
            TilingPopupScene::Nested,
            scale,
        );
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn pending_tiling_preserves_shadows_at_fractional_output_scales() {
    let _gl = gl::lock();
    for scale in [1.25, 1.5] {
        check_pending_tiling_pixels_at_scale(
            TilingFinish::Buffers,
            true,
            TilingPopupScene::None,
            scale,
        );
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn opening_animation_preserves_a_pending_tiling_picture_and_its_popups() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for shadow in [false, true] {
            check_pending_tiling_pixels_at_scale(
                TilingFinish::OpenAnimation,
                shadow,
                TilingPopupScene::Nested,
                scale,
            );
        }
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn geometry_animation_releases_a_pending_tiling_picture_and_its_popups() {
    let _gl = gl::lock();
    check_pending_tiling_pixels(
        TilingFinish::ResizeAnimation,
        true,
        TilingPopupScene::Nested,
    );
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn position_animation_preserves_pending_tiling_content_and_popups() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for shadow in [false, true] {
            check_pending_tiling_pixels_at_scale(
                TilingFinish::MoveAnimation,
                shadow,
                TilingPopupScene::Nested,
                scale,
            );
        }
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn tiling_captures_content_and_popups_during_an_active_translation() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::Buffers,
            true,
            TilingPopupScene::Nested,
            scale,
            Some(50),
        );
    }
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn tiling_supersession_preserves_an_active_translation_picture() {
    let _gl = gl::lock();
    check_pending_tiling_capture(
        TilingFinish::Supersede,
        false,
        TilingPopupScene::Nested,
        1.0,
        Some(50),
    );
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn a_deadline_releases_the_picture_captured_during_motion() {
    let _gl = gl::lock();
    check_pending_tiling_capture(
        TilingFinish::Deadline,
        false,
        TilingPopupScene::Nested,
        1.25,
        Some(50),
    );
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn exiting_tiling_releases_the_picture_captured_during_motion() {
    let _gl = gl::lock();
    check_pending_tiling_capture(
        TilingFinish::Exit,
        false,
        TilingPopupScene::Nested,
        1.5,
        Some(50),
    );
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn tiling_captures_a_visible_translation_with_an_offscreen_destination() {
    let _gl = gl::lock();
    check_pending_tiling_capture(
        TilingFinish::Buffers,
        true,
        TilingPopupScene::Nested,
        1.5,
        Some(-1000),
    );
}

#[test]
#[ignore = "needs Mesa surfaceless EGL; run with --include-ignored"]
fn opening_tiling_pictures_release_on_deadline_and_exit() {
    let _gl = gl::lock();
    for finish in [TilingFinish::OpenDeadline, TilingFinish::OpenExit] {
        check_pending_tiling_pixels_at_scale(finish, false, TilingPopupScene::Nested, 1.25);
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_captures_an_already_opening_window_and_its_popups() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for shadow in [false, true] {
            check_pending_tiling_capture(
                TilingFinish::ActiveOpen,
                shadow,
                TilingPopupScene::Nested,
                scale,
                None,
            );
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_release_animates_from_the_retained_size_and_old_content() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleaseAnimation,
            false,
            TilingPopupScene::None,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_release_does_not_stack_translucent_content() {
    let _gl = gl::lock();
    check_pending_tiling_capture(
        TilingFinish::ReleaseTranslucent,
        true,
        TilingPopupScene::None,
        1.25,
        None,
    );
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_resize_blends_pixel_alpha_before_window_opacity() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleasePixelAlpha,
            false,
            TilingPopupScene::None,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn repeated_tiling_resize_holds_and_resumes_the_blended_picture() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleaseRetarget,
            false,
            TilingPopupScene::None,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn pending_tiling_menu_updates_do_not_release_parent_content() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::Buffers,
            false,
            TilingPopupScene::Update,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_resize_keeps_live_menus_at_native_size() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleaseAnimation,
            false,
            TilingPopupScene::Menu,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_captures_an_unresolved_resize_and_hides_its_late_reply() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::UnresolvedHold,
            false,
            TilingPopupScene::Menu,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn late_tiling_buffers_animate_without_reopening_the_expired_group() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::LateReply,
            false,
            TilingPopupScene::Menu,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_release_preserves_the_complete_ssd_frame() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleaseSsdFrame,
            true,
            TilingPopupScene::Menu,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_release_preserves_the_complete_csd_frame() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        check_pending_tiling_capture(
            TilingFinish::ReleaseCsdFrame,
            true,
            TilingPopupScene::Menu,
            scale,
            None,
        );
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_resize_release_preserves_an_inherited_opening_picture() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for opening_scale in [0.9, 1.0] {
                check_opening_resize_release(scale, ssd, opening_scale, false);
            }
        }
    }
}

fn check_opening_resize_release(scale: f64, ssd: bool, opening_scale: f64, blur: bool) {
    use super::{last_configured, map_window_shm, window_by_app_id};
    use driftline::config::Action;
    let Some(renderer) = gl::surfaceless_renderer("opening resize") else {
        return;
    };
    let config = format!(
        "{}\n[effects]\nanimation_speed = 0.25\nanimation_scale = {opening_scale}\n",
        config_toml(true)
            .replace("corner_radius = 0", "corner_radius = 8")
            .replace(
                "[decorations]",
                "[decorations]\nopacity = 0.6\nopacity_focused = 0.6"
            )
    );
    let config = if ssd {
        format!(
            "{config}\n[[window_rules]]\napp_id = \"a\"\ndecoration = \"server\"\n\n[[window_rules]]\napp_id = \"b\"\ndecoration = \"server\"\n"
        )
    } else {
        config
    };
    let temp = TempDir::new();
    let shader = temp.path().join("opening-stripes.glsl");
    std::fs::write(&shader, "precision mediump float; varying vec2 v_coords; void main() { float s=step(0.5,fract(v_coords.x*60.0)); gl_FragColor=vec4(vec3(s),1.0); }").unwrap();
    let config = if blur {
        config
            .replace(
                "type = \"none\"",
                &format!("type = \"shader\"\npath = \"{}\"", shader.display()),
            )
            .replace("[decorations]", "[decorations]\nblur = true")
    } else {
        config
    };
    let mut renderer = renderer;
    let blur_shaders = blur.then(|| crate::render::compile_blur_shaders(&mut renderer));
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    if let Some((down, up, mask)) = blur_shaders {
        assert!(down.is_some() && up.is_some() && mask.is_some());
        f.state().render.blur_down_shader = down;
        f.state().render.blur_up_shader = up;
        f.state().render.blur_mask_shader = mask;
    }
    let first = f.add_client();
    let second = f.add_client();
    let old = solid_bgra(200, 100, [0, 0, 128, 128], false);
    let a = map_window_shm(&mut f, first, "a", (200, 100), (200, 100), &old);
    let b = map_window_shm(&mut f, second, "b", (200, 100), (200, 100), &old);
    super::tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().stage.set_position(&wa, (50, 80).into());
    f.state().stage.set_position(&wb, (350, 80).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, first, &a, (180, 50), [255, 0, 0, 255]);
    f.state().start_window_open_animation(&wa);
    f.state().start_window_open_animation(&wb);
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(48));
    let id = f.state().stage.id_of(&wb).unwrap();
    let first_frame = tiling_live_frame(&mut f, &output);
    let updated = solid_bgra(200, 100, [0, 0, 96, 96], false);
    let window = f.client(first).window(&a);
    window.attach_shm_buffer((200, 100), &updated);
    window.commit();
    f.double_roundtrip(first);
    let original = tiling_live_frame(&mut f, &output);
    assert!(
        first_frame != original,
        "opening cache must follow client commits"
    );
    assert_eq!(f.state().render.animated_content.len(), 2);
    if blur {
        assert_eq!(f.state().render.blur_cache.len(), 2);
    }
    f.state().execute_action(&Action::ToggleTiling);
    let captured = tiling_live_frame(&mut f, &output);
    let delta = original
        .iter()
        .zip(&captured)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x SSD={ssd} blur={blur}: first opening capture differs by {delta}"
    );
    for round in 0..3 {
        assert!(f.state().tiling_snapshot_count() > 0);
        let previous = f.state().window_animations.open_progress(id).unwrap();
        let waiting = tiling_live_frame(&mut f, &output);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        assert!(f.state().window_animations.open_progress(id).unwrap() > previous);
        let held = tiling_live_frame(&mut f, &output);
        assert!(waiting != held);
        let progress = f.state().window_animations.open_progress(id).unwrap();
        let clients = if round == 0 {
            vec![(first, &a), (second, &b)]
        } else {
            vec![(second, &b)]
        };
        for (client, surface) in clients {
            let size = last_configured(&mut f, client, surface);
            let pixels = solid_bgra(
                size.0,
                size.1,
                if round % 2 == 0 {
                    [0, 128, 0, 128]
                } else {
                    [128, 0, 0, 128]
                },
                false,
            );
            let window = f.client(client).window(surface);
            window.set_size(size.0 as u16, size.1 as u16);
            window.attach_shm_buffer(size, &pixels);
            window.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        assert_eq!(f.state().tiling_snapshot_count(), 0);
        assert_eq!(
            f.state().window_animations.open_progress(id),
            Some(progress)
        );
        assert!(f.state().resize_crossfades.contains_key(&id));
        let released = tiling_live_frame(&mut f, &output);
        let delta = held
            .iter()
            .zip(&released)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        let differences: Vec<_> = held
            .as_chunks::<4>()
            .0
            .iter()
            .zip(released.as_chunks::<4>().0)
            .enumerate()
            .filter(|(_, (a, b))| a.iter().zip(b.iter()).any(|(a, b)| a.abs_diff(*b) > 2))
            .take(8)
            .map(|(i, (a, b))| (i % 1200, i / 1200, *a, *b))
            .collect();
        assert!(
            delta <= 2,
            "{scale}x SSD={ssd} round {round}: opening release differs by {delta}: {differences:?}"
        );
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        assert!(f.state().window_animations.open_progress(id).unwrap() > progress);
        let moving = tiling_live_frame(&mut f, &output);
        assert!(moving != released);
        if round != 2 {
            f.state().execute_action(&Action::CycleColumnWidth);
            let captured = tiling_live_frame(&mut f, &output);
            let delta = moving
                .iter()
                .zip(&captured)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                delta <= 2,
                "{scale}x SSD={ssd}: opening recapture differs by {delta}"
            );
        }
    }
    super::tick_until_settled(&mut f);
    assert!(f.state().window_animations.open_progress(id).is_none());
    assert!(f.state().resize_crossfades.is_empty());
    tiling_live_frame(&mut f, &output);
    assert!(f.state().render.animated_content.is_empty());
    f.state().execute_action(&Action::ToggleTiling);
    f.state().render.remove_output(&output.name());
    f.state().render.blur_down_shader = None;
    f.state().render.blur_up_shader = None;
    f.state().render.blur_mask_shader = None;
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_resize_release_preserves_frost_during_inherited_opening() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for opening_scale in [0.9, 1.0] {
                check_opening_resize_release(scale, ssd, opening_scale, true);
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_captures_odd_offset_geometry_during_framed_canvas_resize() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for growing in [false, true] {
                check_odd_offset_resize_capture(scale, ssd, growing, false);
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn tiling_recapture_handles_fractional_buffers_and_changed_geometry_offsets() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for growing in [false, true] {
                check_odd_offset_resize_capture(scale, ssd, growing, true);
            }
        }
    }
}

fn check_odd_offset_resize_capture(scale: f64, ssd: bool, growing: bool, fractional: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use crate::state::window_animation::{AnimSpace, ContentPolicy, GeometryRole};
    use driftline::config::Action;
    use smithay::utils::Rectangle;

    let Some(renderer) = gl::surfaceless_renderer("odd offset canvas resize") else {
        return;
    };
    let config = config_toml(true)
        .replace("corner_radius = 0", "corner_radius = 8")
        .replace(
            "[decorations]",
            "[decorations]\nopacity = 0.6\nopacity_focused = 0.6",
        );
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"odd\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let paint = |size: (i32, i32), color: [u8; 4]| {
        let buffer_scale = if fractional { scale } else { 1.0 };
        let texels = (
            (f64::from(size.0) * buffer_scale).ceil() as i32,
            (f64::from(size.1) * buffer_scale).ceil() as i32,
        );
        let mut pixels = solid_bgra(texels.0, texels.1, color, false);
        if fractional {
            for y in 0..texels.1 {
                for x in 0..texels.0 {
                    let i = ((y * texels.0 + x) * 4) as usize;
                    if x > texels.0 / 3 && x < texels.0 / 2 && y > texels.1 / 3 && y < texels.1 / 2
                    {
                        pixels[i..i + 4].fill(0);
                    } else if (x / 17 + y / 13) % 2 == 0 {
                        for channel in &mut pixels[i..i + 4] {
                            *channel /= 2;
                        }
                    }
                }
            }
        }
        (texels, pixels)
    };
    let (texels, pixels) = paint((313, 223), [255, 255, 255, 255]);
    let surface = map_window_shm(&mut f, client, "odd", (313, 223), texels, &pixels);
    f.client(client)
        .window(&surface)
        .xdg_surface
        .set_window_geometry(7, 9, 299, 201);
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "odd").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    assert_eq!(
        window.geometry(),
        Rectangle::new((7, 9).into(), (299, 201).into())
    );
    f.state().stage.set_position(&window, (121, 161).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, client, &surface, (280, 51), [255, 0, 0, 255]);
    f.state().begin_geometry_animation_seeded(
        &window,
        Rectangle::new(
            (103.0, 143.0).into(),
            if growing {
                (191.0, 131.0)
            } else {
                (411.0, 281.0)
            }
            .into(),
        ),
        AnimSpace::Canvas,
        None,
        GeometryRole::Normal,
        ContentPolicy::Stretch,
        Some((121, 161).into()),
    );
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(16));
    let before = tiling_live_frame(&mut f, &output);
    let visual = f
        .state()
        .window_animations
        .geometry_visual_rect(id)
        .unwrap();
    let popup_bounds = |frame: &[u8]| {
        let points: Vec<_> = frame
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, pixel)| pixel[2] > pixel[0] + 40 && pixel[2] > pixel[1] + 40)
            .map(|(i, _)| ((i % 1200) as i32, (i / 1200) as i32))
            .collect();
        let left = points.iter().map(|p| p.0).min().expect("visible popup");
        let top = points.iter().map(|p| p.1).min().unwrap();
        let width = points.iter().map(|p| p.0).max().unwrap() - left + 1;
        let height = points.iter().map(|p| p.1).max().unwrap() - top + 1;
        assert_eq!(
            (width, height),
            (
                (100.0 * scale).round() as i32,
                (80.0 * scale).round() as i32
            ),
            "resize must preserve native popup dimensions"
        );
        (left, top)
    };
    let popup_before = popup_bounds(&before);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    let compare = |a: &[u8], b: &[u8], phase: &str| {
        let delta = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
        assert!(
            delta <= 2,
            "{scale}x SSD={ssd} growing={growing} {phase}: delta {delta}"
        );
    };
    let held = tiling_live_frame(&mut f, &output);
    compare(&before, &held, "capture");
    let size = last_configured(&mut f, client, &surface);
    let offset = if fractional { (11, 5) } else { (7, 9) };
    let logical = (size.0 + 2 * offset.0, size.1 + 2 * offset.1);
    let (texels, pixels) = paint(logical, [0, 255, 0, 255]);
    let w = f.client(client).window(&surface);
    w.set_size(logical.0 as u16, logical.1 as u16);
    w.xdg_surface
        .set_window_geometry(offset.0, offset.1, size.0, size.1);
    w.attach_shm_buffer(texels, &pixels);
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(
        f.state().window_animations.geometry_visual_rect(id),
        Some(visual)
    );
    compare(&held, &tiling_live_frame(&mut f, &output), "release");
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(16));
    let mixed = tiling_live_frame(&mut f, &output);
    let popup_after = popup_bounds(&mixed);
    let advanced = f
        .state()
        .window_animations
        .geometry_visual_rect(id)
        .unwrap();
    let camera = f.state().camera();
    let displacement = (
        ((advanced.loc.x - camera.x) * scale).round() as i32
            - (visual.loc.x * scale).round() as i32,
        ((advanced.loc.y - camera.y) * scale).round() as i32
            - (visual.loc.y * scale).round() as i32,
    );
    assert_eq!(
        (
            popup_after.0 - popup_before.0,
            popup_after.1 - popup_before.1
        ),
        displacement,
        "popup must follow the parent geometry anchor"
    );
    f.state().execute_action(&Action::CycleColumnWidth);
    compare(&mixed, &tiling_live_frame(&mut f, &output), "recapture");
    if fractional {
        let held = tiling_live_frame(&mut f, &output);
        let offered = last_configured(&mut f, client, &surface);
        let next_size = if growing {
            offered
        } else {
            (size.0 / 2 + 1, size.1 / 2 + 1)
        };
        assert_eq!(
            next_size.0 > size.0,
            growing,
            "exercise both buffer-size directions"
        );
        let logical = (next_size.0 + 6, next_size.1 + 26);
        let (texels, pixels) = paint(logical, CONTENT_BGRA);
        let w = f.client(client).window(&surface);
        w.set_size(logical.0 as u16, logical.1 as u16);
        w.xdg_surface
            .set_window_geometry(3, 13, next_size.0, next_size.1);
        w.attach_shm_buffer(texels, &pixels);
        w.ack_last_and_commit();
        f.double_roundtrip(client);
        assert_eq!(f.state().tiling_snapshot_count(), 0);
        assert_eq!(
            window.geometry(),
            Rectangle::new((3, 13).into(), next_size.into())
        );
        compare(
            &held,
            &tiling_live_frame(&mut f, &output),
            "recapture release",
        );
        tick_until_settled(&mut f);
        let settled = tiling_live_frame(&mut f, &output);
        assert!(!f.state().resize_crossfades.contains_key(&id));
        let position = f.state().stage.position_of(&window).unwrap();
        let camera = f.state().camera();
        let sample = |x: f64, y: f64| {
            let x = ((f64::from(position.x - 3) - camera.x + x) * scale).round() as usize;
            let y = ((f64::from(position.y - 13) - camera.y + y) * scale).round() as usize;
            assert!(
                x < 1200 && y < 800,
                "independent sample must remain visible"
            );
            &settled[(y * 1200 + x) * 4..(y * 1200 + x) * 4 + 4]
        };
        let hole = sample(
            f64::from(logical.0) * 5.0 / 12.0,
            f64::from(logical.1) * 5.0 / 12.0,
        );
        assert!(
            hole[3] <= 2,
            "transparent client hole must survive settling: {hole:?}"
        );
        let tx = ((f64::from(texels.0) * 0.65) as i32 / 17) * 17 + 8;
        let ty = ((f64::from(texels.1) * 0.65) as i32 / 13) * 13 + 6;
        let expected = if (tx / 17 + ty / 13) % 2 == 0 {
            76_u8
        } else {
            153_u8
        };
        let stripe = sample(f64::from(tx) / scale, f64::from(ty) / scale);
        assert!(
            stripe[0].abs_diff(expected) <= 2
                && stripe[3].abs_diff(expected) <= 2
                && stripe[1] == 0
                && stripe[2] == 0,
            "settled stripe must retain client colour and opacity: {stripe:?}, expected {expected}"
        );
    }
    f.state().execute_action(&Action::ToggleTiling);
    tick_until_settled(&mut f);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn degraded_canvas_resize_keeps_capped_live_content_when_tiling_starts() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for reply in [false, true] {
                check_degraded_canvas_resize(scale, ssd, reply, false);
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn degraded_canvas_resize_preserves_minified_content_when_tiling_starts() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for reply in [false, true] {
                check_degraded_canvas_resize(scale, ssd, reply, true);
            }
        }
    }
}

fn check_degraded_canvas_resize(scale: f64, ssd: bool, reply: bool, shrinking: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::Action;
    use std::time::{Duration, Instant};

    let Some(renderer) = gl::surfaceless_renderer("degraded canvas resize") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"degraded\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "degraded",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "degraded").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    f.state().stage.set_position(&window, (100, 120).into());
    f.state().set_camera((0.0, 0.0).into());
    let requested = if shrinking { (120, 60) } else { (600, 400) };
    f.state()
        .animate_window_geometry(&window, requested.into(), None);
    f.state().send_size_configure(&window, requested.into());
    last_configured(&mut f, client, &surface);
    let now = Instant::now();
    let tick = Duration::from_millis(16);
    f.state().tick_window_animations_at(tick, now);
    let degraded = now + Duration::from_secs(1);
    f.state().tick_window_animations_at(tick, degraded);
    let visual = f
        .state()
        .animated_visual(id, (100.0, 120.0).into(), (200.0, 100.0).into());
    assert!(visual.cap_content);
    assert_eq!(visual.size.w < 200.0 && visual.size.h < 100.0, shrinking);
    assert!(!f.state().window_animations.can_capture_canvas_geometry(id));
    let before = tiling_live_frame(&mut f, &output);
    let content_bounds = |frame: &[u8], extent: (f64, f64)| {
        let points: Vec<_> = frame
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| **p == [255, 0, 0, 255])
            .map(|(i, _)| (i % 1200, i / 1200))
            .collect();
        let w = points.iter().map(|p| p.0).max().unwrap()
            - points.iter().map(|p| p.0).min().unwrap()
            + 1;
        let h = points.iter().map(|p| p.1).max().unwrap()
            - points.iter().map(|p| p.1).min().unwrap()
            + 1;
        assert_eq!(
            (w, h),
            (
                (extent.0 * scale).round() as usize,
                (extent.1 * scale).round() as usize
            ),
            "stale client content must never be magnified"
        );
    };
    let extent = (visual.size.w.min(200.0), visual.size.h.min(100.0));
    content_bounds(&before, extent);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(
        f.state().tiling_snapshot_count(),
        0,
        "degraded content uses live fallback"
    );
    let held = tiling_live_frame(&mut f, &output);
    let delta = before
        .iter()
        .zip(&held)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x SSD={ssd}: live fallback entry differs by {delta}"
    );
    content_bounds(&held, extent);
    for _ in 0..3 {
        f.state().execute_action(&Action::CycleColumnWidth);
        assert_eq!(f.state().tiling_snapshot_count(), 0);
        assert!(
            held == tiling_live_frame(&mut f, &output),
            "superseding the group must preserve the capped live picture"
        );
    }
    let size = last_configured(&mut f, client, &surface);
    if reply {
        let w = f.client(client).window(&surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
    } else {
        let expired = degraded + Duration::from_secs(1);
        f.state().refresh_tiling_at(expired);
        f.pump(4);
        f.state().refresh_tiling_at(expired);
    }
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    for i in 0..600 {
        if !f.state().window_animations.is_active() {
            break;
        }
        f.state().tick_window_animations_at(
            tick,
            degraded + Duration::from_secs(2) + Duration::from_millis(i * 16),
        );
    }
    assert!(!f.state().window_animations.is_active());
    let settled = tiling_live_frame(&mut f, &output);
    if !reply {
        content_bounds(&settled, (200.0, 100.0));
    }
    assert!(!f.state().resize_crossfades.contains_key(&id));
    f.state().execute_action(&Action::ToggleTiling);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn held_tiling_images_survive_output_scale_and_camera_changes() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for (from, to) in [(1.0, 1.25), (1.25, 1.5), (1.5, 1.0)] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_tiling_view_change(from, to, ssd, mixed, TilingViewChange::Scale);
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn held_tiling_images_release_after_fractional_physical_camera_motion() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_tiling_view_change(
                    scale,
                    scale,
                    ssd,
                    mixed,
                    TilingViewChange::FractionalCamera,
                );
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn held_tiling_images_survive_output_mode_height_changes() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_tiling_view_change(
                    scale,
                    scale,
                    ssd,
                    mixed,
                    TilingViewChange::Mode(1200, 1000),
                );
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn held_tiling_images_survive_mode_width_changes_and_shrink() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for (width, height) in [(1600, 800), (900, 800), (1200, 600), (900, 600)] {
        for scale in [1.0, 1.25, 1.5] {
            for ssd in [false, true] {
                for mixed in [false, true] {
                    check_tiling_view_change(
                        scale,
                        scale,
                        ssd,
                        mixed,
                        TilingViewChange::Mode(width, height),
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TilingViewChange {
    Scale,
    FractionalCamera,
    Mode(i32, i32),
    Transform(&'static str, smithay::utils::Transform),
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn held_tiling_images_survive_rotated_and_reflected_output_rendering() {
    use smithay::utils::Transform;
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for (name, transform) in [
        ("90", Transform::_90),
        ("180", Transform::_180),
        ("270", Transform::_270),
        ("flipped", Transform::Flipped),
        ("flipped-90", Transform::Flipped90),
        ("flipped-180", Transform::Flipped180),
        ("flipped-270", Transform::Flipped270),
    ] {
        for scale in [1.0, 1.25, 1.5] {
            for ssd in [false, true] {
                for mixed in [false, true] {
                    check_tiling_view_change(
                        scale,
                        scale,
                        ssd,
                        mixed,
                        TilingViewChange::Transform(name, transform),
                    );
                }
            }
        }
    }
}

fn check_tiling_view_change(from: f64, to: f64, ssd: bool, mixed: bool, change: TilingViewChange) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::Action;
    let Some(renderer) = gl::surfaceless_renderer("held output view change") else {
        return;
    };
    let config = config_toml(true)
        .replace("corner_radius = 0", "corner_radius = 8")
        .replace(
            "[decorations]",
            "[decorations]\nopacity = 0.6\nopacity_focused = 0.6",
        );
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"scaled\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), from);
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "scaled",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [255, 255, 255, 255], false),
    );
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "scaled").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    f.state().stage.set_position(&window, (100, 120).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, client, &surface, (180, 51), [255, 0, 0, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    let mut held_origin = smithay::utils::Point::from((100.0, 120.0));
    if mixed {
        let size = last_configured(&mut f, client, &surface);
        let w = f.client(client).window(&surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        held_origin = f
            .state()
            .window_animations
            .geometry_visual_rect(id)
            .unwrap()
            .loc;
        f.state().execute_action(&Action::CycleColumnWidth);
        assert_eq!(f.state().tiling_snapshot_count(), 2);
    }
    let mode_size = if let TilingViewChange::Mode(width, height) = change {
        (width, height)
    } else {
        (1200, 800)
    };
    let config = format!(
        "{config}\n[[outputs]]\nname = {:?}\nmode = \"{}x{}\"\nscale = {to}\n",
        output.name(),
        mode_size.0,
        mode_size.1
    );
    let config = if let TilingViewChange::Transform(name, _) = change {
        format!("{config}transform = {name:?}\n")
    } else {
        config
    };
    f.state().reload_config_from_contents(&config);
    if let TilingViewChange::Transform(_, transform) = change {
        assert_eq!(output.current_transform(), transform);
    }
    assert_eq!(output.current_scale().fractional_scale(), to);
    if matches!(change, TilingViewChange::Mode(_, _)) {
        let intent = f
            .state()
            .pending_mode_changes
            .remove(&output.name())
            .expect("backend mode request");
        assert!(
            matches!(intent, crate::state::ModeIntent::Custom { w, h, .. }
            if (w, h) == mode_size)
        );
        let mode = smithay::output::Mode {
            size: mode_size.into(),
            refresh: 60_000,
        };
        output.change_current_state(Some(mode), None, None, None);
        output.set_preferred(mode);
        smithay::desktop::layer_map_for_output(&output).arrange();
        let size = crate::state::output_logical_size(&output);
        f.state().resize_fullscreen_for_output(&output, size);
        f.state().render.remove_output(&output.name());
        f.state().output_config_dirty = true;
    }
    assert!(f.state().pending_mode_changes.is_empty());
    let transformed_size = output
        .current_transform()
        .transform_size(output.current_mode().unwrap().size);
    let frame_width = transformed_size.w as usize;
    let frame_height = transformed_size.h as usize;
    let stationary = tiling_live_frame(&mut f, &output);
    let camera = if matches!(change, TilingViewChange::FractionalCamera) {
        (16.4, 12.2)
    } else {
        (16.0, 12.0)
    };
    f.state().set_camera(camera.into());
    f.state().refresh_tiling();
    assert!(f.state().tiling_view.is_some());
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    let held = tiling_live_frame(&mut f, &output);
    if matches!(
        change,
        TilingViewChange::Scale | TilingViewChange::Mode(_, _)
    ) {
        let dx = (16.0 * to).round() as usize;
        let dy = (12.0 * to).round() as usize;
        let mut camera_delta = 0;
        for y in 0..frame_height - dy {
            for x in 0..frame_width - dx {
                let destination = (y * frame_width + x) * 4;
                let source = ((y + dy) * frame_width + x + dx) * 4;
                for channel in 0..4 {
                    camera_delta = camera_delta
                        .max(held[destination + channel].abs_diff(stationary[source + channel]));
                }
            }
        }
        assert!(
            camera_delta <= 2,
            "{change:?} {from}→{to} SSD={ssd} mixed={mixed}: camera translation differs by {camera_delta}"
        );
    }
    let popup_origin = |frame: &[u8]| {
        let points: Vec<_> = frame
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p[2] > p[0] + 40 && p[2] > p[1] + 40)
            .map(|(i, _)| ((i % frame_width) as i32, (i / frame_width) as i32))
            .collect();
        (
            points.iter().map(|p| p.0).min().unwrap(),
            points.iter().map(|p| p.1).min().unwrap(),
        )
    };
    let old_popup = popup_origin(&stationary);
    let new_popup = popup_origin(&held);
    let expected = (
        ((held_origin.x - camera.0) * to).round() as i32 - (held_origin.x * to).round() as i32,
        ((held_origin.y - camera.1) * to).round() as i32 - (held_origin.y * to).round() as i32,
    );
    assert_eq!(
        (new_popup.0 - old_popup.0, new_popup.1 - old_popup.1),
        expected,
        "fractional camera motion must preserve the popup's parent anchor"
    );
    let points: Vec<_> = held
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, p)| p[2] > p[0] + 40 && p[2] > p[1] + 40)
        .map(|(i, _)| (i % frame_width, i / frame_width))
        .collect();
    let width =
        points.iter().map(|p| p.0).max().unwrap() - points.iter().map(|p| p.0).min().unwrap() + 1;
    let height =
        points.iter().map(|p| p.1).max().unwrap() - points.iter().map(|p| p.1).min().unwrap() + 1;
    assert_eq!(
        (width, height),
        ((100.0 * to).round() as usize, (80.0 * to).round() as usize),
        "popup must use the new output scale"
    );
    let size = last_configured(&mut f, client, &surface);
    if matches!(
        change,
        TilingViewChange::Mode(_, _) | TilingViewChange::Transform(_, _)
    ) {
        let chrome = f.state().element_chrome(&window);
        let expected_height =
            (frame_height as f64 / to).round() as i32 - chrome.bar - 2 * chrome.border;
        assert_eq!(
            size.1, expected_height,
            "new mode must reflow the tile to the new usable height"
        );
        if matches!(change, TilingViewChange::Mode(_, _)) {
            let fraction = if mixed { 1.0 } else { 0.5 };
            let expected_width =
                ((frame_width as f64 / to).round() * fraction).round() as i32 - 2 * chrome.border;
            assert_eq!(
                size.0, expected_width,
                "mode change must recompute proportional column width"
            );
        }
    }
    let w = f.client(client).window(&surface);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    let released = tiling_live_frame(&mut f, &output);
    let delta = held
        .iter()
        .zip(&released)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{change:?} {from}→{to} SSD={ssd} mixed={mixed}: scale-change release differs by {delta}"
    );
    f.state()
        .tick_window_animations(std::time::Duration::from_millis(16));
    let mixed_frame = tiling_live_frame(&mut f, &output);
    if ssd {
        let visual = f
            .state()
            .window_animations
            .geometry_visual_rect(id)
            .unwrap();
        let camera = f.state().camera();
        let x = ((visual.loc.x - camera.x + visual.size.w / 2.0) * to).round() as usize;
        let y = ((visual.loc.y
            - camera.y
            - f64::from(f.state().config.decorations.title_bar_height) / 2.0)
            * to)
            .round() as usize;
        let alpha = mixed_frame[(y * frame_width + x) * 4 + 3];
        assert!(
            alpha.abs_diff(153) <= 2,
            "title alpha must not accumulate: {alpha}"
        );
    }
    f.state().execute_action(&Action::CycleColumnWidth);
    let recaptured = tiling_live_frame(&mut f, &output);
    let delta = mixed_frame
        .iter()
        .zip(&recaptured)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{change:?} {from}→{to} SSD={ssd} mixed={mixed}: title recapture differs by {delta}"
    );
    let size = last_configured(&mut f, client, &surface);
    let w = f.client(client).window(&surface);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, CONTENT_BGRA, false));
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    let resumed = tiling_live_frame(&mut f, &output);
    let delta = recaptured
        .iter()
        .zip(&resumed)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{change:?} {from}→{to} SSD={ssd} mixed={mixed}: title recapture release differs by {delta}"
    );
    tick_until_settled(&mut f);
    tiling_live_frame(&mut f, &output);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    f.state().execute_action(&Action::ToggleTiling);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn output_transform_pixels_match_an_independent_upright_render() {
    use smithay::utils::Transform;
    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("output transform oracle") else {
        return;
    };
    let mut f = Fixture::with_config(super::config(&config_toml(true)));
    let output = f.add_output(1, (1200, 800));
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "transform-oracle",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    super::tick_until_settled(&mut f);
    let window = super::window_by_app_id(&mut f, "transform-oracle").unwrap();
    f.state().stage.set_position(&window, (100, 120).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, client, &surface, (180, 51), [255, 0, 0, 255]);
    for transform in [
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ] {
        output.change_current_state(None, Some(transform), None, None);
        let mode = output.current_mode().unwrap().size;
        let size = transform.transform_size(mode);
        let mut backend = f.state().backend.take().unwrap();
        let renderer = backend.renderer();
        let elements = crate::render::compose_frame(f.state(), renderer, &output, Vec::new());
        let refs: Vec<_> = elements.iter().collect();
        let raw = crate::render::render_elements_to_rgba_transformed(
            renderer,
            mode,
            Scale::from(1.0),
            transform,
            &refs,
        )
        .unwrap();
        let expected =
            crate::render::render_elements_to_rgba(renderer, size, Scale::from(1.0), &refs)
                .unwrap();
        let upright = upright_scanout(
            &raw,
            mode.w as usize,
            size.w as usize,
            size.h as usize,
            transform,
        );
        assert!(expected.as_chunks::<4>().0.contains(&[255, 0, 0, 255]));
        assert!(expected.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
        let delta = upright
            .iter()
            .zip(&expected)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(
            delta <= 2,
            "{transform:?}: GPU transform differs from pixel permutation by {delta}"
        );
        f.state().backend = Some(backend);
    }
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[derive(Clone, Copy, Debug)]
enum CaptureAllocationSite {
    ParentContent,
    ParentComposite,
    Popup,
    ResizeContent,
    ResizeClip,
}

#[derive(Clone, Copy, Debug)]
enum CaptureFailureOutcome {
    Reply,
    Timeout,
    Exit,
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn failed_tiling_capture_allocations_use_bounded_fallback_and_recover() {
    let _gl = gl::lock();
    let (ready, loaded) = std::sync::mpsc::sync_channel(1);
    driftline::text::warm_fonts(move || {
        let _ = ready.send(());
    });
    loaded
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for site in [
                CaptureAllocationSite::ParentContent,
                CaptureAllocationSite::ParentComposite,
                CaptureAllocationSite::Popup,
                CaptureAllocationSite::ResizeContent,
                CaptureAllocationSite::ResizeClip,
            ] {
                for outcome in [
                    CaptureFailureOutcome::Reply,
                    CaptureFailureOutcome::Timeout,
                    CaptureFailureOutcome::Exit,
                ] {
                    check_failed_tiling_capture(scale, ssd, site, outcome);
                }
            }
        }
    }
}

fn check_failed_tiling_capture(
    scale: f64,
    ssd: bool,
    site: CaptureAllocationSite,
    outcome: CaptureFailureOutcome,
) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use crate::render::capture_fault::CaptureAllocationFailure;
    use driftline::config::Action;
    use std::time::{Duration, Instant};
    let Some(renderer) = gl::surfaceless_renderer("tiling allocation failure") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"allocation\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "allocation",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "allocation").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    f.state().stage.set_position(&window, (100, 120).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, client, &surface, (180, 51), [255, 0, 0, 255]);
    let before = tiling_live_frame(&mut f, &output);
    {
        let failure = CaptureAllocationFailure::after(site as usize);
        f.state().execute_action(&Action::ToggleTiling);
        assert!(
            failure.triggered(),
            "allocation {site:?} must reach the capture allocator"
        );
    }
    assert!(f.state().tiling_view.is_some());
    assert_eq!(
        f.state().tiling_snapshot_count(),
        if matches!(
            site,
            CaptureAllocationSite::ParentContent
                | CaptureAllocationSite::ParentComposite
                | CaptureAllocationSite::Popup
        ) {
            0
        } else {
            2
        },
        "failed parent/popup capture must not install a partial tree"
    );
    let held = tiling_live_frame(&mut f, &output);
    let delta = before
        .iter()
        .zip(&held)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x SSD={ssd} allocation={site:?} {outcome:?}: fallback entry differs by {delta}"
    );
    match outcome {
        CaptureFailureOutcome::Reply => {
            let size = last_configured(&mut f, client, &surface);
            let w = f.client(client).window(&surface);
            w.set_size(size.0 as u16, size.1 as u16);
            w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
            w.ack_last_and_commit();
            f.double_roundtrip(client);
            assert_eq!(window.geometry().size, size.into());
        }
        CaptureFailureOutcome::Timeout => {
            let expired = Instant::now() + Duration::from_secs(1);
            f.state().refresh_tiling_at(expired);
            f.pump(4);
            f.state().refresh_tiling_at(expired);
        }
        CaptureFailureOutcome::Exit => f.state().execute_action(&Action::ToggleTiling),
    }
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    if matches!(outcome, CaptureFailureOutcome::Exit) {
        assert!(f.state().tiling_view.is_none());
        assert_eq!(f.state().tiling_pending_configures(), 0);
        f.state().execute_action(&Action::ToggleTiling);
    } else {
        tick_until_settled(&mut f);
        f.state().execute_action(&Action::CycleColumnWidth);
    }
    assert_eq!(
        f.state().tiling_snapshot_count(),
        2,
        "successful capture must recover the complete parent/popup tree"
    );
    let recovered_hold = tiling_live_frame(&mut f, &output);
    let size = last_configured(&mut f, client, &surface);
    let w = f.client(client).window(&surface);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert!(
        f.state().resize_crossfades.contains_key(&id),
        "recovery must restore real content retention"
    );
    let recovered_release = tiling_live_frame(&mut f, &output);
    let delta = recovered_hold
        .iter()
        .zip(&recovered_release)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x SSD={ssd} {site:?} {outcome:?}: recovered capture release differs by {delta}; differing pixels: {:?}",
        recovered_hold
            .as_chunks::<4>()
            .0
            .iter()
            .zip(recovered_release.as_chunks::<4>().0)
            .enumerate()
            .filter(|(_, (a, b))| a.iter().zip(b.iter()).any(|(a, b)| a.abs_diff(*b) > 2))
            .take(12)
            .collect::<Vec<_>>()
    );
    tick_until_settled(&mut f);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    assert!(!f.state().window_animations.is_active());
    let settled = tiling_live_frame(&mut f, &output);
    assert!(settled.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    assert!(settled.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    let w = f.client(client).window(&surface);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
    w.commit();
    f.double_roundtrip(client);
    assert!(
        !f.state().window_animations.is_active(),
        "same-size repaint must not revive a failed transition"
    );
    f.state().execute_action(&Action::ToggleTiling);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn remapping_a_held_or_mixed_tiling_window_clears_retention_and_keeps_placement() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_remapped_tiling_capture(scale, ssd, mixed);
            }
        }
    }
}

fn check_remapped_tiling_capture(scale: f64, ssd: bool, mixed: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id, window_position};
    use driftline::config::Action;
    let Some(renderer) = gl::surfaceless_renderer("remap tiling capture") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"remap\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "remap",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "remap").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    f.state().stage.set_position(&window, (100, 120).into());
    f.state().set_camera((0.0, 0.0).into());
    colored_tiling_popup(&mut f, client, &surface, (180, 51), [255, 0, 0, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    if mixed {
        let size = last_configured(&mut f, client, &surface);
        let w = f.client(client).window(&surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        assert!(f.state().resize_crossfades.contains_key(&id));
        f.state().execute_action(&Action::CycleColumnWidth);
    }
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    let old_offer = last_configured(&mut f, client, &surface);
    let destination = (210, 170).into();
    f.state().map_window(window.clone(), destination, true);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    assert!(!f.state().window_animations.is_active());
    assert_eq!(f.state().stage.id_of(&window), Some(id));
    assert_eq!(window_position(&mut f, &window), destination);
    // An already produced tiling buffer can arrive after the remap cancelled
    // its group; it must not restore the old placement or retained picture.
    let w = f.client(client).window(&surface);
    w.set_size(old_offer.0 as u16, old_offer.1 as u16);
    w.attach_shm_buffer(
        old_offer,
        &solid_bgra(old_offer.0, old_offer.1, [0, 255, 0, 255], false),
    );
    w.commit();
    f.double_roundtrip(client);
    assert_eq!(
        window_position(&mut f, &window),
        destination,
        "{scale}x SSD={ssd} mixed={mixed}: late tiling buffer moved remapped window"
    );
    let size = last_configured(&mut f, client, &surface);
    assert_eq!(size, (200, 100));
    let w = f.client(client).window(&surface);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    tick_until_settled(&mut f);
    assert_eq!(
        window_position(&mut f, &window),
        destination,
        "{scale}x SSD={ssd} mixed={mixed}: restore reply moved remapped window"
    );
    assert!(!f.state().resize_crossfades.contains_key(&id));
    assert!(!f.state().window_animations.is_active());
    let settled = tiling_live_frame(&mut f, &output);
    assert!(settled.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    assert!(settled.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn disconnecting_a_held_or_mixed_tiling_peer_releases_the_survivor() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_disconnected_tiling_peer(scale, ssd, mixed);
            }
        }
    }
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn disconnecting_an_output_keeps_the_survivor_tiling_snapshot() {
    use super::{last_configured, tick_until_settled};

    let _gl = gl::lock();
    let Some(renderer) = gl::surfaceless_renderer("disconnect tiling output snapshot") else {
        return;
    };
    let mut f = Fixture::with_config(super::config(&config_toml(true)));
    let survivor = f.add_output(1, (1200, 800));
    let removed = f.add_output(2, (1920, 1080));
    crate::state::output_state(&removed).camera = (5000.0, 5000.0).into();
    gl::install(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let survivor_surface = map_window_shm(
        &mut f,
        first,
        "snapshot-survivor",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [0, 255, 0, 255], false),
    );
    f.state().focused_output = Some(removed.clone());
    let removed_surface = map_window_shm(
        &mut f,
        second,
        "snapshot-removed",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [255, 0, 255, 255], false),
    );
    tick_until_settled(&mut f);
    f.state()
        .execute_action(&driftline::config::Action::ToggleTiling);

    let survivor_held = tiling_live_frame(&mut f, &survivor);
    let _removed_held = tiling_live_frame(&mut f, &removed);
    assert_eq!(f.state().tiling_snapshot_count(), 2);

    f.remove_output(&removed);

    assert!(f.state().tiling_snapshot_count() >= 1);
    assert_eq!(
        tiling_live_frame(&mut f, &survivor),
        survivor_held,
        "disconnecting the other output leaves this output's retained picture intact"
    );

    for (client, surface, color) in [
        (first, &survivor_surface, [0, 255, 0, 255]),
        (second, &removed_surface, [255, 0, 255, 255]),
    ] {
        let size = last_configured(&mut f, client, surface);
        let window = f.client(client).window(surface);
        window.set_size(size.0 as u16, size.1 as u16);
        window.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
        window.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    f.double_roundtrip(first);
    tick_until_settled(&mut f);
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    let settled = tiling_live_frame(&mut f, &survivor);
    assert!(settled.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    f.state()
        .execute_action(&driftline::config::Action::ToggleTiling);
    f.state().render.remove_output(&survivor.name());
    gl::uninstall(&mut f);
}

fn check_disconnected_tiling_peer(scale: f64, ssd: bool, mixed: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::{Action, Direction};
    let Some(renderer) = gl::surfaceless_renderer("disconnect tiling peer") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"*\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window_shm(
        &mut f,
        first,
        "survivor",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    let b = map_window_shm(
        &mut f,
        second,
        "departing",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [255, 0, 255, 255], false),
    );
    tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "survivor").unwrap();
    let wb = window_by_app_id(&mut f, "departing").unwrap();
    let aid = f.state().stage.id_of(&wa).unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    colored_tiling_popup(&mut f, first, &a, (180, 51), [255, 0, 0, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    if mixed {
        for (client, surface, color) in [
            (first, &a, [0, 255, 0, 255]),
            (second, &b, [255, 0, 255, 255]),
        ] {
            let size = last_configured(&mut f, client, surface);
            let w = f.client(client).window(surface);
            w.set_size(size.0 as u16, size.1 as u16);
            w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
            w.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        assert!(f.state().resize_crossfades.contains_key(&aid));
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    }
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    let before = wa.geometry().size;
    let held = tiling_live_frame(&mut f, &output);
    let w = f.client(first).window(&a);
    w.set_size(target.0 as u16, target.1 as u16);
    w.attach_shm_buffer(
        target,
        &solid_bgra(target.0, target.1, [0, 255, 0, 255], false),
    );
    w.ack_last_and_commit();
    f.double_roundtrip(first);
    assert_eq!(
        wa.geometry().size,
        before,
        "ready peer must wait for the group"
    );
    assert_eq!(f.state().tiling_snapshot_count(), 3);
    assert_eq!(tiling_live_frame(&mut f, &output), held);
    f.kill_client(second);
    f.double_roundtrip(first);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert!(f.state().stage.window_by_id(bid).is_none());
    assert!(!f.state().resize_crossfades.contains_key(&bid));
    assert_eq!(f.state().stage.id_of(&wa), Some(aid));
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(aid));
    // Removing a stacked peer can issue a fresh full-height offer. Answer it
    // before checking convergence rather than waiting for the group deadline.
    let size = last_configured(&mut f, first, &a);
    let w = f.client(first).window(&a);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
    w.ack_last_and_commit();
    f.double_roundtrip(first);
    tick_until_settled(&mut f);
    f.state().refresh_window_outputs();
    f.double_roundtrip(first);
    let client = f.client(first);
    let memberships = &client.state.surface_outputs[&a];
    assert_eq!(memberships.len(), 1);
    assert_eq!(client.state.outputs[&memberships[0]], output.name());
    let frame = tiling_live_frame(&mut f, &output);
    assert!(frame.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    assert!(frame.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    assert!(!frame.as_chunks::<4>().0.contains(&[255, 0, 255, 255]));
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&aid));
    assert!(!f.state().window_animations.is_active());
    let w = f.client(first).window(&a);
    w.commit();
    f.double_roundtrip(first);
    assert!(!f.state().window_animations.is_active());
    assert_eq!(tiling_live_frame(&mut f, &output), frame);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn protocol_remap_during_held_or_mixed_tiling_resize_reenrolls_cleanly() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_protocol_tiling_remap(scale, ssd, mixed);
            }
        }
    }
}

fn check_protocol_tiling_remap(scale: f64, ssd: bool, mixed: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::Action;
    let Some(renderer) = gl::surfaceless_renderer("protocol tiling remap") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!(
            "{config}\n[[window_rules]]\napp_id = \"protocol-remap\"\ndecoration = \"server\"\n"
        )
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let empty = tiling_live_frame(&mut f, &output);
    let client = f.add_client();
    let surface = map_window_shm(
        &mut f,
        client,
        "protocol-remap",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    tick_until_settled(&mut f);
    let window = window_by_app_id(&mut f, "protocol-remap").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    let popup = colored_tiling_popup(&mut f, client, &surface, (180, 51), [255, 0, 255, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    if mixed {
        let size = last_configured(&mut f, client, &surface);
        let w = f.client(client).window(&surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [0, 255, 0, 255], false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        assert!(f.state().resize_crossfades.contains_key(&id));
        f.state().execute_action(&Action::CycleColumnWidth);
    }
    assert_eq!(f.state().tiling_snapshot_count(), 2);
    f.client(client).popup(&popup).destroy();
    let w = f.client(client).window(&surface);
    w.attach_null();
    w.commit();
    f.double_roundtrip(client);
    assert!(
        f.state().tiling_view.is_some(),
        "unmap must retain the empty layout"
    );
    assert!(
        f.state().stage.tiled_columns().is_empty(),
        "unmapped window reserved a phantom column"
    );
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    assert!(!f.state().window_animations.is_active());
    assert!(f.state().focused_element().is_none());
    assert!(
        f.client(client)
            .state
            .surface_outputs
            .get(&surface)
            .is_none_or(Vec::is_empty)
    );
    let unmapped = tiling_live_frame(&mut f, &output);
    assert!(
        unmapped == empty,
        "{scale}x SSD={ssd} mixed={mixed}: unmap left pixels: {:?}",
        unmapped
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != [0, 0, 0, 0])
            .take(12)
            .collect::<Vec<_>>()
    );
    // The role reset discards metadata. A new empty commit begins the initial
    // configure handshake, and only its first buffer may occupy a column.
    let w = f.client(client).window(&surface);
    w.xdg_toplevel.set_app_id("protocol-remap".into());
    w.commit();
    f.double_roundtrip(client);
    let size = last_configured(&mut f, client, &surface);
    assert!(size.0 > 200 && size.1 > 100);
    assert!(f.state().stage.tiled_columns().is_empty());
    let w = f.client(client).window(&surface);
    w.set_size(size.0 as u16, size.1 as u16);
    w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, [255, 0, 0, 255], false));
    w.ack_last_and_commit();
    f.double_roundtrip(client);
    tick_until_settled(&mut f);
    assert_eq!(f.state().stage.id_of(&window), Some(id));
    assert_eq!(f.state().stage.active_tile(), Some(id));
    assert_eq!(f.state().window_ssd_bar(&window) > 0, ssd);
    f.state().refresh_window_outputs();
    f.double_roundtrip(client);
    let c = f.client(client);
    let memberships = &c.state.surface_outputs[&surface];
    assert_eq!(memberships.len(), 1);
    assert_eq!(c.state.outputs[&memberships[0]], output.name());
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(id));
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().resize_crossfades.contains_key(&id));
    let frame = tiling_live_frame(&mut f, &output);
    assert!(frame.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    assert!(!frame.as_chunks::<4>().0.contains(&[255, 0, 0, 255]));
    assert!(!frame.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    assert!(!frame.as_chunks::<4>().0.contains(&[255, 0, 255, 255]));
    f.client(client).window(&surface).commit();
    f.double_roundtrip(client);
    assert!(!f.state().window_animations.is_active());
    assert_eq!(tiling_live_frame(&mut f, &output), frame);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn superseding_held_or_mixed_tiling_groups_preserves_ready_and_pending_peers() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_superseded_tiling_peers(scale, ssd, mixed);
            }
        }
    }
}

fn check_superseded_tiling_peers(scale: f64, ssd: bool, mixed: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::{Action, Direction};
    let Some(renderer) = gl::surfaceless_renderer("supersede tiling peers") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"*\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window_shm(
        &mut f,
        first,
        "survivor",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    let b = map_window_shm(
        &mut f,
        second,
        "departing",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [255, 0, 255, 255], false),
    );
    tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "survivor").unwrap();
    let wb = window_by_app_id(&mut f, "departing").unwrap();
    let aid = f.state().stage.id_of(&wa).unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    colored_tiling_popup(&mut f, first, &a, (180, 51), [255, 0, 0, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    if mixed {
        for (client, surface, color) in [
            (first, &a, [0, 255, 0, 255]),
            (second, &b, [255, 0, 255, 255]),
        ] {
            let size = last_configured(&mut f, client, surface);
            let w = f.client(client).window(surface);
            w.set_size(size.0 as u16, size.1 as u16);
            w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
            w.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        assert!(f.state().resize_crossfades.contains_key(&aid));
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    }
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    let before = wa.geometry().size;
    let held = tiling_live_frame(&mut f, &output);
    let w = f.client(first).window(&a);
    w.set_size(target.0 as u16, target.1 as u16);
    w.attach_shm_buffer(
        target,
        &solid_bgra(target.0, target.1, [0, 255, 0, 255], false),
    );
    w.ack_last_and_commit();
    f.double_roundtrip(first);
    assert_eq!(
        wa.geometry().size,
        before,
        "ready peer must wait for the group"
    );
    assert_eq!(f.state().tiling_snapshot_count(), 3);
    assert_eq!(tiling_live_frame(&mut f, &output), held);
    f.state().execute_action(&Action::CycleColumnWidth);
    let next_a = last_configured(&mut f, first, &a);
    let next_b = last_configured(&mut f, second, &b);
    f.double_roundtrip(first);
    assert_eq!(
        wa.geometry().size,
        target.into(),
        "supersession must release the queued old buffer"
    );
    assert!(f.state().tiling_pending_configures() > 0);
    assert_eq!(f.state().tiling_snapshot_count(), 3);
    assert!(
        tiling_live_frame(&mut f, &output) == held,
        "{scale}x SSD={ssd} mixed={mixed}: supersession changed the held image"
    );
    if mixed {
        assert_ne!(next_a, target);
        let w = f.client(first).window(&a);
        w.set_size(next_a.0 as u16, next_a.1 as u16);
        w.attach_shm_buffer(
            next_a,
            &solid_bgra(next_a.0, next_a.1, [255, 255, 0, 255], false),
        );
        w.ack_last_and_commit();
        f.double_roundtrip(first);
        assert_eq!(
            wa.geometry().size,
            target.into(),
            "new ready buffer must still wait for the pending peer"
        );
        assert!(tiling_live_frame(&mut f, &output) == held);
    }
    let w = f.client(second).window(&b);
    w.set_size(next_b.0 as u16, next_b.1 as u16);
    w.attach_shm_buffer(
        next_b,
        &solid_bgra(next_b.0, next_b.1, [0, 255, 255, 255], false),
    );
    w.ack_last_and_commit();
    f.double_roundtrip(second);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, next_a.into());
    assert_eq!(wb.geometry().size, next_b.into());
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    let released = tiling_live_frame(&mut f, &output);
    let delta = released
        .iter()
        .zip(&held)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(
        delta <= 2,
        "{scale}x SSD={ssd} mixed={mixed}: superseded release differs by {delta}"
    );
    tick_until_settled(&mut f);
    assert!(!f.state().resize_crossfades.contains_key(&aid));
    assert!(!f.state().resize_crossfades.contains_key(&bid));
    assert_eq!(f.state().stage.id_of(&wa), Some(aid));
    assert_eq!(f.state().stage.id_of(&wb), Some(bid));
    let expected_focus = if mixed { aid } else { bid };
    assert_eq!(f.state().stage.active_tile(), Some(expected_focus));
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(expected_focus));
    f.state().refresh_window_outputs();
    for (client, surface) in [(first, &a), (second, &b)] {
        f.double_roundtrip(client);
        let c = f.client(client);
        let memberships = &c.state.surface_outputs[surface];
        assert_eq!(memberships.len(), 1);
        assert_eq!(c.state.outputs[&memberships[0]], output.name());
    }
    let settled = tiling_live_frame(&mut f, &output);
    let expected_a = if mixed {
        [0, 255, 255, 255]
    } else {
        [0, 255, 0, 255]
    };
    assert!(settled.as_chunks::<4>().0.contains(&expected_a));
    assert!(settled.as_chunks::<4>().0.contains(&[255, 255, 0, 255]));
    assert!(!settled.as_chunks::<4>().0.contains(&[255, 0, 255, 255]));
    if mixed {
        assert!(!settled.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    }
    assert!(settled.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    for (client, surface) in [(first, &a), (second, &b)] {
        f.client(client).window(surface).commit();
        f.double_roundtrip(client);
    }
    assert!(!f.state().window_animations.is_active());
    assert!(tiling_live_frame(&mut f, &output) == settled);
    f.state().render.remove_output(&output.name());
    gl::uninstall(&mut f);
}

#[test]
#[ignore = "needs surfaceless EGL; run with --include-ignored"]
fn removing_the_only_output_cancels_held_or_mixed_tiling_and_recovers() {
    let _gl = gl::lock();
    for scale in [1.0, 1.25, 1.5] {
        for ssd in [false, true] {
            for mixed in [false, true] {
                check_removed_tiling_output(scale, ssd, mixed);
            }
        }
    }
}

fn check_removed_tiling_output(scale: f64, ssd: bool, mixed: bool) {
    use super::{last_configured, tick_until_settled, window_by_app_id};
    use driftline::config::{Action, Direction};
    let Some(renderer) = gl::surfaceless_renderer("remove tiling output") else {
        return;
    };
    let config = config_toml(true).replace("corner_radius = 0", "corner_radius = 8");
    let config = if ssd {
        format!("{config}\n[[window_rules]]\napp_id = \"*\"\ndecoration = \"server\"\n")
    } else {
        config
    };
    let mut f = Fixture::with_config(super::config(&config));
    let output = f.add_output_scaled(1, (1200, 800), scale);
    gl::install(&mut f, renderer);
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window_shm(
        &mut f,
        first,
        "survivor",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, CONTENT_BGRA, false),
    );
    let b = map_window_shm(
        &mut f,
        second,
        "departing",
        (200, 100),
        (200, 100),
        &solid_bgra(200, 100, [255, 0, 255, 255], false),
    );
    tick_until_settled(&mut f);
    let wa = window_by_app_id(&mut f, "survivor").unwrap();
    let wb = window_by_app_id(&mut f, "departing").unwrap();
    let aid = f.state().stage.id_of(&wa).unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    colored_tiling_popup(&mut f, first, &a, (180, 51), [255, 0, 0, 255]);
    tiling_live_frame(&mut f, &output);
    f.state().execute_action(&Action::ToggleTiling);
    if mixed {
        for (client, surface, color) in [
            (first, &a, [0, 255, 0, 255]),
            (second, &b, [255, 0, 255, 255]),
        ] {
            let size = last_configured(&mut f, client, surface);
            let w = f.client(client).window(surface);
            w.set_size(size.0 as u16, size.1 as u16);
            w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
            w.ack_last_and_commit();
            f.double_roundtrip(client);
        }
        f.double_roundtrip(first);
        f.state()
            .tick_window_animations(std::time::Duration::from_millis(16));
        tiling_live_frame(&mut f, &output);
        assert!(f.state().resize_crossfades.contains_key(&aid));
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    }
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    let before = wa.geometry().size;
    let held = tiling_live_frame(&mut f, &output);
    let w = f.client(first).window(&a);
    w.set_size(target.0 as u16, target.1 as u16);
    w.attach_shm_buffer(
        target,
        &solid_bgra(target.0, target.1, [0, 255, 0, 255], false),
    );
    w.ack_last_and_commit();
    f.double_roundtrip(first);
    assert_eq!(
        wa.geometry().size,
        before,
        "ready peer must wait for the group"
    );
    assert_eq!(f.state().tiling_snapshot_count(), 3);
    assert_eq!(tiling_live_frame(&mut f, &output), held);
    let old_layout = f.state().stage.tiling_layout_id();
    f.remove_output(&output);
    f.double_roundtrip(first);
    f.double_roundtrip(second);
    assert!(
        f.state().tiling_view.is_none(),
        "disconnected placeholder kept the layout alive"
    );
    assert_eq!(
        wa.geometry().size,
        target.into(),
        "disconnect must release the queued buffer"
    );
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().window_animations.is_active());
    assert!(!f.state().resize_crossfades.contains_key(&aid));
    assert!(!f.state().resize_crossfades.contains_key(&bid));
    assert!(
        f.state()
            .render
            .animated_content
            .keys()
            .all(|(name, _)| name != &output.name())
    );
    assert!(f.state().disconnected_outputs.contains(&output.name()));
    for (client, surface) in [(first, &a), (second, &b)] {
        assert!(
            f.client(client)
                .state
                .surface_outputs
                .get(surface)
                .is_none_or(Vec::is_empty)
        );
    }
    f.state().execute_action(&Action::ToggleTiling);
    assert!(
        f.state().tiling_view.is_none(),
        "headless placeholder admitted tiling"
    );
    for (client, surface, color) in [
        (first, &a, [0, 255, 0, 255]),
        (second, &b, [0, 255, 255, 255]),
    ] {
        let size = last_configured(&mut f, client, surface);
        assert_eq!(size, (200, 100));
        let w = f.client(client).window(surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    tick_until_settled(&mut f);
    let reconnected = f.add_output_scaled(1, (1200, 800), scale);
    assert_ne!(output, reconnected);
    assert!(f.state().disconnected_outputs.is_empty());
    assert!(f.state().tiling_view.is_none());
    assert_eq!(f.state().stage.id_of(&wa), Some(aid));
    assert_eq!(f.state().stage.id_of(&wb), Some(bid));
    f.state().execute_action(&Action::ToggleTiling);
    assert_ne!(f.state().stage.tiling_layout_id(), old_layout);
    for (client, surface, color) in [
        (first, &a, [0, 255, 0, 255]),
        (second, &b, [0, 255, 255, 255]),
    ] {
        let size = last_configured(&mut f, client, surface);
        let w = f.client(client).window(surface);
        w.set_size(size.0 as u16, size.1 as u16);
        w.attach_shm_buffer(size, &solid_bgra(size.0, size.1, color, false));
        w.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    f.double_roundtrip(first);
    tick_until_settled(&mut f);
    f.state().refresh_window_outputs();
    for (client, surface) in [(first, &a), (second, &b)] {
        f.double_roundtrip(client);
        let c = f.client(client);
        let memberships = &c.state.surface_outputs[surface];
        assert_eq!(memberships.len(), 1);
        assert_eq!(c.state.outputs[&memberships[0]], reconnected.name());
    }
    assert_eq!(f.state().tiling_snapshot_count(), 0);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(!f.state().window_animations.is_active());
    let focused = f.state().focused_element().unwrap();
    let focused_id = f.state().stage.id_of(&focused);
    assert_eq!(f.state().stage.active_tile(), focused_id);
    let frame = tiling_live_frame(&mut f, &reconnected);
    assert!(frame.as_chunks::<4>().0.contains(&[0, 255, 0, 255]));
    assert!(frame.as_chunks::<4>().0.contains(&[255, 255, 0, 255]));
    assert!(frame.as_chunks::<4>().0.contains(&[0, 0, 255, 255]));
    assert!(!frame.as_chunks::<4>().0.contains(&[255, 0, 255, 255]));
    for (client, surface) in [(first, &a), (second, &b)] {
        f.client(client).window(surface).commit();
        f.double_roundtrip(client);
    }
    assert!(!f.state().window_animations.is_active());
    assert!(tiling_live_frame(&mut f, &reconnected) == frame);
    f.state().render.remove_output(&reconnected.name());
    gl::uninstall(&mut f);
}
