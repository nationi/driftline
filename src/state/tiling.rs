use smithay::output::Output;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Logical, Point, SERIAL_COUNTER};
use smithay::wayland::seat::WaylandFocus;

use driftline::config::Direction;
use driftline::layout::scrolling::{self, Column, Constraints, Tile};
use driftline::window_ext::WindowExt;

use super::{Driftline, StageWindow, configured_window_size, output_state, visual_frame_center};
use crate::grabs::SizeConstraints;

const HOVER_RETURN_TARGET: i32 = 48;

fn preserve_hover_target(
    target: Point<i32, Logical>,
    previous: smithay::utils::Rectangle<i32, Logical>,
    usable: smithay::utils::Rectangle<i32, Logical>,
) -> Point<i32, Logical> {
    let axis = |camera: i32, start: i32, size: i32, inset: i32, extent: i32| {
        let visible = HOVER_RETURN_TARGET.min(size.max(1)).min(extent.max(1));
        let min = i64::from(start) + i64::from(visible) - i64::from(inset) - i64::from(extent);
        let max = i64::from(start) + i64::from(size) - i64::from(visible) - i64::from(inset);
        i64::from(camera)
            .clamp(min, max)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    };
    Point::from((
        axis(
            target.x,
            previous.loc.x,
            previous.size.w,
            usable.loc.x,
            usable.size.w,
        ),
        axis(
            target.y,
            previous.loc.y,
            previous.size.h,
            usable.loc.y,
            usable.size.h,
        ),
    ))
}

pub(crate) struct TilingView {
    layout_id: driftline::stage::TilingLayoutId,
    output: Output,
    camera: Point<f64, Logical>,
    zoom: f64,
    origin: Point<i32, Logical>,
    last_usable: Option<smithay::utils::Rectangle<i32, Logical>>,
    anchor: Option<(driftline::stage::ColumnId, i32)>,
    close_return: Option<CloseReturnView>,
    configures: scrolling::transaction::ConfigureBatch,
    commits: Option<TilingCommits>,
    commit_wakeups: usize,
    presentation: Option<TilingPresentation>,
    snapshots: std::collections::HashMap<driftline::stage::ElementId, TilingSnapshot>,
    pending_close_return: Option<CloseReturnView>,
    selected: Option<driftline::stage::ElementId>,
    configure_timer: Option<smithay::reexports::calloop::RegistrationToken>,
    initial_configures:
        std::collections::HashMap<driftline::stage::ElementId, InitialTilingConfigure>,
    revealed: Option<(
        driftline::stage::ElementId,
        smithay::utils::Rectangle<i32, Logical>,
    )>,
    hover_camera_guard: Option<(driftline::stage::ElementId, driftline::stage::ElementId)>,
}

#[derive(Default)]
pub(crate) struct TilingViews {
    views: std::collections::HashMap<String, TilingView>,
    active_output: Option<String>,
}

impl TilingViews {
    pub(crate) fn insert(&mut self, view: TilingView) {
        let output = view.output.name();
        self.views.insert(output.clone(), view);
        self.active_output = Some(output);
    }

    fn insert_preserving_selection(&mut self, view: TilingView) {
        let active_output = self.active_output.clone();
        self.insert(view);
        if active_output.is_some() {
            self.active_output = active_output;
        }
    }

    pub(crate) fn select_output(&mut self, output: &Output) -> bool {
        self.select_output_name(&output.name())
    }

    pub(crate) fn select_output_name(&mut self, name: &str) -> bool {
        if !self.views.contains_key(name) {
            return false;
        }
        self.active_output = Some(name.to_owned());
        true
    }

    pub(crate) fn for_output(&self, output: &Output) -> Option<&TilingView> {
        self.views.get(&output.name())
    }

    fn iter(&self) -> impl Iterator<Item = (&str, &TilingView)> {
        self.views
            .iter()
            .map(|(output, view)| (output.as_str(), view))
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut TilingView)> {
        self.views
            .iter_mut()
            .map(|(output, view)| (output.as_str(), view))
    }

    pub(crate) fn as_ref(&self) -> Option<&TilingView> {
        self.views.get(self.active_output.as_ref()?)
    }

    pub(crate) fn as_mut(&mut self) -> Option<&mut TilingView> {
        self.views.get_mut(self.active_output.as_ref()?)
    }

    fn take_all(&mut self) -> Vec<TilingView> {
        self.active_output = None;
        self.views.drain().map(|(_, view)| view).collect()
    }

    fn take_output(&mut self, output: &str) -> Option<TilingView> {
        let view = self.views.remove(output);
        if self.active_output.as_deref() == Some(output) {
            self.active_output = None;
        }
        view
    }

    pub(crate) fn is_some(&self) -> bool {
        !self.views.is_empty()
    }

    pub(crate) fn is_none(&self) -> bool {
        self.views.is_empty()
    }
}

struct TilingSnapshot {
    image: crate::render::ClosingSnapshot,
    held_geometry: Option<smithay::utils::Rectangle<f64, Logical>>,
    resize: Option<crate::render::ResizeCrossfade>,
    mask: Option<crate::render::BlurMaskSource>,
    popups: Vec<TilingPopupSnapshot>,
    blur: crate::render::SnapshotBlur,
    mask_policy: Option<crate::render::SnapshotBlur>,
    shadow: Option<crate::render::SnapshotShadow>,
    border: Option<crate::render::SnapshotBorder>,
    opacity: f64,
    native_chrome: bool,
    bar: Option<crate::render::TitlePixels>,
}

pub(crate) struct TilingSnapshotElements {
    pub bar: Option<TilingSnapshotBar>,
    pub chrome_animation: Option<crate::render::WindowRenderAnimation>,
    pub popup_animation: Option<crate::render::WindowRenderAnimation>,
    pub live_popups: Option<TilingLivePopups>,
    pub elements: Vec<crate::render::OutputRenderElements>,
    pub blur: Option<crate::render::BlurRequestData>,
    pub shadow: Option<crate::render::SnapshotShadow>,
    pub border: Option<crate::render::SnapshotBorder>,
}

pub(crate) struct TilingSnapshotBar {
    pub buffer: crate::render::TitlePixels,
    pub rect: smithay::utils::Rectangle<f64, Logical>,
    pub opacity: f64,
}

pub(crate) struct TilingLivePopups {
    pub root: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    pub origin: Point<f64, Logical>,
    pub geometry_loc: Point<i32, Logical>,
    pub opacity: f64,
}

fn popup_stamp(
    surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    geometry: smithay::utils::Rectangle<i32, Logical>,
) -> crate::render::ResizeContentStamp {
    crate::render::ResizeContentStamp::for_surface(
        surface,
        geometry,
        1.0,
        crate::render::BakeChrome {
            bare: true,
            corner_radius: [0.0; 4],
        },
    )
}

impl TilingSnapshot {
    fn matches_popups(
        &self,
        root: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> bool {
        let mut saved = self.popups.iter();
        for (popup, offset) in mapped_tiling_popups(root) {
            let Some(previous) = saved.next() else {
                return false;
            };
            if popup.wl_surface() != &previous.surface
                || offset != previous.offset
                || popup.geometry() != previous.geometry
                || popup_stamp(popup.wl_surface(), popup.geometry()) != previous.stamp
            {
                return false;
            }
        }
        saved.next().is_none()
    }
}

struct TilingPopupSnapshot {
    surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    offset: Point<i32, Logical>,
    geometry: smithay::utils::Rectangle<i32, Logical>,
    image: crate::render::ClosingSnapshot,
    stamp: crate::render::ResizeContentStamp,
}

struct TilingPresentation {
    requests: std::collections::HashMap<driftline::stage::ElementId, smithay::utils::Serial>,
    deadline: std::time::Instant,
}

struct TilingCommits {
    group: scrolling::commit_group::CommitGroup,
    generation: scrolling::transaction::LayoutGeneration,
    clients: Vec<smithay::reexports::wayland_server::Client>,
}

struct InitialTilingConfigure {
    serial: smithay::utils::Serial,
    size: smithay::utils::Size<i32, Logical>,
    sent_at: std::time::Instant,
    expired: bool,
}

#[derive(Clone, Copy)]
struct CloseReturnView {
    opened: driftline::stage::ColumnId,
    previous: driftline::stage::ColumnId,
    offset: f64,
}

fn mapped_tiling_popups(
    surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
) -> impl Iterator<Item = (smithay::desktop::PopupKind, Point<i32, Logical>)> {
    smithay::desktop::PopupManager::popups_for_surface(surface).filter(|(popup, _)| {
        smithay::wayland::compositor::with_states(popup.wl_surface(), |states| {
            states
                .data_map
                .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>()
                .is_some_and(|data| data.lock().unwrap().buffer().is_some())
        })
    })
}

impl Driftline {
    fn show_tiling_mode_cue(&mut self, outputs: &[Output]) {
        let speed = self.config.effects.animation_speed;
        if speed >= 1.0 {
            self.tiling_mode_cues.clear();
            return;
        }
        let duration = std::time::Duration::from_millis((420.0 + (1.0 - speed) * 360.0) as u64);
        self.tiling_mode_cues.clear();
        for output in outputs {
            self.tiling_mode_cues.insert(
                output.name(),
                super::TilingModeCue {
                    elapsed: std::time::Duration::ZERO,
                    duration,
                },
            );
            self.redraws_needed.insert(output.clone());
        }
    }

