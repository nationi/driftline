use smithay::utils::{Logical, Rectangle, Size};

use crate::layout::scrolling::ColumnWidth;

use super::{ElementId, Stage, StageElement};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TiledWindow {
    pub id: ElementId,
    pub restore: Rectangle<i32, Logical>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnId(u64);

/// Identity of one tiling session, including an empty strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TilingLayoutId(u64);

#[derive(Clone, Debug, PartialEq)]
pub struct TiledColumn {
    pub id: ColumnId,
    pub selected: ElementId,
    pub width: ColumnWidth,
    pub windows: Vec<TiledWindow>,
}

#[derive(Default)]
pub(super) struct Tiling {
    pub(super) columns: Vec<TiledColumn>,
    pub(super) active: Option<ColumnId>,
    next_column_id: u64,
    pub(super) restore_previous: Option<ColumnId>,
    pending_close_return: Option<(ColumnId, ColumnId)>,
    layout_id: Option<TilingLayoutId>,
    next_layout_id: u64,
}

impl<W: StageElement> Stage<W> {
    pub fn tiling_layout_id(&self) -> Option<TilingLayoutId> {
        self.tiling.layout_id
    }

    /// Begin a layout once; removing its last window does not end its lifetime.
    pub fn begin_tiling(&mut self) -> TilingLayoutId {
        if let Some(id) = self.tiling.layout_id {
            return id;
        }
        let id = TilingLayoutId(self.tiling.next_layout_id);
        self.tiling.next_layout_id = self
            .tiling
            .next_layout_id
            .checked_add(1)
            .expect("tiling layout ID space exhausted");
        self.tiling.layout_id = Some(id);
        id
    }

    pub fn tiled_columns(&self) -> &[TiledColumn] {
        &self.tiling.columns
    }

    pub fn close_return_column(&self) -> Option<ColumnId> {
        self.tiling.restore_previous
    }

    /// Take the (closed column, previous column) pair once after a close return.
    pub fn take_tiling_close_return(&mut self) -> Option<(ColumnId, ColumnId)> {
        self.tiling.pending_close_return.take()
    }

    pub fn active_column(&self) -> Option<ColumnId> {
        self.tiling.active
    }

    pub fn column(&self, id: ColumnId) -> Option<&TiledColumn> {
        self.tiling.columns.iter().find(|c| c.id == id)
    }

    pub fn active_tile(&self) -> Option<ElementId> {
        self.column(self.tiling.active?).map(|c| c.selected)
    }

    pub fn column_of(&self, window: ElementId) -> Option<ColumnId> {
        self.column_index(window)
            .map(|index| self.tiling.columns[index].id)
    }

    fn next_column_id(&mut self) -> ColumnId {
        let id = ColumnId(self.tiling.next_column_id);
        self.tiling.next_column_id = self
            .tiling
            .next_column_id
            .checked_add(1)
            .expect("column ID space exhausted");
        id
    }

    pub fn tiled_windows(&self) -> impl Iterator<Item = &TiledWindow> {
        self.tiling.columns.iter().flat_map(|c| &c.windows)
    }

    pub fn is_tiled(&self, id: ElementId) -> bool {
        self.tiled_windows().any(|w| w.id == id)
    }

    fn column_index(&self, id: ElementId) -> Option<usize> {
        self.tiling
            .columns
            .iter()
            .position(|c| c.windows.iter().any(|w| w.id == id))
    }

    /// Insert after the selected column, saving the original canvas rectangle once.
    pub fn tile(&mut self, id: ElementId, restore_size: Size<i32, Logical>) -> bool {
        if self.is_tiled(id) || restore_size.w <= 0 || restore_size.h <= 0 {
            return false;
        }
        let Some(entry) = self.entries.iter().find(|entry| entry.id == id) else {
            return false;
        };
        let restore = Rectangle::new(entry.position, restore_size);
        self.begin_tiling();
        let column_id = self.next_column_id();
        let column = TiledColumn {
            id: column_id,
            selected: id,
            width: ColumnWidth::default(),
            windows: vec![TiledWindow { id, restore }],
        };
        let index = self
            .tiling
            .active
            .and_then(|id| self.tiling.columns.iter().position(|c| c.id == id))
            .map_or(self.tiling.columns.len(), |i| i + 1);
        self.tiling.pending_close_return = None;
        self.tiling.restore_previous = self.tiling.active;
        self.tiling.columns.insert(index, column);
        self.tiling.active = Some(column_id);
        true
    }

    pub fn select_tile(&mut self, id: ElementId) -> bool {
        let Some(index) = self.column_index(id) else {
            return false;
        };
        if self.tiling.active != Some(self.tiling.columns[index].id) {
            self.tiling.restore_previous = None;
            self.tiling.pending_close_return = None;
        }
        self.tiling.columns[index].selected = id;
        self.tiling.active = Some(self.tiling.columns[index].id);
        true
    }

    pub fn adjacent_column(&self, backward: bool) -> Option<ElementId> {
        let index = self
            .tiling
            .columns
            .iter()
            .position(|c| Some(c.id) == self.tiling.active)?;
        let next = if backward {
            index.checked_sub(1)?
        } else {
            index + 1
        };
        self.tiling.columns.get(next).map(|c| c.selected)
    }

    pub fn adjacent_tile(&self, backward: bool) -> Option<ElementId> {
        let active = self.active_tile()?;
        let column = &self.tiling.columns[self.column_index(active)?];
        let index = column.windows.iter().position(|w| w.id == active)?;
        let next = if backward {
            index.checked_sub(1)?
        } else {
            index + 1
        };
        column.windows.get(next).map(|w| w.id)
    }

    pub fn move_column(&mut self, backward: bool) -> bool {
        let Some(index) = self
            .tiling
            .columns
            .iter()
            .position(|c| Some(c.id) == self.tiling.active)
        else {
            return false;
        };
        let next = if backward {
            index.checked_sub(1)
        } else {
            Some(index + 1)
        };
        let Some(next) = next.filter(|next| *next < self.tiling.columns.len()) else {
            return false;
        };
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.columns.swap(index, next);
        true
    }

    pub fn move_tile(&mut self, backward: bool) -> bool {
        let Some(id) = self.active_tile() else {
            return false;
        };
        let column_index = self.column_index(id).unwrap();
        let column = &mut self.tiling.columns[column_index];
        let index = column.windows.iter().position(|w| w.id == id).unwrap();
        let next = if backward {
            index.checked_sub(1)
        } else {
            Some(index + 1)
        };
        let Some(next) = next.filter(|next| *next < column.windows.len()) else {
            return false;
        };
        column.windows.swap(index, next);
        true
    }

    /// Move the active window after the selected window of its neighboring column.
    pub fn move_into_column(&mut self, backward: bool) -> bool {
        let Some(active) = self.active_tile() else {
            return false;
        };
        let Some(target) = self.adjacent_column(backward) else {
            return false;
        };
        let source = self.column_index(active).unwrap();
        let window = self.take_tiled_window(source, active);
        let target_index = self.column_index(target).unwrap();
        let column = &mut self.tiling.columns[target_index];
        let at = column.windows.iter().position(|w| w.id == target).unwrap() + 1;
        column.windows.insert(at, window);
        column.selected = active;
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = Some(column.id);
        true
    }

    /// Split the active window into a new column immediately to the right.
    pub fn expel_from_column(&mut self) -> bool {
        let Some(active) = self.active_tile() else {
            return false;
        };
        let Some(source) = self.column_index(active) else {
            return false;
        };
        if self.tiling.columns[source].windows.len() == 1 {
            return false;
        }
        let width = self.tiling.columns[source].width;
        let window = self.take_tiled_window(source, active);
        let id = self.next_column_id();
        self.tiling.columns.insert(
            source + 1,
            TiledColumn {
                id,
                selected: active,
                width,
                windows: vec![window],
            },
        );
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = Some(id);
        true
    }

    /// Append the first window of the next column without changing focus.
    pub fn consume_window_into_column(&mut self) -> bool {
        let Some(active) = self.active_tile() else {
            return false;
        };
        let target = self.column_index(active).unwrap();
        let source = target + 1;
        let Some(column) = self.tiling.columns.get(source) else {
            return false;
        };
        let id = column.windows[0].id;
        let window = self.take_tiled_window(source, id);
        self.tiling.columns[target].windows.push(window);
        true
    }

    /// Split the last window right while keeping the source column active.
    pub fn expel_window_from_column(&mut self) -> bool {
        let Some(active) = self.active_tile() else {
            return false;
        };
        let source = self.column_index(active).unwrap();
        let column = &self.tiling.columns[source];
        if column.windows.len() == 1 {
            return false;
        }
        let last = column.windows.last().unwrap().id;
        let width = column.width;
        let window = self.take_tiled_window(source, last);
        let id = self.next_column_id();
        self.tiling.columns.insert(
            source + 1,
            TiledColumn {
                id,
                selected: last,
                width,
                windows: vec![window],
            },
        );
        true
    }

    fn take_tiled_window(&mut self, source: usize, id: ElementId) -> TiledWindow {
        let column = &mut self.tiling.columns[source];
        let index = column.windows.iter().position(|w| w.id == id).unwrap();
        let window = column.windows.remove(index);
        if column.windows.is_empty() {
            self.tiling.columns.remove(source);
        } else if column.selected == id {
            column.selected = column.windows[index.saturating_sub(1)].id;
        }
        window
    }

    pub fn cycle_column_width(&mut self) {
        if let Some(index) = self
            .tiling
            .active
            .and_then(|id| self.tiling.columns.iter().position(|c| c.id == id))
        {
            let column = &mut self.tiling.columns[index];
            column.width = match column.width {
                ColumnWidth::Proportion(p) if p < 0.5 => ColumnWidth::Proportion(0.5),
                ColumnWidth::Proportion(p) if p < 1.0 => ColumnWidth::Proportion(1.0),
                _ => ColumnWidth::Proportion(1.0 / 3.0),
            };
        }
    }

    pub fn clear_tiling(&mut self) -> Vec<TiledWindow> {
        self.tiling.layout_id = None;
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = None;
        std::mem::take(&mut self.tiling.columns)
            .into_iter()
            .flat_map(|c| c.windows)
            .collect()
    }

    pub(super) fn prune_tiling(&mut self) {
        let live: std::collections::HashSet<_> = self.entries.iter().map(|e| e.id).collect();
        let previous_column = self
            .tiling
            .columns
            .iter()
            .position(|c| Some(c.id) == self.tiling.active);
        let surviving_before = previous_column.map_or(0, |index| {
            self.tiling.columns[..index]
                .iter()
                .filter(|c| c.windows.iter().any(|w| live.contains(&w.id)))
                .count()
        });
        if previous_column.is_some_and(|index| surviving_before != index) {
            self.tiling.restore_previous = None;
            self.tiling.pending_close_return = None;
        }
        for column in &mut self.tiling.columns {
            let selected_index = column
                .windows
                .iter()
                .position(|w| w.id == column.selected)
                .unwrap_or(0);
            let surviving_before = column.windows[..selected_index]
                .iter()
                .filter(|w| live.contains(&w.id))
                .count();
            column.windows.retain(|w| live.contains(&w.id));
            if !column.windows.is_empty() && !live.contains(&column.selected) {
                column.selected = column.windows[surviving_before.min(column.windows.len() - 1)].id;
            }
        }
        self.tiling.columns.retain(|c| !c.windows.is_empty());
        if !self
            .tiling
            .columns
            .iter()
            .any(|c| Some(c.id) == self.tiling.active)
        {
            let previous = self
                .tiling
                .restore_previous
                .take()
                .filter(|id| self.tiling.columns.iter().any(|c| c.id == *id));
            if let Some(previous) = previous
                && let Some(closed) = self.tiling.active
            {
                self.tiling.pending_close_return = Some((closed, previous));
            }
            self.tiling.active = previous.or_else(|| {
                self.tiling
                    .columns
                    .get(surviving_before)
                    .or(self.tiling.columns.last())
                    .map(|c| c.id)
            });
        }
        if self
            .tiling
            .restore_previous
            .is_some_and(|id| !self.tiling.columns.iter().any(|c| c.id == id))
        {
            self.tiling.restore_previous = None;
            self.tiling.pending_close_return = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::stage::mock::TestWindow;

    #[test]
    fn close_return_event_survives_noop_sweeps_and_is_consumed_once() {
        let mut stage = Stage::<TestWindow>::new();
        let a = TestWindow::new(1);
        let b = TestWindow::new(2);
        for window in [&a, &b] {
            stage.map(window.clone(), (0, 0).into());
            stage.tile(stage.id_of(window).unwrap(), (100, 100).into());
        }
        let previous = stage.column_of(stage.id_of(&a).unwrap()).unwrap();
        let opened = stage.active_column().unwrap();
        stage.remove(&b);
        stage.retain_alive();
        stage.select_tile(stage.id_of(&a).unwrap());
        assert_eq!(stage.take_tiling_close_return(), Some((opened, previous)));
        assert_eq!(stage.take_tiling_close_return(), None);
        stage.map(b.clone(), (0, 0).into());
        stage.tile(stage.id_of(&b).unwrap(), (100, 100).into());
        stage.remove(&b);
        stage.clear_tiling();
        assert_eq!(stage.take_tiling_close_return(), None);
        stage.verify_invariants();
    }

    #[test]
    fn layout_identity_survives_an_empty_strip_and_is_not_recycled() {
        let mut stage = Stage::<TestWindow>::new();
        assert_eq!(stage.tiling_layout_id(), None);
        let first = stage.begin_tiling();
        assert_eq!(stage.begin_tiling(), first);
        stage.verify_invariants();
        let window = TestWindow::new(0);
        stage.map(window.clone(), (10, 20).into());
        stage.tile(stage.id_of(&window).unwrap(), (100, 200).into());
        assert_eq!(stage.tiling_layout_id(), Some(first));
        stage.remove(&window);
        assert!(stage.tiled_columns().is_empty());
        assert_eq!(stage.tiling_layout_id(), Some(first));
        stage.verify_invariants();
        stage.clear_tiling();
        assert_eq!(stage.tiling_layout_id(), None);
        let second = stage.begin_tiling();
        assert_ne!(first, second);
        stage.clear_tiling();
        stage.map(window.clone(), (10, 20).into());
        stage.tile(stage.id_of(&window).unwrap(), (100, 200).into());
        let third = stage.tiling_layout_id().unwrap();
        assert_ne!(first, third);
        assert_ne!(second, third);
        stage.verify_invariants();
    }

    #[test]
    fn pinned_niri_consume_expel_topology_trace() {
        // Source-derived trace: niri 12de51d6, scrolling.rs consume_into_column
        // and expel_from_column. This covers topology, not sizing or animations.
        let mut stage = Stage::<TestWindow>::new();
        let mut ids = Vec::new();
        for i in 0..4 {
            let window = TestWindow::new(i);
            stage.map(window.clone(), (10, 20).into());
            let id = stage.id_of(&window).unwrap();
            stage.tile(id, (100, 200).into());
            ids.push(id);
        }
        let original: Vec<_> = stage.tiled_columns().iter().map(|c| c.id).collect();
        let assert_trace = |stage: &Stage<TestWindow>, expected: &[&[usize]], active: usize| {
            let topology: Vec<Vec<_>> = stage
                .tiled_columns()
                .iter()
                .map(|c| {
                    c.windows
                        .iter()
                        .map(|w| ids.iter().position(|id| *id == w.id).unwrap())
                        .collect()
                })
                .collect();
            assert_eq!(topology, expected);
            assert_eq!(stage.active_tile(), Some(ids[active]));
            stage.verify_invariants();
        };
        assert!(!stage.consume_window_into_column());
        stage.select_tile(ids[1]);
        assert!(stage.consume_window_into_column());
        assert_trace(&stage, &[&[0], &[1, 2], &[3]], 1);
        assert_eq!(stage.active_column(), Some(original[1]));
        assert!(stage.consume_window_into_column());
        assert_trace(&stage, &[&[0], &[1, 2, 3]], 1);
        assert!(stage.expel_window_from_column());
        assert_trace(&stage, &[&[0], &[1, 2], &[3]], 1);
        stage.select_tile(ids[2]);
        assert!(stage.expel_window_from_column());
        assert_trace(&stage, &[&[0], &[1], &[2], &[3]], 1);
        assert!(!stage.expel_window_from_column());
        stage.select_tile(ids[2]);
        assert!(stage.move_into_column(true));
        stage.select_tile(ids[0]);
        assert!(stage.consume_window_into_column());
        assert_trace(&stage, &[&[0, 1], &[2], &[3]], 0);
        assert_eq!(stage.tiled_columns()[1].selected, ids[2]);
        assert_eq!(stage.active_column(), Some(original[0]));
    }

    #[test]
    fn column_identity_survives_selection_merge_close_and_replacement() {
        let mut stage = Stage::<TestWindow>::new();
        let a = TestWindow::new(0);
        let b = TestWindow::new(1);
        for window in [&a, &b] {
            stage.map(window.clone(), (10, 20).into());
            stage.tile(stage.id_of(window).unwrap(), (100, 200).into());
        }
        let aid = stage.id_of(&a).unwrap();
        let bid = stage.id_of(&b).unwrap();
        let first = stage.column_of(aid).unwrap();
        let second = stage.column_of(bid).unwrap();
        assert_ne!(first, second);
        assert!(stage.move_into_column(true));
        assert_eq!(stage.active_column(), Some(first));
        assert_eq!(stage.column(second), None);
        stage.select_tile(aid);
        assert_eq!(stage.active_column(), Some(first));
        stage.select_tile(bid);
        assert!(stage.expel_from_column());
        let split = stage.active_column().unwrap();
        assert_ne!(split, first);
        assert_ne!(split, second);
        assert_eq!(stage.column_of(aid), Some(first));
        assert!(stage.move_into_column(true));
        stage.remove(&b);
        assert_eq!(stage.active_column(), Some(first));
        assert_eq!(stage.active_tile(), Some(aid));
        let replacement = TestWindow::new(2);
        stage.replace(&a, replacement.clone());
        assert_eq!(
            stage.column_of(stage.id_of(&replacement).unwrap()),
            Some(first)
        );
        stage.clear_tiling();
        stage.tile(aid, (100, 200).into());
        assert!(![first, second, split].contains(&stage.active_column().unwrap()));
        stage.verify_invariants();
    }

    #[test]
    fn reorder_preserves_identity_selection_restore_rectangles_and_z_order() {
        let mut stage = Stage::<TestWindow>::new();
        let windows: Vec<_> = (0..3).map(TestWindow::new).collect();
        for window in &windows {
            stage.map(window.clone(), (10, 20).into());
            stage.tile(stage.id_of(window).unwrap(), (100, 200).into());
        }
        let ids: Vec<_> = windows.iter().map(|w| stage.id_of(w).unwrap()).collect();
        let columns: Vec<_> = stage.tiled_columns().iter().map(|c| c.id).collect();
        assert!(!stage.move_column(false));
        assert!(stage.move_column(true));
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![columns[0], columns[2], columns[1]]
        );
        assert_eq!(stage.active_column(), Some(columns[2]));
        assert_eq!(stage.active_tile(), Some(ids[2]));
        assert!(stage.move_into_column(false));
        assert_eq!(stage.active_column(), Some(columns[1]));
        assert!(stage.move_tile(true));
        assert!(!stage.move_tile(true));
        assert_eq!(
            stage.tiled_columns()[1]
                .windows
                .iter()
                .map(|w| w.id)
                .collect::<Vec<_>>(),
            vec![ids[2], ids[1]]
        );
        assert!(stage.move_tile(false));
        assert!(!stage.move_tile(false));
        assert_eq!(stage.active_tile(), Some(ids[2]));
        assert_eq!(stage.windows().cloned().collect::<Vec<_>>(), windows);
        for tile in stage.tiled_windows() {
            assert_eq!(
                tile.restore,
                Rectangle::new((10, 20).into(), (100, 200).into())
            );
        }
        stage.verify_invariants();
    }

    #[test]
    fn insertion_selection_and_removal_preserve_column_order() {
        let mut stage = Stage::<TestWindow>::new();
        let windows: Vec<_> = (0..3).map(TestWindow::new).collect();
        for window in &windows {
            stage.map(window.clone(), (10, 20).into());
        }
        let ids: Vec<_> = windows.iter().map(|w| stage.id_of(w).unwrap()).collect();
        assert!(stage.tile(ids[0], (400, 300).into()));
        assert!(stage.tile(ids[1], (300, 200).into()));
        stage.select_tile(ids[0]);
        assert!(stage.tile(ids[2], (200, 100).into()));
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|c| c.selected)
                .collect::<Vec<_>>(),
            vec![ids[0], ids[2], ids[1]]
        );
        assert!(!stage.tile(ids[0], (1, 1).into()));
        assert_eq!(
            stage.tiled_columns()[0].windows[0].restore.size,
            (400, 300).into()
        );
        stage.remove(&windows[2]);
        assert_eq!(stage.active_tile(), Some(ids[0]));
        assert_eq!(stage.adjacent_column(true), None);
        assert_eq!(stage.adjacent_column(false), Some(ids[1]));
        stage.verify_invariants();
    }

    #[test]
    fn replacement_keeps_tiling_identity_and_crash_prunes_it() {
        let mut stage = Stage::<TestWindow>::new();
        let old = TestWindow::new(0);
        stage.map(old.clone(), (10, 20).into());
        let id = stage.id_of(&old).unwrap();
        stage.tile(id, (400, 300).into());
        let replacement = TestWindow::new(1);
        stage.replace(&old, replacement.clone());
        assert!(stage.is_tiled(id));
        assert_eq!(stage.window_by_id(id), Some(&replacement));
        replacement.kill();
        stage.retain_alive();
        assert!(stage.tiled_columns().is_empty());
        assert_eq!(stage.active_tile(), None);
        stage.verify_invariants();
    }

    proptest! {
        #[test]
        fn grouped_operations_preserve_membership_and_restore_geometry(
            ops in prop::collection::vec((0u8..12, 0usize..10), 0..150)
        ) {
            let mut stage = Stage::<TestWindow>::new();
            let mut windows: Vec<_> = (0..10).map(TestWindow::new).collect();
            for window in &windows { stage.map(window.clone(), (10, 20).into()); }
            let mut expected = std::collections::HashSet::new();
            for (op, index) in ops {
                let id = stage.id_of(&windows[index]).unwrap();
                match op {
                    0 => { stage.tile(id, (100, 200).into()); expected.insert(id); }
                    1 => { stage.select_tile(id); }
                    2 => { stage.move_into_column(true); }
                    3 => { stage.move_into_column(false); }
                    4 => { stage.expel_from_column(); }
                    5 => {
                        stage.remove(&windows[index]);
                        expected.remove(&id);
                        stage.map(windows[index].clone(), (10, 20).into());
                    }
                    6 => {
                        windows[index].kill();
                        stage.retain_alive();
                        expected.remove(&id);
                        windows[index] = TestWindow::new(index as u64);
                        stage.map(windows[index].clone(), (10, 20).into());
                    }
                    7 => { stage.move_column(index % 2 == 0); }
                    8 => { stage.move_tile(index % 2 == 0); }
                    9 => { stage.consume_window_into_column(); }
                    10 => { stage.expel_window_from_column(); }
                    _ => { stage.cycle_column_width(); }
                }
                stage.verify_invariants();
                prop_assert_eq!(stage.tiled_windows().map(|w| w.id).collect::<std::collections::HashSet<_>>(), expected.clone());
                for tile in stage.tiled_windows() {
                    prop_assert_eq!(tile.restore, Rectangle::new((10, 20).into(), (100, 200).into()));
                }
            }
        }

        #[test]
        fn random_operations_match_column_order_model(ops in prop::collection::vec((0u8..5, 0u64..12), 0..150)) {
            let mut stage = Stage::<TestWindow>::new();
            let windows: Vec<_> = (0..12).map(TestWindow::new).collect();
            for window in &windows { stage.map(window.clone(), (0, 0).into()); }
            let mut model = Vec::new();
            let mut active = None;
            for (op, index) in ops {
                let window = &windows[index as usize];
                let id = stage.id_of(window).unwrap();
                match op {
                    0 => {
                        stage.tile(id, (100, 100).into());
                        if !model.contains(&id) {
                            let at = model.iter().position(|id| Some(*id) == active).map_or(model.len(), |i| i + 1);
                            model.insert(at, id);
                            active = Some(id);
                        }
                    }
                    1 => {
                        stage.select_tile(id);
                        if model.contains(&id) { active = Some(id); }
                    }
                    2 => { stage.cycle_column_width(); }
                    3 => {
                        stage.clear_tiling();
                        model.clear();
                        active = None;
                    }
                    _ => { stage.raise(window); }
                }
                stage.verify_invariants();
                prop_assert_eq!(stage.tiled_columns().iter().map(|c| c.selected).collect::<Vec<_>>(), model.clone());
                prop_assert_eq!(stage.active_tile(), active);
            }
        }
    }

    #[test]
    fn moving_and_expelling_keeps_selection_and_remembers_each_columns_selected_window() {
        let mut stage = Stage::<TestWindow>::new();
        let windows: Vec<_> = (0..3).map(TestWindow::new).collect();
        let mut ids = Vec::new();
        for window in &windows {
            stage.map(window.clone(), (10, 20).into());
            let id = stage.id_of(window).unwrap();
            stage.tile(id, (100, 200).into());
            ids.push(id);
        }
        assert!(stage.move_into_column(true));
        assert_eq!(stage.tiled_columns().len(), 2);
        assert_eq!(
            stage.tiled_columns()[1]
                .windows
                .iter()
                .map(|w| w.id)
                .collect::<Vec<_>>(),
            vec![ids[1], ids[2]]
        );
        assert_eq!(stage.adjacent_tile(true), Some(ids[1]));
        stage.select_tile(ids[0]);
        assert_eq!(stage.adjacent_column(false), Some(ids[2]));
        stage.select_tile(ids[1]);
        assert!(stage.expel_from_column());
        assert_eq!(stage.active_tile(), Some(ids[1]));
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|c| c.selected)
                .collect::<Vec<_>>(),
            vec![ids[0], ids[2], ids[1]]
        );
        assert!(!stage.expel_from_column());
        stage.verify_invariants();
    }
}
