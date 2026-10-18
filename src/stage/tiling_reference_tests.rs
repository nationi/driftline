use std::collections::HashMap;

use serde::Deserialize;
use smithay::utils::Rectangle;

use super::{Stage, mock::TestWindow};
use crate::canvas::Chrome;
use crate::layout::scrolling::{self, Column, ColumnWidth, Constraints, Tile};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    revision: String,
    provenance: String,
    traces: Vec<Trace>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Trace {
    name: String,
    size: [i32; 2],
    width: i32,
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    #[serde(flatten)]
    operation: Operation,
    columns: Vec<Vec<u64>>,
    selected: Option<u64>,
    frames: Vec<[i64; 5]>,
    camera: Option<[i32; 2]>,
    target: Option<[i32; 2]>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Operation {
    Map { id: u64 },
    Focus { id: u64 },
    Close { id: u64 },
    Consume,
    Expel,
    MoveTile { backward: bool },
    MoveColumn { backward: bool },
    SetWidth { width: i32 },
    Resize { size: [i32; 2] },
}

#[test]
fn pinned_niri_geometry_and_fit_target_traces() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/niri-scrolling.json")).unwrap();
    assert_eq!(corpus.revision, "12de51d6a60baad8896366920278a24f17921f52");
    assert_eq!(
        corpus.provenance,
        "source-derived; not captured from a running niri compositor"
    );
    assert!(!corpus.traces.is_empty());
    for trace in corpus.traces {
        let mut stage = Stage::<TestWindow>::new();
        let mut windows = HashMap::new();
        let mut names = HashMap::new();
        let mut size = (trace.size[0], trace.size[1]).into();
        for (index, step) in trace.steps.iter().enumerate() {
            let context = format!("{} step {index}: {:?}", trace.name, step.operation);
            match step.operation {
                Operation::Map { id } => {
                    let window = TestWindow::new(id);
                    assert!(windows.insert(id, window.clone()).is_none(), "{context}");
                    stage.map(window.clone(), (0, 0).into());
                    let element_id = stage.id_of(&window).unwrap();
                    names.insert(element_id, id);
                    assert!(stage.tile(element_id, (100, 100).into()), "{context}");
                    let active = stage.active_column().unwrap();
                    stage
                        .tiling
                        .columns
                        .iter_mut()
                        .find(|c| c.id == active)
                        .unwrap()
                        .width = ColumnWidth::Fixed(trace.width);
                }
                Operation::Focus { id } => {
                    assert!(
                        stage.select_tile(stage.id_of(&windows[&id]).unwrap()),
                        "{context}"
                    );
                }
                Operation::Close { id } => {
                    stage.remove(&windows[&id]);
                }
                Operation::Consume => {
                    assert!(stage.consume_window_into_column(), "{context}");
                }
                Operation::Expel => {
                    assert!(stage.expel_window_from_column(), "{context}");
                }
                Operation::MoveTile { backward } => {
                    assert!(stage.move_tile(backward), "{context}");
                }
                Operation::MoveColumn { backward } => {
                    assert!(stage.move_column(backward), "{context}");
                }
                Operation::SetWidth { width } => {
                    let active = stage.active_column().unwrap();
                    stage
                        .tiling
                        .columns
                        .iter_mut()
                        .find(|c| c.id == active)
                        .unwrap()
                        .width = ColumnWidth::Fixed(width);
                }
                Operation::Resize { size: next } => {
                    size = (next[0], next[1]).into();
                }
            }
            stage.verify_invariants();
            let topology: Vec<Vec<_>> = stage
                .tiled_columns()
                .iter()
                .map(|column| column.windows.iter().map(|tile| names[&tile.id]).collect())
                .collect();
            assert_eq!(topology, step.columns, "{context}");
            assert_eq!(
                stage.active_tile().map(|id| names[&id]),
                step.selected,
                "{context}"
            );
            let columns: Vec<_> = stage
                .tiled_columns()
                .iter()
                .map(|column| Column {
                    width: column.width,
                    tiles: column
                        .windows
                        .iter()
                        .map(|tile| Tile {
                            id: tile.id,
                            constraints: Constraints::default(),
                            chrome: Chrome::NONE,
                        })
                        .collect(),
                })
                .collect();
            let placements = scrolling::arrange_columns(&columns, (0, 0).into(), size, 0).unwrap();
            let frames: Vec<_> = placements
                .iter()
                .map(|p| {
                    [
                        i64::try_from(names[&p.id]).unwrap(),
                        i64::from(p.frame.loc.x),
                        i64::from(p.frame.loc.y),
                        i64::from(p.frame.size.w),
                        i64::from(p.frame.size.h),
                    ]
                })
                .collect();
            assert_eq!(frames, step.frames, "{context}");
            match (step.camera, step.target) {
                (Some(camera), Some(target)) => {
                    let frame = placements
                        .iter()
                        .find(|p| Some(p.id) == stage.active_tile())
                        .unwrap()
                        .frame;
                    let actual = scrolling::reveal(
                        frame,
                        Rectangle::from_size(size),
                        (camera[0], camera[1]).into(),
                    )
                    .unwrap();
                    assert_eq!([actual.x, actual.y], target, "{context}");
                }
                (None, None) => (),
                _ => panic!("camera input and target must be paired: {context}"),
            }
        }
    }
}