    pub(crate) fn move_tiled_window_to_output(
        &mut self,
        id: driftline::stage::ElementId,
        target: &Output,
    ) -> bool {
        let Some(source_name) = self.stage.tiling_output_for_window(id).map(str::to_owned) else {
            return false;
        };
        if source_name == target.name() || !self.tiling_view.is_some() {
            return false;
        }
        let Some(source) = self.output_by_name(&source_name) else {
            return false;
        };

        let mut wake_clients = Vec::new();
        for output_name in [source_name.as_str(), target.name().as_str()] {
            self.tiling_view.select_output_name(output_name);
            let (commits, timer) = self.tiling_view.as_mut().map_or((None, None), |view| {
                let commits = view.commits.take();
                let timer = view.configure_timer.take();
                view.presentation = None;
                view.snapshots.remove(&id);
                view.initial_configures.remove(&id);
                (commits, timer)
            });
            if let Some(mut commits) = commits {
                commits.group.cancel();
                wake_clients.extend(commits.clients);
            }
            if let Some(timer) = timer {
                self.loop_handle.remove(timer);
            }
        }
        if !wake_clients.is_empty() {
            self.wake_tiling_clients(wake_clients);
        }
        if !self.stage.move_tiled_window_to_output(id, &target.name()) {
            return false;
        }
        if let Some(element) = self.stage.window_by_id(id).cloned() {
            self.drop_owed_recenter(&element);
            if let Some(window) = element.client() {
                self.cancel_window_animation(window);
            }
        }
        self.drop_resize_crossfade(id);

        self.focused_output = Some(target.clone());
        self.tiling_view.select_output(target);
        self.stage.activate_tiling_output(&target.name());
        self.stage.select_tile(id);
        self.refresh_tiling_for_output(&source, std::time::Instant::now());
        self.refresh_tiling_for_output(target, std::time::Instant::now());
        if let Some(element) = self.stage.window_by_id(id).cloned() {
            self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
        }
        self.render.blur_geometry_generation += 1;
        self.mark_all_dirty();
        self.session_store_mark_dirty();
        true
    }

    pub(super) fn transfer_tiling_from_disconnected_output(&mut self, from: &Output, to: &Output) {
        let Some(mut view) = self.tiling_view.take_output(&from.name()) else {
            return;
        };
        if let Some(mut commits) = view.commits.take() {
            commits.group.cancel();
            self.wake_tiling_clients(commits.clients);
        }
        if let Some(timer) = view.configure_timer.take() {
            self.loop_handle.remove(timer);
        }
        let moved = self.stage.transfer_tiling_output(&from.name(), &to.name());
        for tile in moved {
            if let Some(element) = self.stage.window_by_id(tile.id).cloned() {
                self.drop_owed_recenter(&element);
                if let Some(window) = element.client() {
                    self.cancel_window_animation(window);
                }
            }
        }
        self.tiling_view.select_output(to);
        self.stage.activate_tiling_output(&to.name());
        self.refresh_tiling_for_output(to, std::time::Instant::now());
    }

    pub(super) fn output_connected_during_tiling(&mut self, output: &Output) {
        if self.tiling_view.is_none() {
            return;
        }
        let (camera, zoom) = {
            let state = output_state(output);
            (state.camera, state.zoom)
        };
        let usable = self.usable_area_on(output);
        let camera_origin: Point<i32, Logical> = camera.to_i32_round();
        let (Ok(x), Ok(y)) = (
            i32::try_from(i64::from(camera_origin.x) + i64::from(usable.loc.x)),
            i32::try_from(i64::from(camera_origin.y) + i64::from(usable.loc.y)),
        ) else {
            return;
        };
        let active_output = self.stage.active_tiling_output().to_owned();
        let layout_id = self.stage.begin_tiling_for_output(&output.name());
        self.tiling_view.insert_preserving_selection(TilingView {
            layout_id,
            output: output.clone(),
            camera,
            zoom,
            origin: Point::from((x, y)),
            last_usable: None,
            anchor: None,
            close_return: None,
            configures: scrolling::transaction::ConfigureBatch::new(layout_id),
            configure_timer: None,
            commits: None,
            commit_wakeups: 0,
            presentation: None,
            snapshots: Default::default(),
            pending_close_return: None,
            selected: None,
            initial_configures: Default::default(),
            revealed: None,
            hover_camera_guard: None,
        });
        let mut state = output_state(output);
        state.zoom = 1.0;
        state.zoom_target = None;
        self.stage.activate_tiling_output(&active_output);
    }

    pub(crate) fn toggle_tiling(&mut self) {
        if self.tiling_view.is_some() {
            self.exit_tiling();
            return;
        }
        let Some(active_output) = self.active_output() else {
            return;
        };
        let outputs: Vec<_> = self
            .space
            .outputs()
            .filter(|output| !self.disconnected_outputs.contains(&output.name()))
            .cloned()
            .collect();
        if outputs.is_empty() || self.stage.has_fullscreen() || !self.interactive_move.is_empty() {
            tracing::warn!(
                "Scrolling tiling requires connected outputs and no active move or fullscreen"
            );
            return;
        }
        self.cancel_animations_everywhere();
        let mut view_seeds = Vec::with_capacity(outputs.len());
        for output in &outputs {
            let (camera, zoom) = {
                let state = output_state(output);
                (state.camera, state.zoom)
            };
            let usable = self.usable_area_on(output);
            let camera_origin: Point<i32, Logical> = camera.to_i32_round();
            let (Ok(x), Ok(y)) = (
                i32::try_from(i64::from(camera_origin.x) + i64::from(usable.loc.x)),
                i32::try_from(i64::from(camera_origin.y) + i64::from(usable.loc.y)),
            ) else {
                return;
            };
            view_seeds.push((output.clone(), camera, zoom, Point::from((x, y))));
        }
        let cue_outputs = outputs.clone();
        for (output, camera, zoom, origin) in view_seeds {
            let layout_id = self.stage.begin_tiling_for_output(&output.name());
            self.tiling_view.insert(TilingView {
                layout_id,
                output: output.clone(),
                camera,
                zoom,
                origin,
                last_usable: None,
                anchor: None,
                close_return: None,
                configures: scrolling::transaction::ConfigureBatch::new(layout_id),
                configure_timer: None,
                commits: None,
                commit_wakeups: 0,
                presentation: None,
                snapshots: Default::default(),
                pending_close_return: None,
                selected: None,
                initial_configures: Default::default(),
                revealed: None,
                hover_camera_guard: None,
            });
            let mut state = output_state(&output);
            state.zoom = 1.0;
            state.zoom_target = None;
        }

        // Resolve ownership once against the user's actual focused output.
        // Later per-output refreshes must not steal windows just because they
        // temporarily select a different output to compute its geometry.
        let existing: Vec<_> = self
            .stage
            .windows()
            .filter(|window| self.tiling_eligible(window))
            .filter_map(|window| {
                let id = self.stage.id_of(window)?;
                let remembered_output = self
                    .tiling_topology
                    .as_ref()
                    .and_then(|topology| self.stage.tiling_topology_output(topology, id))
                    .or_else(|| {
                        self.session_store
                            .durable_tiling
                            .get(&id)
                            .map(|tile| tile.output.as_str())
                    })
                    .filter(|name| outputs.iter().any(|output| output.name() == *name));
                let output = remembered_output
                    .and_then(|name| self.output_by_name(name))
                    .or_else(|| self.output_for_tiling_window(window))?;
                let size = window.client().map(configured_window_size)?;
                Some((output.name(), id, size))
            })
            .collect();
        for (output, id, size) in existing {
            self.stage.activate_tiling_output(&output);
            self.stage.tile(id, size);
        }
        if let Some(topology) = self.tiling_topology.as_ref() {
            self.stage.restore_tiling_topology(topology);
        } else {
            let placements: Vec<_> = self
                .session_store
                .durable_tiling
                .iter()
                .map(|(id, tile)| {
                    let width = match tile.width {
                        driftline::session::SessionColumnWidth::Fixed(width) => {
                            scrolling::ColumnWidth::Fixed(width)
                        }
                        driftline::session::SessionColumnWidth::Proportion(value) => {
                            scrolling::ColumnWidth::Proportion(value)
                        }
                    };
                    (
                        *id,
                        tile.output.clone(),
                        tile.column,
                        tile.stack,
                        width,
                        tile.selected,
                        tile.active,
                    )
                })
                .collect();
            self.stage.restore_tiling_placements(&placements);
        }
        self.update_output_from_camera();
        for output in outputs {
            self.refresh_tiling_for_output(&output, std::time::Instant::now());
        }
        self.focused_output = Some(active_output);
        self.stage
            .activate_tiling_output(&self.focused_output.as_ref().unwrap().name());
        self.show_tiling_mode_cue(&cue_outputs);
    }

    pub(super) fn exit_tiling(&mut self) {
        self.exit_tiling_with_animation(true);
    }

    pub(super) fn exit_tiling_with_animation(&mut self, animate: bool) {
        let mut tiled_screen_rects = std::collections::HashMap::new();
        if animate && self.config.effects.animation_speed < 1.0 {
            for tiled in self.stage.tiled_windows() {
                let Some(element) = self.stage.window_by_id(tiled.id) else {
                    continue;
                };
                let Some(output_name) = self.stage.tiling_output_for_window(tiled.id) else {
                    continue;
                };
                let Some(output) = self.output_by_name(output_name) else {
                    continue;
                };
                let Some(loc) = self.stage.position_of(element) else {
                    continue;
                };
                let visual = self.geometry_seed(tiled.id, loc, element.geometry().size);
                let (camera, zoom) = {
                    let os = output_state(&output);
                    (os.camera, os.zoom)
                };
                tiled_screen_rects.insert(
                    tiled.id,
                    smithay::utils::Rectangle::<f64, Logical>::new(
                        Point::from((
                            (visual.loc.x - camera.x) * zoom,
                            (visual.loc.y - camera.y) * zoom,
                        )),
                        smithay::utils::Size::from((visual.size.w * zoom, visual.size.h * zoom)),
                    ),
                );
            }
        }
        let saved: Vec<_> = self
            .stage
            .tiled_windows()
            .filter_map(|window| {
                self.current_tiling_tile(window.id)
                    .map(|placement| (window.id, placement))
            })
            .collect();
        self.session_store.durable_tiling.extend(saved);
        let views = self.tiling_view.take_all();
        if views.is_empty() {
            return;
        }
        self.tiling_topology = Some(self.stage.tiling_topology());
        self.cancel_animations_everywhere();
        for mut view in views {
            if let Some(mut commits) = view.commits.take() {
                commits.group.cancel();
                self.wake_tiling_clients(commits.clients);
            }
            if let Some(timer) = view.configure_timer.take() {
                self.loop_handle.remove(timer);
            }
            {
                let mut os = output_state(&view.output);
                os.camera = view.camera;
                os.zoom = view.zoom;
                os.camera_target = None;
                os.zoom_target = None;
            }
            for (id, _) in view.initial_configures {
                if !self.stage.is_tiled(id)
                    && let Some(window) = self.stage.window_by_id(id).and_then(|w| w.client())
                    && window
                        .wl_surface()
                        .is_some_and(|s| self.pending_center.contains(&*s))
                    && let Some(toplevel) = window.toplevel()
                {
                    toplevel.with_pending_state(|s| s.size = None);
                    toplevel.send_configure();
                }
            }
            for column in self.stage.clear_tiling_for_output(&view.output.name()) {
                let Some(element) = self.stage.window_by_id(column.id).cloned() else {
                    continue;
                };
                self.drop_owed_recenter(&element);
                if let Some(window) = element.client() {
                    self.cancel_window_animation(window);
                    self.send_size_configure(window, column.restore.size);
                }
                let center = visual_frame_center(
                    column.restore.loc,
                    column.restore.size,
                    f64::from(self.window_ssd_bar(&element)),
                );
                self.establish_exit_placement(
                    &element,
                    column.restore.loc,
                    column.restore.size,
                    center,
                    true,
                    false,
                );
                if let Some(screen_rect) = tiled_screen_rects.get(&column.id)
                    && view.zoom > 0.0
                {
                    let seed = smithay::utils::Rectangle::<f64, Logical>::new(
                        Point::from((
                            view.camera.x + screen_rect.loc.x / view.zoom,
                            view.camera.y + screen_rect.loc.y / view.zoom,
                        )),
                        smithay::utils::Size::from((
                            screen_rect.size.w / view.zoom,
                            screen_rect.size.h / view.zoom,
                        )),
                    );
                    self.begin_element_geometry_animation_seeded(
                        &element,
                        seed,
                        super::window_animation::AnimSpace::Canvas,
                        element.client().map(|_| column.restore.size),
                        super::window_animation::GeometryRole::Normal,
                        super::window_animation::ContentPolicy::Stretch,
                        Some(column.restore.loc),
                    );
                }
            }
        }
        self.update_output_from_camera();
        self.render.blur_geometry_generation += 1;
        self.mark_all_dirty();
        self.session_store_mark_dirty();
        self.tiling_mode_cues.clear();
    }

