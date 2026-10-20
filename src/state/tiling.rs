use smithay::output::Output;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Logical, Point, SERIAL_COUNTER};
use smithay::wayland::seat::WaylandFocus;

use driftwm::config::Direction;
use driftwm::layout::scrolling::{self, Column, Constraints, Tile};
use driftwm::window_ext::WindowExt;

use super::{DriftWm, StageWindow, configured_window_size, output_state, visual_frame_center};
use crate::grabs::SizeConstraints;

pub(crate) struct TilingView {
    layout_id: driftwm::stage::TilingLayoutId,
    output: Output,
    camera: Point<f64, Logical>,
    zoom: f64,
    origin: Point<i32, Logical>,
    last_usable: Option<smithay::utils::Rectangle<i32, Logical>>,
    anchor: Option<(driftwm::stage::ColumnId, i32)>,
    close_return: Option<CloseReturnView>,
    configures: scrolling::transaction::ConfigureBatch,
    commits: Option<TilingCommits>,
    commit_wakeups: usize,
    presentation: Option<TilingPresentation>,
    snapshots: std::collections::HashMap<driftwm::stage::ElementId, TilingSnapshot>,
    pending_close_return: Option<CloseReturnView>,
    selected: Option<driftwm::stage::ElementId>,
    configure_timer: Option<smithay::reexports::calloop::RegistrationToken>,
    initial_configures:
        std::collections::HashMap<driftwm::stage::ElementId, InitialTilingConfigure>,
    revealed: Option<(
        driftwm::stage::ElementId,
        smithay::utils::Rectangle<i32, Logical>,
    )>,
}

struct TilingSnapshot {
    image: crate::render::ClosingSnapshot,
    held_geometry: Option<smithay::utils::Rectangle<f64, Logical>>,
    resize: Option<crate::render::ResizeCrossfade>,
    popups: Vec<TilingPopupSnapshot>,
    blur: crate::render::SnapshotBlur,
    shadow: Option<crate::render::SnapshotShadow>,
    border: Option<crate::render::SnapshotBorder>,
    opacity: f64,
    native_chrome: bool,
    bar: Option<smithay::backend::renderer::element::memory::MemoryRenderBuffer>,
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
    pub buffer: smithay::backend::renderer::element::memory::MemoryRenderBuffer,
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
    requests: std::collections::HashMap<driftwm::stage::ElementId, smithay::utils::Serial>,
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
    opened: driftwm::stage::ColumnId,
    previous: driftwm::stage::ColumnId,
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

impl DriftWm {
    pub(crate) fn toggle_tiling(&mut self) {
        if self.tiling_view.is_some() {
            self.exit_tiling();
            return;
        }
        let Some(output) = self.active_output() else {
            return;
        };
        if self.space.outputs().count() != 1
            || self.is_fullscreen()
            || !self.interactive_move.is_empty()
        {
            tracing::warn!(
                "Scrolling tiling currently requires one output and no active move or fullscreen"
            );
            return;
        }
        self.cancel_animations_everywhere();
        let camera = self.camera();
        let zoom = self.zoom();
        let usable = self.get_usable_area();
        let camera_origin: Point<i32, Logical> = camera.to_i32_round();
        let (Ok(x), Ok(y)) = (
            i32::try_from(i64::from(camera_origin.x) + i64::from(usable.loc.x)),
            i32::try_from(i64::from(camera_origin.y) + i64::from(usable.loc.y)),
        ) else {
            return;
        };
        let origin = Point::from((x, y));
        let layout_id = self.stage.begin_tiling();
        self.tiling_view = Some(TilingView {
            layout_id,
            output,
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
        });
        self.set_zoom(1.0);
        self.refresh_tiling();
    }

