use std::collections::HashMap;

use driftline::protocols::output_management::{ModeInfo, OutputHeadState, notify_changes};
use smithay::utils::Transform;
use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_head_v1::{self, ZwlrOutputHeadV1},
    zwlr_output_manager_v1::{self, ZwlrOutputManagerV1},
    zwlr_output_mode_v1::{self, ZwlrOutputModeV1},
};

use super::Fixture;
use super::client::ClientId;
use crate::state::Driftline;

#[derive(Default)]
struct Observer {
    heads: Vec<ZwlrOutputHeadV1>,
    modes: Vec<ZwlrOutputModeV1>,
    current: Option<ZwlrOutputModeV1>,
    finished_heads: usize,
    finished_modes: usize,
    done: usize,
}

impl Dispatch<wl_registry::WlRegistry, ()> for Observer {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "zwlr_output_manager_v1"
        {
            registry.bind::<ZwlrOutputManagerV1, _, _>(name, version.min(4), qh, ());
        }
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for Observer {
    fn event(
        state: &mut Self,
        _: &ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => state.heads.push(head),
            zwlr_output_manager_v1::Event::Done { .. } => state.done += 1,
            _ => (),
        }
    }
    wayland_client::event_created_child!(Observer, ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<ZwlrOutputHeadV1, ()> for Observer {
    fn event(
        state: &mut Self,
        _: &ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_head_v1::Event::Mode { mode } => state.modes.push(mode),
            zwlr_output_head_v1::Event::CurrentMode { mode } => state.current = Some(mode),
            zwlr_output_head_v1::Event::Finished => state.finished_heads += 1,
            _ => (),
        }
    }
    wayland_client::event_created_child!(Observer, ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<ZwlrOutputModeV1, ()> for Observer {
    fn event(
        state: &mut Self,
        _: &ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_output_mode_v1::Event::Finished = event {
            state.finished_modes += 1;
        }
    }
}

fn snapshot(current: usize, extra_mode: bool) -> HashMap<String, OutputHeadState> {
    let mut modes: Vec<_> = [60_000, 180_000]
        .into_iter()
        .map(|refresh| ModeInfo {
            width: 1920,
            height: 1080,
            refresh,
            preferred: refresh == 60_000,
        })
        .collect();
    if extra_mode {
        modes.push(ModeInfo {
            width: 1280,
            height: 720,
            refresh: 60_000,
            preferred: false,
        });
    }
    HashMap::from([(
        "HDMI-A-1".into(),
        OutputHeadState {
            name: "HDMI-A-1".into(),
            description: "Test display".into(),
            make: "Test".into(),
            model: "Display".into(),
            serial_number: "1".into(),
            physical_size: (500, 300),
            modes,
            current_mode_index: Some(current),
            position: (0, 0),
            transform: Transform::Normal,
            scale: 1.0,
        },
    )])
}

fn update(f: &mut Fixture, state: HashMap<String, OutputHeadState>) {
    notify_changes::<Driftline>(&mut f.state().output_management_state, state);
}

fn pump(f: &mut Fixture, id: ClientId, queue: &mut EventQueue<Observer>, observer: &mut Observer) {
    for _ in 0..3 {
        f.double_roundtrip(id);
        queue.dispatch_pending(observer).unwrap();
    }
}

fn connect(f: &mut Fixture) -> (ClientId, EventQueue<Observer>, Observer) {
    let id = f.add_client();
    let connection = f.client(id).connection.clone();
    let mut queue = connection.new_event_queue();
    connection.display().get_registry(&queue.handle(), ());
    let mut observer = Observer::default();
    pump(f, id, &mut queue, &mut observer);
    assert_eq!(observer.heads.len(), 1);
    assert_eq!(observer.modes.len(), 2);
    (id, queue, observer)
}

#[test]
fn refresh_change_preserves_monitor_and_mode_objects() {
    let mut f = Fixture::new();
    update(&mut f, snapshot(0, false));
    let (id, mut queue, mut observer) = connect(&mut f);
    let fast_mode = observer.modes[1].clone();
    update(&mut f, snapshot(1, false));
    pump(&mut f, id, &mut queue, &mut observer);
    assert_eq!(observer.current, Some(fast_mode));
    assert_eq!(observer.heads.len(), 1);
    assert_eq!(observer.modes.len(), 2);
    assert_eq!((observer.finished_heads, observer.finished_modes), (0, 0));
    assert_eq!(observer.done, 2);
    update(&mut f, snapshot(1, false));
    pump(&mut f, id, &mut queue, &mut observer);
    assert_eq!(
        observer.done, 2,
        "unchanged outputs must not invalidate configurations"
    );
}

#[test]
fn changing_available_modes_keeps_the_monitor_object() {
    let mut f = Fixture::new();
    update(&mut f, snapshot(0, false));
    let (id, mut queue, mut observer) = connect(&mut f);
    update(&mut f, snapshot(1, true));
    pump(&mut f, id, &mut queue, &mut observer);
    assert_eq!(observer.heads.len(), 1);
    assert_eq!(observer.finished_heads, 0);
    assert_eq!(observer.finished_modes, 2);
    assert_eq!(observer.modes.len(), 5);
    assert_eq!(observer.current, Some(observer.modes[3].clone()));
}

#[test]
fn releasing_disconnected_monitor_does_not_forget_its_replacement() {
    let mut f = Fixture::new();
    update(&mut f, snapshot(0, false));
    let (id, mut queue, mut observer) = connect(&mut f);
    let old_head = observer.heads[0].clone();
    update(&mut f, HashMap::new());
    update(&mut f, snapshot(0, false));
    pump(&mut f, id, &mut queue, &mut observer);
    old_head.release();
    pump(&mut f, id, &mut queue, &mut observer);
    update(&mut f, snapshot(1, false));
    pump(&mut f, id, &mut queue, &mut observer);
    assert_eq!(observer.heads.len(), 2);
    assert_eq!(observer.finished_heads, 1);
    assert_eq!(observer.modes.len(), 4);
    assert_eq!(observer.current, Some(observer.modes[3].clone()));
}