    pub(crate) fn focus_tiling_column(&mut self, direction: Direction) {
        self.select_focused_tile();
        let target = match direction {
            Direction::Left => self.stage.adjacent_column(true),
            Direction::Right => self.stage.adjacent_column(false),
            Direction::Up => self.stage.adjacent_tile(true),
            Direction::Down => self.stage.adjacent_tile(false),
            _ => None,
        };
        if let Some(id) = target
            && let Some(element) = self.stage.window_by_id(id).cloned()
        {
            // The camera follows the selected tile. Don't let sloppy focus
            // pull keyboard focus back to whatever scrolls under the cursor.
            self.hover_follows_camera = false;
            self.stage.select_tile(id);
            self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            self.refresh_tiling();
        }
    }

    pub(crate) fn complete_tiling_reorder(
        &mut self,
        source: driftline::stage::ElementId,
        output_name: &str,
        position: Point<f64, Logical>,
        drag_delta: Point<f64, Logical>,
    ) {
        if self.tiling_view.is_none()
            || self.stage.tiling_output_for_window(source) != Some(output_name)
        {
            return;
        }
        let active_output_matches = self
            .active_output()
            .is_some_and(|output| output.name() == output_name);
        let target = self
            .topmost_client_under(position)
            .and_then(|window| self.stage.id_of(&window))
            .filter(|target| {
                active_output_matches
                    && *target != source
                    && self.stage.is_tiled(*target)
                    && self.stage.tiling_output_for_window(*target) == Some(output_name)
            });
        if let Some(target) = target
            && self.stage.swap_tiled_windows(source, target)
        {
            self.refresh_tiling();
            if let Some(element) = self.stage.window_by_id(source).cloned() {
                self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            }
            return;
        }

        if drag_delta.x.abs() < 1.0 && drag_delta.y.abs() < 1.0 {
            return;
        }
        if self.config.effects.animation_speed >= 1.0 {
            return;
        }
        let Some(window) = self
            .stage
            .window_by_id(source)
            .and_then(|element| element.client())
            .cloned()
        else {
            return;
        };
        let Some(origin) = self.stage.position_of(&window) else {
            return;
        };
        let mut seed = self.geometry_seed(source, origin, window.geometry().size);
        seed.loc += drag_delta;
        self.begin_geometry_animation_seeded(
            &window,
            seed,
            super::window_animation::AnimSpace::Canvas,
            None,
            super::window_animation::GeometryRole::Normal,
            super::window_animation::ContentPolicy::Stretch,
            Some(origin),
        );
        if self.window_animations.generation_of(source).is_some() {
            self.mark_all_dirty();
        }
    }

    pub(crate) fn move_into_tiling_column(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_into_column(direction == Direction::Left) {
            self.refresh_tiling();
        }
    }

    pub(crate) fn expel_tiling_window(&mut self) {
        if self.select_focused_tile() && self.stage.expel_from_column() {
            self.refresh_tiling();
        }
    }

    pub(crate) fn consume_reference_tiling_window(&mut self) {
        if self.select_focused_tile() && self.stage.consume_window_into_column() {
            self.refresh_tiling();
        }
    }

    pub(crate) fn expel_reference_tiling_window(&mut self) {
        if self.select_focused_tile() && self.stage.expel_window_from_column() {
            if let Some(element) = self
                .stage
                .active_tile()
                .and_then(|id| self.stage.window_by_id(id))
                .cloned()
            {
                self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            }
            self.refresh_tiling();
        }
    }

    pub(crate) fn move_tiling_column(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_column(direction == Direction::Left) {
            self.refresh_tiling();
        }
    }

    pub(crate) fn move_tiling_window(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_tile(direction == Direction::Up) {
            self.refresh_tiling();
        }
    }

    fn select_focused_tile(&mut self) -> bool {
        let Some(element) = self.focused_element() else {
            return false;
        };
        self.stage
            .id_of(&element)
            .is_some_and(|id| self.stage.select_tile(id))
    }

    fn tiling_eligible(&self, element: &StageWindow) -> bool {
        let Some(window) = element.client() else {
            return false;
        };
        let Some(surface) = window.wl_surface() else {
            return false;
        };
        !window.is_widget()
            && window.parent_surface().is_none()
            && self.is_canvas_window(element)
            && !self.stage.is_fit(element)
            && !self.stage.is_fill(element)
            && !self.pending_center.contains(&*surface)
            && !self.pending_recenter.contains_key(&surface.id())
            && !self.element_under_interactive_grab(element)
            && !self.hidden_by_deferred_adopt(element)
    }

    pub(crate) fn initial_tiling_size(
        &self,
        window: &smithay::desktop::Window,
        decoration: &driftline::config::DecorationMode,
        rule: Option<&driftline::config::AppliedWindowRule>,
    ) -> Option<smithay::utils::Size<i32, Logical>> {
        use smithay::wayland::compositor::with_states;
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        let view = self.tiling_view.as_ref()?;
        let surface = window.wl_surface()?;
        if self.active_output().as_ref() != Some(&view.output)
            || self.zoom() != 1.0
            || window.parent_surface().is_some()
            || self.stage.is_fullscreen(window)
            || self.stage.is_fit(window)
            || self.stage.is_fill(window)
            || self.stage.is_pinned(window)
            || self.pending_fullscreen.contains_key(&*surface)
            || self.pending_fit.contains(&*surface)
            || self.pending_adoptions.contains_key(&*surface)
            || self.root_hidden_by_deferred_adopt(&surface)
            || rule.is_some_and(|r| {
                r.widget
                    || r.pinned_to_screen
                    || r.fullscreen == Some(true)
                    || r.size.is_some()
                    || r.position.is_some()
                    || r.output.is_some()
            })
        {
            return None;
        }
        let app_id = window.app_id_or_class().unwrap_or_default();
        if !app_id.is_empty()
            && self.pending_relaunches.keys().any(|sid| {
                self.find_suspended(*sid)
                    .is_some_and(|s| s.identity.app_id == app_id)
            })
        {
            return None;
        }
        let initial_sent = with_states(&surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .is_some_and(|data| data.lock().unwrap().initial_configure_sent)
        });
        if initial_sent {
            return None;
        }
        let constraints = SizeConstraints::for_window(window);
        let column = Column {
            width: scrolling::ColumnWidth::default(),
            tiles: vec![Tile {
                id: self.stage.id_of(window)?,
                constraints: Constraints {
                    min: constraints.min,
                    max: constraints.max,
                },
                chrome: self.mapping_chrome(&surface, decoration),
            }],
        };
        scrolling::arrange_columns(
            &[column],
            (0, 0).into(),
            self.usable_area_on(&view.output).size,
            0,
        )
        .ok()?
        .first()
        .map(|p| p.content.size)
    }

    pub(crate) fn record_initial_tiling_configure(
        &mut self,
        window: &smithay::desktop::Window,
        serial: smithay::utils::Serial,
        size: smithay::utils::Size<i32, Logical>,
    ) {
        if let Some(id) = self.stage.id_of(window)
            && let Some(view) = self.tiling_view.as_mut()
        {
            view.initial_configures.insert(
                id,
                InitialTilingConfigure {
                    serial,
                    size,
                    sent_at: std::time::Instant::now(),
                    expired: false,
                },
            );
            self.arm_tiling_configure_deadline();
        }
    }

    pub(super) fn focus_tiling_navigation(
        &mut self,
        window: &smithay::desktop::Window,
        output: &Output,
        serial: smithay::utils::Serial,
    ) -> bool {
        if self.tiling_view.select_output(output) {
            self.stage.activate_tiling_output(&output.name());
        }
        if self
            .tiling_view
            .as_ref()
            .is_some_and(|view| &view.output == output)
            && self.tiling_eligible(&StageWindow::Client(window.clone()))
        {
            // Reflow owns the camera target; a new window's temporary canvas
            // placement must not overwrite the stationary tiling view.
            self.raise_and_focus(window, serial);
            // Alt-Tab and activation can land here without running the canvas
            // navigation path below. Their camera movement is in service of
            // the chosen tile, so a resting cursor must not pull focus back to
            // whichever tile the reflow places beneath it.
            self.hover_follows_camera = false;
            return true;
        }
        false
    }