    pub(super) fn exit_tiling(&mut self) {
        let Some(mut view) = self.tiling_view.take() else {
            return;
        };
        if let Some(mut commits) = view.commits.take() {
            commits.group.cancel();
            self.wake_tiling_clients(commits.clients);
        }
        if let Some(timer) = view.configure_timer {
            self.loop_handle.remove(timer);
        }
        for (id, _) in view.initial_configures {
            if !self.stage.is_tiled(id)
                && let Some(window) = self.stage.window_by_id(id).and_then(|w| w.client())
                && window
                    .wl_surface()
                    .is_some_and(|s| self.pending_center.contains(&*s))
                && let Some(toplevel) = window.toplevel()
            {
                // No buffer has supplied a canvas size yet; let the client
                // choose instead of leaving an obsolete tiling offer pending.
                toplevel.with_pending_state(|s| s.size = None);
                toplevel.send_configure();
            }
        }
        self.cancel_animations_everywhere();
        for column in self.stage.clear_tiling() {
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
            );
        }
        {
            let mut os = output_state(&view.output);
            os.camera = view.camera;
            os.zoom = view.zoom;
        }
        self.update_output_from_camera();
        self.render.blur_geometry_generation += 1;
        self.mark_all_dirty();
        self.session_store_mark_dirty();
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
            self.stage.select_tile(id);
            self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            self.refresh_tiling();
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
        decoration: &driftwm::config::DecorationMode,
        rule: Option<&driftwm::config::AppliedWindowRule>,
    ) -> Option<smithay::utils::Size<i32, Logical>> {
        use smithay::wayland::compositor::with_states;
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        let view = self.tiling_view.as_ref()?;
        let surface = window.wl_surface()?;
        if self.space.outputs().count() != 1
            || self.active_output().as_ref() != Some(&view.output)
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
        scrolling::arrange_columns(&[column], (0, 0).into(), self.get_usable_area().size, 0)
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
        if self
            .tiling_view
            .as_ref()
            .is_some_and(|view| &view.output == output)
            && self.tiling_eligible(&StageWindow::Client(window.clone()))
        {
            // Reflow owns the camera target; a new window's temporary canvas
            // placement must not overwrite the stationary tiling view.
            self.raise_and_focus(window, serial);
            return true;
        }
        false
    }

    pub(crate) fn refresh_tiling(&mut self) {
        self.refresh_tiling_at(std::time::Instant::now());
    }