    pub(crate) fn preserve_tiling_camera_for_hover(&mut self, window: &smithay::desktop::Window) {
        let element = StageWindow::Client(window.clone());
        let Some(id) = self.stage.id_of(&element) else {
            return;
        };
        let Some(output) = self.stage.tiling_output_for_window(id) else {
            return;
        };
        if let Some(view) = self.tiling_view.views.get_mut(output) {
            let previous = view
                .hover_camera_guard
                .map(|(anchor, _)| anchor)
                .or(view.selected);
            if let Some(previous) = previous {
                view.hover_camera_guard = Some((previous, id));
            }
        }
    }

    fn select_tiling_output_for_window(&mut self, id: driftline::stage::ElementId) -> bool {
        let Some(output) = self.stage.tiling_output_for_window(id).map(str::to_owned) else {
            return false;
        };
        if !self.tiling_view.select_output_name(&output) {
            return false;
        }
        self.stage.activate_tiling_output(&output);
        true
    }

    pub(crate) fn refresh_tiling(&mut self) {
        self.refresh_tiling_at(std::time::Instant::now());
    }

    pub(crate) fn tiling_column_min_width(&self, column: driftline::stage::ColumnId) -> i32 {
        self.stage
            .column(column)
            .into_iter()
            .flat_map(|column| column.windows.iter())
            .filter_map(|tile| {
                let element = self.stage.window_by_id(tile.id)?;
                let window = element.client()?;
                let minimum = SizeConstraints::for_window(window).min.w.max(1);
                Some(minimum.saturating_add(2 * self.element_chrome(element).border))
            })
            .max()
            .unwrap_or(1)
    }

    fn refresh_tiling_for_output(&mut self, output: &Output, now: std::time::Instant) {
        let previous_output = self.focused_output.replace(output.clone());
        self.tiling_view.select_output(output);
        self.stage.activate_tiling_output(&output.name());
        self.refresh_tiling_at(now);
        self.focused_output = previous_output;
    }