    pub(crate) fn refresh_tiling_at(&mut self, now: std::time::Instant) {
        let Some(view) = self.tiling_view.as_ref() else {
            return;
        };
        if self.stage.tiling_layout_id() != Some(view.layout_id)
            || self.space.outputs().count() != 1
            || self.active_output().as_ref() != Some(&view.output)
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

        let selected_closed = view
            .selected
            .is_some_and(|id| self.stage.window_by_id(id).is_none());
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

        if self.stage.tiled_windows().any(|column| {
            self.stage
                .window_by_id(column.id)
                .is_some_and(|w| !self.tiling_eligible(w))
        }) {
            self.prepare_tiling_operation(super::TilingOperation::LayoutInvalidated);
            return;
        }
        let candidates: Vec<_> = self
            .stage
            .windows()
            .filter(|w| self.tiling_eligible(w))
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
        if placements.iter().any(|p| {
            self.stage
                .window_by_id(p.id)
                .and_then(|w| w.client())
                .is_some_and(|w| configured_window_size(w) != p.content.size)
        }) {
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
        if let Some(view) = &mut self.tiling_view {
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
        if self
            .tiling_view
            .as_ref()
            .is_some_and(|v| v.commits.is_some() || v.commit_wakeups != 0)
            || !self.tiling_presentation_ready(now)
        {
            return;
        }
        let view = self.tiling_view.as_mut().unwrap();
        view.presentation = None;
        let snapshots = std::mem::take(&mut view.snapshots);
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
                && !self.window_animations.has_open_fade(id)
                && self.window_animations.generation_of(id).is_some()
                && let Some(mut resize) = snapshot.resize.take()
            {
                if !snapshot.blur.window_blur
                    && snapshot
                        .blur
                        .client_regions
                        .as_ref()
                        .is_none_or(|r| r.is_empty())
                {
                    resize.preserve_initial_pixels();
                }
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
            self.set_camera_target(Some(target.to_f64()));
        }
        if let Some(view) = &mut self.tiling_view {
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
        let tiles: Vec<_> = self.stage.tiled_windows().map(|t| t.id).collect();
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
            let blur = crate::render::SnapshotBlur {
                surface_id: surface.id(),
                position,
                geometry,
                bar_height: self
                    .decorations
                    .contains_key(&crate::decorations::DecorationKey::Surface(surface.id()))
                    .then_some(self.config.decorations.title_bar_height),
                window_blur: driftwm::config::effective_blur(
                    driftwm::config::applied_rule(&surface).as_ref(),
                    &self.config.decorations,
                    false,
                ),
                client_regions: smithay::wayland::compositor::with_states(
                    &surface,
                    crate::handlers::background_effect::get_cached_blur_region,
                ),
            };
            let native_chrome = crossfading
                && !blur.window_blur
                && blur.client_regions.as_ref().is_none_or(|r| r.is_empty())
                && self.window_animations.open_progress(id).is_none();
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
            let bar = native_chrome
                .then(|| chrome.and_then(|c| c.bar.map(|(b, _)| b.clone())))
                .flatten();
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
                    let resize = (self.config.effects.animation_speed < 1.0)
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
                    self.tiling_view.as_mut().unwrap().snapshots.insert(
                        id,
                        TilingSnapshot {
                            image,
                            held_geometry,
                            resize,
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

    pub(crate) fn release_tiling_snapshot_for_animation(&mut self, id: driftwm::stage::ElementId) {
        if self
            .tiling_view
            .as_mut()
            .is_some_and(|view| view.snapshots.remove(&id).is_some())
        {
            // Live animation geometry must replace the saved frost mask too.
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
    }

    pub(crate) fn invalidate_tiling_snapshot_animation(&mut self, id: driftwm::stage::ElementId) {
        if self
            .tiling_view
            .as_ref()
            .is_some_and(|v| v.snapshots.contains_key(&id))
        {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
    }

    pub(crate) fn has_fading_tiling_snapshot(&self) -> bool {
        self.tiling_view.as_ref().is_some_and(|v| {
            v.snapshots
                .keys()
                .any(|id| self.window_animations.open_progress(*id).is_some())
        })
    }

    pub(crate) fn tiling_snapshot_count(&self) -> usize {
        self.tiling_view.as_ref().map_or(0, |v| {
            v.snapshots.values().map(|s| 1 + s.popups.len()).sum()
        })
    }

    pub(crate) fn tiling_snapshot_elements(
        &self,
        id: driftwm::stage::ElementId,
        output: &Output,
        camera: Point<f64, Logical>,
        zoom: f64,
        output_scale: f64,
    ) -> Option<TilingSnapshotElements> {
        let snapshot = self.tiling_view.as_ref()?.snapshots.get(&id)?;
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
        let visual = self.window_animations.opening_visual(
            id,
            snapshot.held_geometry.unwrap_or_else(|| {
                smithay::utils::Rectangle::new(base.loc + displacement, base.size)
            }),
            self.config.effects.animation_scale,
        );
        let physical_scale = zoom * output_scale;
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
            elements.push(snapshot.image.render_retained_rect(
                smithay::utils::Rectangle::new(
                    (visual.loc - camera).to_physical(physical_scale),
                    visual.size.to_physical(physical_scale),
                ),
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
        let blur = snapshot
            .blur
            .request(camera, zoom, output_scale, elements.len())
            .map(|mut request| {
                // Transform already snapped mask geometry and client regions
                // with the same affine as the retained content.
                if let Some(animation) = animation {
                    request.apply_animation(animation);
                }
                request
            });
        Some(TilingSnapshotElements {
            bar: snapshot.bar.as_ref().map(|buffer| {
                let height = (f64::from(snapshot.blur.bar_height.unwrap_or(0)) * output_scale)
                    .round()
                    / output_scale;
                TilingSnapshotBar {
                    buffer: buffer.clone(),
                    rect: smithay::utils::Rectangle::new(
                        visual.loc - camera - Point::from((0.0, height)),
                        (visual.size.w, height).into(),
                    ),
                    opacity,
                }
            }),
            chrome_animation: if snapshot.native_chrome {
                None
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
                    let bar = base.loc.y - border.inner.loc.y;
                    border.inner = smithay::utils::Rectangle::new(
                        visual.loc - Point::from((0.0, bar)),
                        (visual.size.w, visual.size.h + bar).into(),
                    );
                }
                border.opacity *= f64::from(visual.alpha);
                border
            }),
            shadow: snapshot.shadow.clone().map(|mut shadow| {
                if snapshot.native_chrome {
                    let bw = f64::from(shadow.border_width);
                    let bar = base.loc.y - shadow.body.loc.y - bw;
                    shadow.body = smithay::utils::Rectangle::new(
                        visual.loc - Point::from((bw, bar + bw)),
                        (visual.size.w + 2.0 * bw, visual.size.h + bar + 2.0 * bw).into(),
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
        if self
            .tiling_view
            .as_ref()
            .is_none_or(|v| v.presentation.is_none())
        {
            return;
        }
        let Some(element) = self
            .stage
            .windows()
            .find(|w| w.wl_surface().as_deref() == Some(surface))
        else {
            return;
        };
        let Some(id) = self.stage.id_of(element) else {
            return;
        };
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
        let serial = element
            .client()
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
        view.commit_wakeups += 1;
        self.loop_handle.insert_idle(move |state| {
            for client in clients {
                if let Some(client_state) = client.get_data::<super::ClientState>() {
                    let dh = state.display_handle.clone();
                    client_state.compositor_state.blocker_cleared(state, &dh);
                }
            }
            if let Some(view) = state.tiling_view.as_mut()
                && view.layout_id == layout_id
            {
                view.commit_wakeups -= 1;
                state.refresh_tiling();
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
        if self.backend.is_none()
            || self.config.effects.animation_speed >= 1.0
            || self
                .tiling_view
                .as_ref()
                .is_none_or(|v| v.commits.is_some() || v.presentation.is_some())
        {
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

    pub(crate) fn tiling_pre_commit(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        use smithay::wayland::compositor::{
            BufferAssignment, SurfaceAttributes, add_blocker, with_states,
        };
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        if self
            .tiling_view
            .as_ref()
            .is_none_or(|v| v.commits.is_none())
        {
            return;
        }
        let Some(id) = self
            .stage
            .windows()
            .find(|w| w.wl_surface().as_deref() == Some(surface))
            .and_then(|w| self.stage.id_of(w))
        else {
            return;
        };
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
        match self
            .loop_handle
            .insert_source(Timer::from_deadline(deadline), move |_, _, state| {
                if let Some(view) = state.tiling_view.as_mut()
                    && view.layout_id == layout_id
                {
                    view.configure_timer = None;
                    state.refresh_tiling();
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
        self.tiling_view.as_ref().map_or(0, |v| {
            v.configures.pending_count()
                + v.initial_configures.values().filter(|c| !c.expired).count()
        })
    }

    #[cfg(test)]
    pub(crate) fn tiling_configure_progress(
        &self,
        id: driftwm::stage::ElementId,
    ) -> Option<scrolling::transaction::ConfigureProgress> {
        self.tiling_view.as_ref()?.configures.progress(id)
    }

    pub(super) fn tiling_canvas_view(&self, output: &Output) -> Option<(Point<f64, Logical>, f64)> {
        self.tiling_view
            .as_ref()
            .filter(|view| &view.output == output)
            .map(|view| (view.camera, view.zoom))
    }
}