    pub(crate) fn refresh_tiling_at(&mut self, now: std::time::Instant) {
        let Some(output) = self.active_output() else {
            return;
        };
        if !self.tiling_view.select_output(&output) {
            return;
        }
        self.stage.activate_tiling_output(&output.name());
        // Fullscreen parks its window at the viewport origin and gives it the
        // output's size. Keep the saved tile slot intact, but don't validate or
        // reflow this output's layout against fullscreen geometry until exit.
        if self.is_output_fullscreen(&output) {
            return;
        }
        let output_name = output.name();
        // Interactive width changes need canvas-like feedback: client buffers
        // may take several frames to honor their configure, but that must not
        // hold the neighboring columns' positions behind a commit barrier.
        let interactive_resize = self
            .tiling_width_indicator
            .as_ref()
            .is_some_and(|(_, resizing_output, _)| *resizing_output == output_name);
        let Some(view) = self.tiling_view.as_ref() else {
            return;
        };
        if self.stage.tiling_layout_id() != Some(view.layout_id)
            || self.disconnected_outputs.contains(&view.output.name())
            || self.zoom() != 1.0
        {
            self.prepare_tiling_operation(super::TilingOperation::LayoutInvalidated);
            return;
        }
        let origin = view.origin;
        let previous_anchor = view.anchor;
        let saved_return = view.close_return;
        let stationary_x = self.camera_target().unwrap_or_else(|| self.camera()).x;
        let previous_active = self.stage.active_column();

        let selected_closed = view.selected.is_some_and(|id| !self.stage.is_tiled(id));
        self.tiling_view
            .as_mut()
            .unwrap()
            .snapshots
            .retain(|id, _| self.stage.is_tiled(*id) && self.stage.window_by_id(*id).is_some());
        let recover_focus = selected_closed
            && !self.suppress_auto_anchor
            && self.focused_anchor_element().is_none();
        let close_return = self
            .stage
            .take_tiling_close_return()
            .and_then(|(closed, previous)| {
                saved_return.filter(|saved| {
                    saved.opened == closed
                        && saved.previous == previous
                        && self.stage.active_column() == Some(previous)
                        && recover_focus
                })
            });
        if recover_focus {
            let serial = SERIAL_COUNTER.next_serial();
            if let Some(element) = self
                .stage
                .active_tile()
                .and_then(|id| self.stage.window_by_id(id))
                .cloned()
            {
                self.raise_and_focus_element(&element, serial);
            } else {
                self.set_window_focus(None, serial);
            }
        }

        if self
            .stage
            .tiled_windows_for_output(&output_name)
            .any(|column| {
                self.stage.window_by_id(column.id).is_some_and(|w| {
                    // An existing tile stays in its layout while the client
                    // commits the restored size after fullscreen.
                    let restoring_tiled_window = w
                        .wl_surface()
                        .is_some_and(|surface| self.pending_recenter.contains_key(&surface.id()));
                    !self.tiling_eligible(w) && !restoring_tiled_window
                })
            })
        {
            self.prepare_tiling_operation(super::TilingOperation::LayoutInvalidated);
            return;
        }
        let candidates: Vec<_> = self
            .stage
            .windows()
            .filter(|w| self.tiling_eligible(w))
            .filter(|w| {
                let Some(id) = self.stage.id_of(*w) else {
                    return false;
                };
                if let Some(owner) = self.stage.tiling_output_for_window(id) {
                    owner == output_name
                } else {
                    self.output_for_tiling_window(*w)
                        .is_some_and(|owner| owner.name() == output_name)
                }
            })
            .filter_map(|w| {
                let id = self.stage.id_of(w)?;
                let window = w.client()?;
                let initial = self
                    .tiling_view
                    .as_ref()?
                    .initial_configures
                    .contains_key(&id);
                let restore = if initial {
                    window.geometry().size
                } else {
                    configured_window_size(window)
                };
                Some((id, restore))
            })
            .collect();
        let mut added = false;
        for (id, restore_size) in candidates {
            added |= self.stage.tile(id, restore_size);
        }
        if let Some(element) = self.focused_element()
            && let Some(id) = self.stage.id_of(&element)
            && self.stage.tiling_output_for_window(id) == Some(output_name.as_str())
        {
            self.stage.select_tile(id);
        }
        let columns: Vec<Column> = self
            .stage
            .tiled_columns()
            .iter()
            .map(|column| Column {
                width: column.width,
                tiles: column
                    .windows
                    .iter()
                    .filter_map(|tile| {
                        let element = self.stage.window_by_id(tile.id)?;
                        let window = element.client()?;
                        let constraints = SizeConstraints::for_window(window);
                        Some(Tile {
                            id: tile.id,
                            constraints: Constraints {
                                min: constraints.min,
                                max: constraints.max,
                            },
                            chrome: self.element_chrome(element),
                        })
                    })
                    .collect(),
            })
            .collect();
        let usable = self.get_usable_area();
        let gap = self.config.snap_gap.max(0.0).round() as i32;
        let mut placements = match scrolling::arrange_columns(&columns, origin, usable.size, gap) {
            Ok(placements) => placements,
            Err(error) => {
                tracing::warn!(?error, "Leaving scrolling tiling after invalid layout");
                self.prepare_tiling_operation(super::TilingOperation::LayoutInvalidated);
                return;
            }
        };
        let generation = self
            .tiling_view
            .as_mut()
            .unwrap()
            .configures
            .update_layout(&placements);
        if let Some(view) = self.tiling_view.as_mut() {
            view.initial_configures.retain(|id, initial| {
                let Some(element) = self.stage.window_by_id(*id) else {
                    return false;
                };
                if !self.stage.is_tiled(*id)
                    && element
                        .wl_surface()
                        .is_none_or(|surface| !self.pending_center.contains(&*surface))
                {
                    return false;
                }
                if self.stage.is_tiled(*id) {
                    if placements
                        .iter()
                        .any(|p| p.id == *id && p.content.size == initial.size)
                    {
                        view.configures.record(*id, initial.serial, initial.sent_at);
                    }
                    return false;
                }
                initial.expired |=
                    now >= initial.sent_at + scrolling::transaction::CONFIGURE_TIMEOUT;
                true
            });
        }
        // A client can delay or refuse a size. Reserve its current frame as
        // well as the requested one without feeding that size back into configures.
        let mut extra_x = 0i64;
        let mut start = 0;
        for column in &columns {
            let group = &mut placements[start..start + column.tiles.len()];
            start += column.tiles.len();
            let desired_width = group
                .iter()
                .map(|p| i64::from(p.frame.size.w))
                .max()
                .unwrap_or(1);
            let mut reserved_width = desired_width;
            let mut extra_y = 0i64;
            for placement in group {
                let Some(element) = self.stage.window_by_id(placement.id) else {
                    continue;
                };
                let chrome = self.element_chrome(element);
                let actual = element.geometry().size;
                let width = i64::from(placement.frame.size.w)
                    .max(i64::from(actual.w) + 2 * i64::from(chrome.border));
                let height = i64::from(placement.frame.size.h).max(
                    i64::from(actual.h) + i64::from(chrome.bar) + 2 * i64::from(chrome.border),
                );
                let x = i64::from(placement.frame.loc.x) + extra_x;
                let y = i64::from(placement.frame.loc.y) + extra_y;
                let content_x = i64::from(placement.content.loc.x) + extra_x;
                let content_y = i64::from(placement.content.loc.y) + extra_y;
                if [x + width, y + height, width, height, content_x, content_y]
                    .iter()
                    .any(|v| i32::try_from(*v).is_err())
                {
                    self.prepare_tiling_operation(super::TilingOperation::LayoutInvalidated);
                    return;
                }
                placement.frame.loc = (x as i32, y as i32).into();
                placement.content.loc = (content_x as i32, content_y as i32).into();
                extra_y += height - i64::from(placement.frame.size.h);
                placement.frame.size = (width as i32, height as i32).into();
                reserved_width = reserved_width.max(width);
            }
            extra_x += reserved_width - desired_width;
        }
        self.expire_tiling_commits(now);
        if !interactive_resize
            && placements.iter().any(|p| {
                self.stage
                    .window_by_id(p.id)
                    .and_then(|w| w.client())
                    .is_some_and(|w| configured_window_size(w) != p.content.size)
            })
        {
            self.capture_tiling_snapshots();
        }
        let mut requests = std::collections::HashMap::new();
        let mut changed = false;
        for placement in &placements {
            let Some(element) = self.stage.window_by_id(placement.id).cloned() else {
                continue;
            };
            if let Some(window) = element.client()
                && configured_window_size(window) != placement.content.size
            {
                if let Some(serial) =
                    self.send_size_configure_serial(window, placement.content.size)
                {
                    self.tiling_view
                        .as_mut()
                        .unwrap()
                        .configures
                        .record(placement.id, serial, now);
                    requests.insert(placement.id, serial);
                }
                changed = true;
            }
        }
        if !requests.is_empty() {
            let mut group_started_at = now;
            if let Some(mut previous) = self.tiling_view.as_mut().unwrap().commits.take() {
                for (&id, &serial) in previous.group.pending_requests() {
                    if self.stage.is_tiled(id) {
                        group_started_at = group_started_at.min(
                            previous.group.deadline() - scrolling::transaction::CONFIGURE_TIMEOUT,
                        );
                        requests.entry(id).or_insert(serial);
                    }
                }
                previous.group.cancel();
                self.settle_tiling_commits(previous.clients);
            }
            let mut presentation_requests = requests.clone();
            if let Some(previous) = self.tiling_view.as_mut().unwrap().presentation.take() {
                if !previous.requests.is_empty() {
                    group_started_at = group_started_at
                        .min(previous.deadline - scrolling::transaction::CONFIGURE_TIMEOUT);
                }
                for (id, serial) in previous.requests {
                    if self.stage.is_tiled(id) {
                        presentation_requests.entry(id).or_insert(serial);
                    }
                }
            }
            let clients = requests
                .keys()
                .filter_map(|id| self.stage.window_by_id(*id)?.wl_surface()?.client())
                .collect();
            self.tiling_view.as_mut().unwrap().presentation = Some(TilingPresentation {
                requests: presentation_requests,
                deadline: group_started_at + scrolling::transaction::CONFIGURE_TIMEOUT,
            });
            self.tiling_view.as_mut().unwrap().commits = Some(TilingCommits {
                group: scrolling::commit_group::CommitGroup::new(
                    generation,
                    requests,
                    group_started_at,
                ),
                generation,
                clients,
            });
        }
        self.tiling_view.as_mut().unwrap().configures.advance(now);
        for placement in &placements {
            let Some(window) = self
                .stage
                .window_by_id(placement.id)
                .and_then(|w| w.client())
            else {
                continue;
            };
            let serial = window
                .toplevel()
                .and_then(|t| t.with_cached_state(|s| s.last_acked.as_ref().map(|c| c.serial)));
            if let Some(serial) = serial {
                self.tiling_view
                    .as_mut()
                    .unwrap()
                    .configures
                    .observe_commit(generation, placement.id, serial, window.geometry().size);
            }
        }
        self.arm_tiling_configure_deadline();
        if changed {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
        let return_pair = self
            .stage
            .active_column()
            .zip(self.stage.close_return_column());
        let new_return = if added {
            return_pair.and_then(|(opened, previous)| {
                previous_anchor
                    .filter(|(column, _)| Some(*column) == previous_active && *column == previous)
                    .map(|(_, x)| CloseReturnView {
                        opened,
                        previous,
                        offset: stationary_x - f64::from(x),
                    })
            })
        } else {
            None
        };
        if let Some(view) = self.tiling_view.as_mut() {
            view.selected = self.stage.active_tile();
            view.close_return = new_return
                .or(saved_return)
                .filter(|saved| return_pair == Some((saved.opened, saved.previous)));
            view.pending_close_return = close_return
                .or(view.pending_close_return)
                .filter(|saved| self.stage.active_column() == Some(saved.previous));
        }
        if added {
            self.session_store_mark_dirty();
        }
        let waiting_for_clients = self
            .tiling_view
            .as_ref()
            .is_some_and(|v| v.commits.is_some() || v.commit_wakeups != 0)
            || !self.tiling_presentation_ready(now);
        if waiting_for_clients && !interactive_resize {
            return;
        }
        let view = self.tiling_view.as_mut().unwrap();
        view.presentation = None;
        let snapshots = if interactive_resize {
            view.snapshots.clear();
            std::collections::HashMap::new()
        } else {
            std::mem::take(&mut view.snapshots)
        };
        if !snapshots.is_empty() {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
        let close_return = self
            .tiling_view
            .as_mut()
            .unwrap()
            .pending_close_return
            .take();
        let mut moved = false;
        for placement in &placements {
            if let Some(element) = self.stage.window_by_id(placement.id).cloned()
                && self.stage.position_of(&element) != Some(placement.content.loc)
            {
                self.stage.set_position(&element, placement.content.loc);
                moved = true;
            }
        }
        if moved {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
        let selected = placements
            .iter()
            .find(|p| Some(p.id) == self.stage.active_tile())
            .map(|p| (p.id, p.frame));
        let hover_camera_anchor = self
            .tiling_view
            .as_ref()
            .and_then(|view| view.hover_camera_guard)
            .filter(|(_, target)| selected.is_some_and(|(id, _)| id == *target))
            .and_then(|(anchor, _)| {
                placements
                    .iter()
                    .find(|placement| placement.id == anchor)
                    .map(|placement| placement.frame)
            });
        let anchor = self
            .stage
            .active_column()
            .zip(selected.map(|(_, frame)| frame.loc.x));
        let mut anchor_translation = 0.0;
        let translated = if let Some((column, x)) = anchor
            && let Some((previous_column, previous_x)) =
                self.tiling_view.as_ref().and_then(|v| v.anchor)
            && column == previous_column
            && x != previous_x
        {
            // Structural changes before the selected column must not scroll
            // its content under a stationary viewport, including mid-flight.
            let delta = f64::from(x) - f64::from(previous_x);
            anchor_translation = delta;
            self.with_output_state(|os| {
                os.camera.x += delta;
                if let Some(target) = &mut os.camera_target {
                    target.x += delta;
                }
            });
            self.update_output_from_camera();
            if let Some(pointer) = self.seat.get_pointer() {
                self.warp_pointer(pointer.current_location() + Point::from((delta, 0.0)));
            }
            true
        } else {
            false
        };
        // Placement remains atomic in Stage; the renderer chases from the
        // picture held by the completed transaction without another configure.
        for (id, mut snapshot) in snapshots {
            if self.config.effects.animation_speed >= 1.0 {
                continue;
            }
            let Some(window) = self
                .stage
                .window_by_id(id)
                .and_then(|w| w.client())
                .cloned()
            else {
                continue;
            };
            let old_size = snapshot.blur.geometry.size;
            let new_size = window.geometry().size;
            let mut seed = snapshot
                .held_geometry
                .unwrap_or_else(|| self.geometry_seed(id, snapshot.blur.position, old_size));
            if translated {
                seed.loc.x += anchor_translation;
            }
            let destination = self.stage.position_of(&window).unwrap_or_default();
            let content_changed = old_size != new_size || snapshot.held_geometry.is_some();
            if seed == smithay::utils::Rectangle::new(destination.to_f64(), new_size.to_f64())
                && !content_changed
            {
                continue;
            }
            self.begin_geometry_animation_seeded(
                &window,
                seed,
                super::window_animation::AnimSpace::Canvas,
                None,
                super::window_animation::GeometryRole::Normal,
                super::window_animation::ContentPolicy::Stretch,
                Some(destination),
            );
            if content_changed
                && self.window_animations.generation_of(id).is_some()
                && let Some(mut resize) = snapshot.resize.take()
            {
                resize.preserve_initial_pixels();
                resize.set_resize_direction(old_size, new_size);
                self.resize_crossfades.insert(id, resize);
            }
        }
        if let Some(saved) = close_return
            && self.stage.active_column() == Some(saved.previous)
            && let Some((_, frame)) = selected
            && let Ok(target) = scrolling::reveal(
                frame,
                usable,
                Point::from((f64::from(frame.loc.x) + saved.offset, self.camera().y))
                    .to_i32_round(),
            )
        {
            self.set_camera_target(Some(target.to_f64()));
        } else if let Some((id, frame)) = selected
            && self.tiling_view.as_ref().is_some_and(|v| {
                let translation_only = translated
                    && v.last_usable == Some(usable)
                    && v.revealed.is_some_and(|(previous_id, previous_frame)| {
                        previous_id == id
                            && previous_frame.size == frame.size
                            && previous_frame.loc.y == frame.loc.y
                    });
                !translation_only
                    && (v.revealed != Some((id, frame)) || v.last_usable != Some(usable))
            })
            && let Ok(target) = scrolling::reveal(frame, usable, self.camera().to_i32_round())
        {
            let target = hover_camera_anchor.map_or(target, |anchor| {
                preserve_hover_target(target, anchor, usable)
            });
            self.set_camera_target(Some(target.to_f64()));
        }
        if let Some(view) = self.tiling_view.as_mut() {
            view.hover_camera_guard = None;
            view.revealed = selected;
            view.last_usable = Some(usable);
            view.anchor = anchor;
        }
    }

    fn capture_tiling_snapshots(&mut self) {
        use smithay::utils::Rectangle;
        let Some(mut backend) = self.backend.take() else {
            return;
        };
        let output_name = self.tiling_view.as_ref().unwrap().output.name();
        let tiles: Vec<_> = self
            .stage
            .tiled_windows_for_output(&output_name)
            .map(|t| t.id)
            .collect();
        let focused = self.focus_root_window();
        for id in tiles {
            if self
                .tiling_view
                .as_ref()
                .unwrap()
                .snapshots
                .contains_key(&id)
            {
                continue;
            }
            let Some(element) = self.stage.window_by_id(id) else {
                continue;
            };
            let Some(window) = element.client() else {
                continue;
            };
            let Some(surface) = window.wl_surface() else {
                continue;
            };
            let Some(position) = self.stage.position_of(element) else {
                continue;
            };
            let geometry = window.geometry();
            let translating = self
                .window_animations
                .is_canvas_translation(id, geometry.size)
                && !self.resize_crossfades.contains_key(&id);
            let has_geometry = self.window_animations.generation_of(id).is_some();
            if has_geometry && !self.window_animations.can_capture_canvas_geometry(id) {
                continue;
            }
            let crossfading = self.resize_crossfades.contains_key(&id);
            let held_geometry = ((has_geometry && !translating) || crossfading).then(|| {
                self.window_animations
                    .geometry_visual_rect(id)
                    .unwrap_or_else(|| Rectangle::new(position.to_f64(), geometry.size.to_f64()))
            });
            let visible = if let Some(held) = held_geometry {
                held.to_i32_round()
            } else if translating {
                self.window_animations
                    .geometry_visual_rect(id)
                    .unwrap()
                    .to_i32_round()
            } else {
                Rectangle::new(position, geometry.size)
            };
            let visible = self.window_animations.opening_visual(
                id,
                visible.to_f64(),
                self.config.effects.animation_scale,
            );
            let visible = Rectangle::new(visible.loc, visible.size).to_i32_round();
            if !self.canvas_rect_drawable(visible) {
                continue;
            }
            let opacity = self.effective_opacity_of(window, focused.as_ref());
            let scale = self.flatten_scale_for_canvas_rect(visible);
            let retained_content = if crossfading {
                let stamp = crate::render::ResizeContentStamp::for_surface(
                    &surface,
                    geometry,
                    scale.max(1.0),
                    self.baked_chrome_policy(&surface, false),
                );
                let clip = self.render.corner_clip_shader.clone();
                self.resize_crossfades.get_mut(&id).unwrap().blended_buffer(
                    backend.renderer(),
                    &surface,
                    stamp,
                    clip.as_ref(),
                )
            } else {
                None
            };
            let pixels = if crossfading {
                retained_content.as_ref().map(|(buffer, texels)| {
                    crate::render::ClosePixels::from_resize_content(
                        buffer.clone(),
                        *texels,
                        geometry,
                    )
                })
            } else {
                crate::render::capture_close_pixels(
                    backend.renderer(),
                    &surface,
                    geometry,
                    std::time::Instant::now(),
                )
            };
            let Some(pixels) = pixels else {
                continue;
            };
            let (client_regions, client_effect) =
                smithay::wayland::compositor::with_states(&surface, |states| {
                    (
                        crate::handlers::background_effect::get_cached_blur_region(states),
                        crate::handlers::background_effect::has_committed_background_effect(states),
                    )
                });
            let blur = crate::render::SnapshotBlur {
                surface_id: surface.id(),
                position,
                geometry,
                bar_height: self
                    .decorations
                    .contains_key(&crate::decorations::DecorationKey::Surface(surface.id()))
                    .then_some(self.config.decorations.title_bar_height),
                window_blur: driftline::config::effective_blur(
                    driftline::config::applied_rule(&surface).as_ref(),
                    &self.config.decorations,
                    false,
                ),
                client_regions,
            };
            let mask_policy = self
                .resize_crossfades
                .get(&id)
                .and_then(|resize| resize.initial_blur_policy())
                .cloned();
            let retained_mask = if crossfading {
                self.render.blur_mask_shader.as_ref().and_then(|shader| {
                    self.resize_crossfades.get_mut(&id)?.blended_blur_mask(
                        backend.renderer(),
                        shader,
                        &blur,
                        held_geometry
                            .map_or(geometry.size.to_f64(), |r| r.size)
                            .to_physical(scale)
                            .to_i32_round(),
                    )
                })
            } else {
                None
            };
            let native_chrome = crossfading
                || blur.bar_height.is_some()
                || self.window_animations.open_progress(id).is_some()
                || (has_geometry && (!translating || client_effect));
            let chrome = self
                .snapshot_chrome(&surface, geometry, false)
                .map(|mut chrome| {
                    if let Some((buffer, mut rect)) = chrome.bar {
                        let height = (rect.size.h * scale).round() / scale;
                        rect.loc.y = f64::from(geometry.loc.y) - height;
                        rect.size.h = height;
                        chrome.bar = Some((buffer, rect));
                    }
                    chrome
                });
            let origin = position.to_f64() - geometry.loc.to_f64();
            let shadow = chrome.and_then(|c| {
                c.shadow_shader.map(|_| {
                    let border = f64::from(c.border_width.max(0));
                    let bar = c.bar.map_or(0.0, |(_, rect)| rect.size.h);
                    crate::render::SnapshotShadow {
                        surface_id: surface.id(),
                        body: Rectangle::new(
                            position.to_f64() - Point::from((border, bar + border)),
                            (
                                f64::from(geometry.size.w) + 2.0 * border,
                                f64::from(geometry.size.h) + bar + 2.0 * border,
                            )
                                .into(),
                        ),
                        inner_radius: c.corner_radius[2],
                        border_width: c.border_width,
                        opacity,
                    }
                })
            });
            let border = chrome.and_then(|c| {
                c.border_shader.map(|_| {
                    let bar = c.bar.map_or(0.0, |(_, rect)| rect.size.h);
                    crate::render::SnapshotBorder {
                        surface_id: surface.id(),
                        inner: Rectangle::new(
                            position.to_f64() - Point::from((0.0, bar)),
                            (f64::from(geometry.size.w), f64::from(geometry.size.h) + bar).into(),
                        ),
                        radius: c.corner_radius[2],
                        width: c.border_width,
                        color: c.border_color,
                        focused: c.focused,
                        opacity,
                    }
                })
            });
            // Decoration updates replace their buffer, so this clone retains
            // the native-size title pixels across a superseding resize.
            let bar = self
                .resize_crossfades
                .get(&id)
                .and_then(|resize| resize.title.as_ref())
                .map(|title| title.retained(backend.renderer()))
                .or_else(|| {
                    native_chrome
                        .then(|| {
                            chrome.and_then(|c| {
                                c.bar
                                    .map(|(b, _)| crate::render::TitlePixels::Memory(b.clone()))
                            })
                        })
                        .flatten()
                });
            // Separate chrome preserves live alpha blending and output-space
            // rounding. Shadows also stay below frost instead of in its mask.
            let chrome = chrome.map(|mut c| {
                if native_chrome {
                    c.bar = None;
                }
                c.shadow_shader = None;
                c.border_shader = None;
                if crossfading {
                    c.corner_clip = None;
                }
                c
            });
            // Keep the mixed texture's texel grid. Flattening it again filters
            // fractional-scale corner pixels before the same output transform.
            let retained_content = retained_content.filter(|_| native_chrome);
            let image = if let Some((buffer, texels)) = &retained_content {
                Some(crate::render::ClosingSnapshot::from_retained_content(
                    buffer.clone(),
                    *texels,
                    Rectangle::new(position.to_f64(), geometry.size.to_f64()),
                ))
            } else {
                crate::render::snapshot_canvas(
                    backend.renderer(),
                    &pixels,
                    origin,
                    scale,
                    1.0,
                    true,
                    chrome.as_ref(),
                )
            };
            if let Some(image) = image {
                let popups: Option<Vec<_>> = mapped_tiling_popups(&surface)
                    .map(|(popup, offset)| {
                        let geometry = popup.geometry();
                        let pixels = crate::render::capture_close_pixels(
                            backend.renderer(),
                            popup.wl_surface(),
                            geometry,
                            std::time::Instant::now(),
                        )?;
                        // Popups use their own surface origin and bypass the
                        // parent's corner clip, including portions outside it.
                        let origin = position.to_f64() + offset.to_f64() - geometry.loc.to_f64();
                        let image = crate::render::snapshot_canvas(
                            backend.renderer(),
                            &pixels,
                            origin,
                            scale,
                            1.0,
                            true,
                            None,
                        )?;
                        Some(TilingPopupSnapshot {
                            surface: popup.wl_surface().clone(),
                            offset,
                            geometry,
                            image,
                            stamp: popup_stamp(popup.wl_surface(), geometry),
                        })
                    })
                    .collect();
                if let Some(popups) = popups {
                    // Bake while the original client textures are still valid:
                    // an early peer's new buffer can apply before group release.
                    let mut resize = (self.config.effects.animation_speed < 1.0)
                        .then(|| {
                            if let Some((buffer, texels)) = retained_content {
                                return Some(
                                    crate::render::ResizeCrossfade::from_retained_content(
                                        buffer, texels,
                                    ),
                                );
                            }
                            crate::render::resize_crossfade(
                                backend.renderer(),
                                &pixels,
                                geometry.size,
                                scale,
                                self.render.corner_clip_shader.as_ref(),
                                if crossfading {
                                    crate::render::BakeChrome {
                                        bare: true,
                                        corner_radius: [0.0; 4],
                                    }
                                } else {
                                    self.baked_chrome_policy(&surface, false)
                                },
                            )
                        })
                        .flatten();
                    if let Some(resize) = resize.as_mut() {
                        resize.title = bar.clone().map(crate::render::TitleCrossfade::new);
                        resize.retain_blur_mask(
                            mask_policy.as_ref().unwrap_or(&blur),
                            retained_mask.clone(),
                        );
                    }
                    self.tiling_view.as_mut().unwrap().snapshots.insert(
                        id,
                        TilingSnapshot {
                            image,
                            held_geometry,
                            resize,
                            mask: retained_mask,
                            mask_policy,
                            popups,
                            shadow,
                            border,
                            opacity,
                            native_chrome,
                            bar,
                            blur,
                        },
                    );
                    if held_geometry.is_some() {
                        self.window_animations.stop_geometry_keep_open(id);
                        self.drop_resize_crossfade(id);
                    }
                }
            }
        }
        self.backend = Some(backend);
    }

    pub(crate) fn release_tiling_snapshot_for_animation(
        &mut self,
        id: driftline::stage::ElementId,
    ) {
        if self
            .tiling_view
            .iter_mut()
            .any(|(_, view)| view.snapshots.remove(&id).is_some())
        {
            // Live animation geometry must replace the saved frost mask too.
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
    }

    pub(crate) fn invalidate_tiling_snapshot_animation(&mut self, id: driftline::stage::ElementId) {
        if self
            .tiling_view
            .iter()
            .any(|(_, view)| view.snapshots.contains_key(&id))
        {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
    }

    pub(crate) fn has_fading_tiling_snapshot(&self) -> bool {
        self.tiling_view.iter().any(|(_, view)| {
            view.snapshots
                .keys()
                .any(|id| self.window_animations.open_progress(*id).is_some())
        })
    }

    pub(crate) fn tiling_snapshot_count(&self) -> usize {
        self.tiling_view
            .iter()
            .map(|(_, view)| {
                view.snapshots
                    .values()
                    .map(|s| 1 + s.popups.len())
                    .sum::<usize>()
            })
            .sum()
    }

    pub(crate) fn tiling_snapshot_elements(
        &self,
        id: driftline::stage::ElementId,
        output: &Output,
        camera: Point<f64, Logical>,
        zoom: f64,
        output_scale: f64,
    ) -> Option<TilingSnapshotElements> {
        let snapshot = self.tiling_view.for_output(output)?.snapshots.get(&id)?;
        let root = self.stage.window_by_id(id)?.wl_surface()?;
        let popups_match = snapshot.matches_popups(&root);
        // Geometry chases contribute displacement; opening scales/fades the
        // captured dimensions, even after the live client buffer changes size.
        let displacement = self
            .window_animations
            .geometry_visual_rect(id)
            .zip(self.stage.position_of(self.stage.window_by_id(id)?))
            .map_or_else(Point::default, |(visual, target)| {
                visual.loc - target.to_f64()
            });
        let base = smithay::utils::Rectangle::new(
            snapshot.blur.position.to_f64(),
            snapshot.blur.geometry.size.to_f64(),
        );
        let frame = snapshot
            .held_geometry
            .unwrap_or_else(|| smithay::utils::Rectangle::new(base.loc + displacement, base.size));
        let visual =
            self.window_animations
                .opening_visual(id, frame, self.config.effects.animation_scale);
        let physical_scale = zoom * output_scale;
        // Opening wraps native chrome; resize changes its bounds, not its stroke.
        let chrome_animation = (snapshot.native_chrome
            && self.window_animations.open_progress(id).is_some())
        .then(|| {
            crate::render::WindowRenderAnimation::between_rects(
                smithay::utils::Rectangle::new(
                    (frame.loc - camera).to_physical(physical_scale),
                    frame.size.to_physical(physical_scale),
                ),
                smithay::utils::Rectangle::new(
                    (visual.loc - camera).to_physical(physical_scale),
                    visual.size.to_physical(physical_scale),
                ),
            )
        });
        let animation = (visual.loc != base.loc || visual.size != base.size).then(|| {
            crate::render::WindowRenderAnimation::between_rects(
                smithay::utils::Rectangle::new(
                    (base.loc - camera).to_physical(physical_scale),
                    base.size.to_physical(physical_scale),
                ),
                smithay::utils::Rectangle::new(
                    (visual.loc - camera).to_physical(physical_scale),
                    visual.size.to_physical(physical_scale),
                ),
            )
        });
        let popup_animation = animation.map(|animation| {
            if snapshot.native_chrome {
                let root =
                    snapshot.blur.position.to_f64() - snapshot.blur.geometry.loc.to_f64() - camera;
                let source: Point<i32, smithay::utils::Physical> = root
                    .to_physical_precise_round(output_scale)
                    + snapshot
                        .blur
                        .geometry
                        .loc
                        .to_physical_precise_round(output_scale);
                return crate::render::WindowRenderAnimation::popup_at(
                    source.to_f64().upscale(smithay::utils::Scale::from(zoom)),
                    (frame.loc - camera)
                        .to_physical_precise_round::<_, i32>(output_scale)
                        .to_f64()
                        .upscale(smithay::utils::Scale::from(zoom)),
                    chrome_animation,
                    self.window_animations
                        .opening_scale(id, self.config.effects.animation_scale),
                );
            }
            animation.for_popups(
                (base.loc - camera).to_physical(physical_scale),
                self.window_animations
                    .opening_scale(id, self.config.effects.animation_scale),
            )
        });
        let opacity = snapshot.opacity * f64::from(visual.alpha);
        let name = output.name();
        let mut elements: Vec<_> = snapshot
            .popups
            .iter()
            .filter(|_| popups_match)
            .filter_map(|popup| {
                // The live composer snaps the root and popup offset separately.
                // Snapping their sum moves menus by a pixel at fractional DPI.
                let root =
                    snapshot.blur.position.to_f64() - snapshot.blur.geometry.loc.to_f64() - camera;
                let offset = snapshot.blur.geometry.loc + popup.offset - popup.geometry.loc;
                let physical_root: Point<i32, smithay::utils::Physical> =
                    root.to_physical_precise_round(output_scale);
                let physical_offset: Point<i32, smithay::utils::Physical> =
                    offset.to_physical_precise_round(output_scale);
                let snapped = (physical_root + physical_offset)
                    .to_f64()
                    .to_logical(output_scale);
                let correction = snapped - (root + offset.to_f64());
                popup.image.render_element_with_animation(
                    &name,
                    camera - correction,
                    zoom,
                    output_scale,
                    opacity,
                    popup_animation,
                )
            })
            .collect();
        if snapshot.native_chrome {
            let rect = smithay::utils::Rectangle::new(
                (frame.loc - camera).to_physical(physical_scale),
                frame.size.to_physical(physical_scale),
            )
            .to_i32_round::<i32>();
            let rect = chrome_animation.map_or(rect, |a| a.transform_phys_rect(rect));
            elements.push(snapshot.image.render_retained_rect(
                rect.to_f64(),
                output_scale,
                opacity,
            ));
        } else {
            elements.extend(snapshot.image.render_element_with_animation(
                &name,
                camera,
                zoom,
                output_scale,
                opacity,
                animation,
            ));
        }
        let mask_policy = snapshot.mask_policy.as_ref().unwrap_or(&snapshot.blur);
        let mut blur = if snapshot.native_chrome
            && mask_policy
                .client_regions
                .as_ref()
                .is_some_and(|r| !r.is_empty())
        {
            mask_policy.request_in_visual(
                smithay::utils::Rectangle::new(frame.loc - camera, frame.size),
                physical_scale,
                chrome_animation,
                elements.len(),
            )
        } else {
            mask_policy
                .request(camera, zoom, output_scale, elements.len())
                .map(|mut request| {
                    // Transform already snapped mask geometry and client regions
                    // with the same affine as the retained content.
                    if snapshot.native_chrome {
                        request.screen_rect = crate::render::SnapshotBlur::visual_screen_rect(
                            smithay::utils::Rectangle::new(frame.loc - camera, frame.size),
                            physical_scale,
                            mask_policy.bar_height,
                        );
                        if let Some(a) = chrome_animation {
                            request.apply_animation(a);
                        }
                    } else if let Some(animation) = animation {
                        request.apply_animation(animation);
                    }
                    request
                })
        };
        if let Some(mask) = &snapshot.mask {
            let mut policy = snapshot.blur.clone();
            policy.window_blur = true;
            policy.client_regions = None;
            let mut request = policy.request_in_visual(
                smithay::utils::Rectangle::new(frame.loc - camera, frame.size),
                physical_scale,
                chrome_animation,
                elements.len(),
            )?;
            let body = smithay::utils::Rectangle::new(frame.loc - camera, frame.size)
                .to_physical(physical_scale)
                .to_i32_round::<i32>();
            let body = chrome_animation.map_or(body, |a| a.transform_phys_rect(body));
            request.chrome_mask = (mask.title_coverage > 0.0).then_some((0, mask.title_coverage));
            request.mask = Some(crate::render::ResizeCrossfade::render_retained_rect(
                &mask.buffer,
                mask.texels,
                body.to_f64(),
                output_scale,
                1.0,
            ));
            blur = Some(request);
        }
        Some(TilingSnapshotElements {
            bar: snapshot.bar.as_ref().map(|buffer| {
                let height = (f64::from(snapshot.blur.bar_height.unwrap_or(0)) * output_scale)
                    .round()
                    / output_scale;
                TilingSnapshotBar {
                    buffer: buffer.clone(),
                    rect: smithay::utils::Rectangle::new(
                        frame.loc - camera - Point::from((0.0, height)),
                        (frame.size.w, height).into(),
                    ),
                    opacity,
                }
            }),
            chrome_animation: if snapshot.native_chrome {
                chrome_animation
            } else {
                animation
            },
            popup_animation,
            live_popups: (!popups_match).then(|| TilingLivePopups {
                root: root.into_owned(),
                origin: snapshot.blur.position.to_f64() - snapshot.blur.geometry.loc.to_f64(),
                geometry_loc: snapshot.blur.geometry.loc,
                opacity,
            }),
            elements,
            blur,
            border: snapshot.border.clone().map(|mut border| {
                if snapshot.native_chrome {
                    let bar = (f64::from(snapshot.blur.bar_height.unwrap_or(0)) * output_scale)
                        .round()
                        / output_scale;
                    border.inner = smithay::utils::Rectangle::new(
                        frame.loc - Point::from((0.0, bar)),
                        (frame.size.w, frame.size.h + bar).into(),
                    );
                }
                border.opacity *= f64::from(visual.alpha);
                border
            }),
            shadow: snapshot.shadow.clone().map(|mut shadow| {
                if snapshot.native_chrome {
                    let bw = f64::from(shadow.border_width);
                    let bar = (f64::from(snapshot.blur.bar_height.unwrap_or(0)) * output_scale)
                        .round()
                        / output_scale;
                    shadow.body = smithay::utils::Rectangle::new(
                        frame.loc - Point::from((bw, bar + bw)),
                        (frame.size.w + 2.0 * bw, frame.size.h + bar + 2.0 * bw).into(),
                    );
                }
                shadow.opacity *= f64::from(visual.alpha);
                shadow
            }),
        })
    }

    fn tiling_presentation_ready(&self, now: std::time::Instant) -> bool {
        let Some(presentation) = self
            .tiling_view
            .as_ref()
            .and_then(|v| v.presentation.as_ref())
        else {
            return true;
        };
        now >= presentation.deadline
            || presentation
                .requests
                .keys()
                .all(|id| !self.stage.is_tiled(*id))
    }

    pub(crate) fn tiling_buffer_committed(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        use smithay::wayland::compositor::{BufferAssignment, SurfaceAttributes, with_states};
        let Some(id) = self
            .stage
            .windows()
            .find(|w| w.wl_surface().as_deref() == Some(surface))
            .and_then(|window| self.stage.id_of(window))
        else {
            return;
        };
        if !self.select_tiling_output_for_window(id)
            || self
                .tiling_view
                .as_ref()
                .is_none_or(|view| view.presentation.is_none())
        {
            return;
        }
        let new_buffer = with_states(surface, |states| {
            matches!(
                states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .current()
                    .buffer,
                Some(BufferAssignment::NewBuffer(_))
            )
        });
        if !new_buffer {
            return;
        }
        let serial = self
            .stage
            .window_by_id(id)
            .and_then(|element| element.client())
            .and_then(|w| w.toplevel())
            .and_then(|t| t.with_cached_state(|s| s.last_acked.as_ref().map(|c| c.serial)));
        let Some(presentation) = self
            .tiling_view
            .as_mut()
            .and_then(|v| v.presentation.as_mut())
        else {
            return;
        };
        if let Some(serial) = serial
            && presentation
                .requests
                .get(&id)
                .is_some_and(|expected| serial >= *expected)
        {
            presentation.requests.remove(&id);
        }
    }

    fn wake_tiling_clients(&self, clients: Vec<smithay::reexports::wayland_server::Client>) {
        // Pre-commit hooks run while Smithay is building a transaction. Dispatch
        // released commits only after that stack has unwound.
        self.loop_handle.insert_idle(move |state| {
            for client in clients {
                if let Some(client_state) = client.get_data::<super::ClientState>() {
                    let dh = state.display_handle.clone();
                    client_state.compositor_state.blocker_cleared(state, &dh);
                }
            }
        });
    }

    fn settle_tiling_commits(&mut self, clients: Vec<smithay::reexports::wayland_server::Client>) {
        let Some(view) = self.tiling_view.as_mut() else {
            return;
        };
        let layout_id = view.layout_id;
        let output = view.output.clone();
        view.commit_wakeups += 1;
        self.loop_handle.insert_idle(move |state| {
            for client in clients {
                if let Some(client_state) = client.get_data::<super::ClientState>() {
                    let dh = state.display_handle.clone();
                    client_state.compositor_state.blocker_cleared(state, &dh);
                }
            }
            if state.tiling_view.select_output(&output) {
                state.stage.activate_tiling_output(&output.name());
                let should_refresh = if let Some(view) = state.tiling_view.as_mut()
                    && view.layout_id == layout_id
                {
                    view.commit_wakeups -= 1;
                    true
                } else {
                    false
                };
                if should_refresh {
                    state.refresh_tiling_for_output(&output, std::time::Instant::now());
                }
            }
        });
    }

    fn expire_tiling_commits(&mut self, now: std::time::Instant) {
        let Some(commits) = self.tiling_view.as_mut().and_then(|v| v.commits.as_mut()) else {
            return;
        };
        let mut released = commits.group.expire(now);
        let dead: Vec<_> = commits
            .group
            .pending_requests()
            .keys()
            .copied()
            .filter(|id| !self.stage.is_tiled(*id) || self.stage.window_by_id(*id).is_none())
            .collect();
        for id in dead {
            released |= commits.group.remove(id);
        }
        if released {
            let commits = self.tiling_view.as_mut().unwrap().commits.take().unwrap();
            self.settle_tiling_commits(commits.clients);
        }
    }

    pub(crate) fn arm_late_tiling_resize(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        use smithay::wayland::compositor::{BufferAssignment, SurfaceAttributes, with_states};
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        if self.backend.is_none() || self.config.effects.animation_speed >= 1.0 {
            return;
        }
        let serial = with_states(surface, |states| {
            if !matches!(
                states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .pending()
                    .buffer,
                Some(BufferAssignment::NewBuffer(_))
            ) {
                return None;
            }
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()?
                .lock()
                .unwrap()
                .last_acked
                .as_ref()
                .map(|c| c.serial)
        });
        let Some(serial) = serial else {
            return;
        };
        let Some(window) = self.window_for_surface(surface) else {
            return;
        };
        let Some(id) = self
            .stage
            .id_of(&window)
            .filter(|id| self.stage.is_tiled(*id))
        else {
            return;
        };
        if !self.select_tiling_output_for_window(id)
            || self
                .tiling_view
                .as_ref()
                .is_none_or(|view| view.commits.is_some() || view.presentation.is_some())
        {
            return;
        }
        if self.window_animations.generation_of(id).is_some()
            || self.window_animations.open_progress(id).is_some()
            || self.resize_crossfades.contains_key(&id)
        {
            return;
        }
        if let Some(size) = self
            .tiling_view
            .as_mut()
            .unwrap()
            .configures
            .take_late_resize(id, serial)
        {
            // This reuses the bounded start hold and pre-commit content capture;
            // it sends no configure and never reopens the expired layout group.
            self.animate_window_geometry(&window, size, None);
        }
    }

    pub(crate) fn handle_tiling_unmap(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> bool {
        use smithay::backend::renderer::utils::RendererSurfaceStateUserData;
        use smithay::wayland::compositor::with_states;
        if self.tiling_view.is_none() {
            return false;
        }
        let Some(window) = self.window_for_surface(surface) else {
            return false;
        };
        let Some(id) = self
            .stage
            .id_of(&window)
            .filter(|id| self.stage.is_tiled(*id))
        else {
            return false;
        };
        if !self.select_tiling_output_for_window(id) {
            return false;
        }
        let mapped = with_states(surface, |states| {
            states
                .data_map
                .get::<RendererSurfaceStateUserData>()
                .is_some_and(|data| data.lock().unwrap().buffer().is_some())
        });
        if mapped {
            return false;
        }
        window.on_commit();
        self.stage.untile(id);
        self.release_tiling_snapshot_for_animation(id);
        self.window_animations.remove(id);
        self.drop_resize_crossfade(id);
        self.render
            .animated_content
            .retain(|(_, candidate), _| *candidate != id);
        self.decorations
            .remove(&crate::decorations::DecorationKey::Surface(surface.id()));
        self.pending_ssd.remove(&surface.id());
        self.pending_center.insert(surface.clone());
        super::membership::send_output_leaves(&window);
        if self
            .focused_element()
            .as_ref()
            .and_then(|w| self.stage.id_of(w))
            == Some(id)
        {
            self.set_window_focus(None, SERIAL_COUNTER.next_serial());
        }
        self.mark_all_dirty();
        true
    }

    pub(crate) fn tiling_pre_commit(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        use smithay::wayland::compositor::{
            BufferAssignment, SurfaceAttributes, add_blocker, with_states,
        };
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        let Some(id) = self
            .stage
            .windows()
            .find(|w| w.wl_surface().as_deref() == Some(surface))
            .and_then(|w| self.stage.id_of(w))
        else {
            return;
        };
        if !self.select_tiling_output_for_window(id)
            || self
                .tiling_view
                .as_ref()
                .is_none_or(|view| view.commits.is_none())
        {
            return;
        }
        let serial = with_states(surface, |states| {
            let new_buffer = matches!(
                states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .pending()
                    .buffer,
                Some(BufferAssignment::NewBuffer(_))
            );
            if !new_buffer {
                return None;
            }
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()?
                .lock()
                .unwrap()
                .last_acked
                .as_ref()
                .map(|c| c.serial)
        });
        let Some(serial) = serial else { return };
        let now = std::time::Instant::now();
        self.expire_tiling_commits(now);
        let Some(commits) = self.tiling_view.as_mut().and_then(|v| v.commits.as_mut()) else {
            return;
        };
        if !commits.group.accepts(id, serial) {
            return;
        }
        add_blocker(surface, commits.group.blocker());
        if commits.group.arrive(commits.generation, id, serial, now) {
            let commits = self.tiling_view.as_mut().unwrap().commits.take().unwrap();
            self.settle_tiling_commits(commits.clients);
        }
    }

    fn arm_tiling_configure_deadline(&mut self) {
        use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
        let Some(view) = self.tiling_view.as_mut() else {
            return;
        };
        let initial_deadline = view
            .initial_configures
            .values()
            .filter(|c| !c.expired)
            .map(|c| c.sent_at + scrolling::transaction::CONFIGURE_TIMEOUT)
            .min();
        let Some(deadline) = view
            .configures
            .next_deadline()
            .into_iter()
            .chain(initial_deadline)
            .chain(view.commits.as_ref().map(|c| c.group.deadline()))
            .chain(
                view.presentation
                    .as_ref()
                    .filter(|p| p.deadline > std::time::Instant::now())
                    .map(|p| p.deadline),
            )
            .min()
        else {
            if let Some(timer) = view.configure_timer.take() {
                self.loop_handle.remove(timer);
            }
            return;
        };
        if view.configure_timer.is_some() {
            return;
        }
        let layout_id = view.layout_id;
        let output = view.output.clone();
        match self
            .loop_handle
            .insert_source(Timer::from_deadline(deadline), move |_, _, state| {
                if state.tiling_view.select_output(&output) {
                    state.stage.activate_tiling_output(&output.name());
                    let should_refresh = if let Some(view) = state.tiling_view.as_mut()
                        && view.layout_id == layout_id
                    {
                        view.configure_timer = None;
                        true
                    } else {
                        false
                    };
                    if should_refresh {
                        state.refresh_tiling_for_output(&output, std::time::Instant::now());
                    }
                }
                TimeoutAction::Drop
            }) {
            Ok(token) => view.configure_timer = Some(token),
            Err(error) => {
                tracing::warn!(?error, "Cannot schedule tiling configure deadline");
                view.presentation = None;
                if let Some(mut commits) = view.commits.take() {
                    commits.group.cancel();
                    self.settle_tiling_commits(commits.clients);
                }
            }
        }
    }

    pub(crate) fn tiling_pending_configures(&self) -> usize {
        self.tiling_view
            .iter()
            .map(|(_, view)| {
                view.configures.pending_count()
                    + view
                        .initial_configures
                        .values()
                        .filter(|c| !c.expired)
                        .count()
            })
            .sum()
    }

    #[cfg(test)]
    pub(crate) fn tiling_configure_progress(
        &self,
        id: driftline::stage::ElementId,
    ) -> Option<scrolling::transaction::ConfigureProgress> {
        let output = self.stage.tiling_output_for_window(id)?;
        let output = self.output_by_name(output)?;
        self.tiling_view
            .for_output(&output)?
            .configures
            .progress(id)
    }

    pub(crate) fn tiling_canvas_view(&self, output: &Output) -> Option<(Point<f64, Logical>, f64)> {
        self.tiling_view
            .for_output(output)
            .map(|view| (view.camera, view.zoom))
    }
}
