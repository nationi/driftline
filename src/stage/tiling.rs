use smithay::utils::{Logical, Rectangle, Size};

use crate::layout::scrolling::ColumnWidth;

use super::{ElementId, Stage, StageElement};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TiledWindow {
    pub id: ElementId,
    pub restore: Rectangle<i32, Logical>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TiledColumn {
    /// The selected window in this column, remembered during horizontal navigation.
    pub id: ElementId,
    pub width: ColumnWidth,
    pub windows: Vec<TiledWindow>,
}

#[derive(Default)]
pub(super) struct Tiling {
    pub(super) columns: Vec<TiledColumn>,
    pub(super) active: Option<ElementId>,
}

impl<W: StageElement> Stage<W> {
    pub fn tiled_columns(&self) -> &[TiledColumn] {
        &self.tiling.columns
    }

    /// Selected window of the active column.
    pub fn active_column(&self) -> Option<ElementId> {
        self.tiling.active
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
        let column = TiledColumn {
            id,
            width: ColumnWidth::Proportion(0.5),
            windows: vec![TiledWindow {
                id,
                restore: Rectangle::new(entry.position, restore_size),
            }],
        };
        let index = self
            .tiling
            .active
            .and_then(|id| self.column_index(id))
            .map_or(self.tiling.columns.len(), |i| i + 1);
        self.tiling.columns.insert(index, column);
        self.tiling.active = Some(id);
        true
    }

    pub fn select_column(&mut self, id: ElementId) -> bool {
        let Some(index) = self.column_index(id) else {
            return false;
        };
        self.tiling.columns[index].id = id;
        self.tiling.active = Some(id);
        true
    }

    pub fn adjacent_column(&self, backward: bool) -> Option<ElementId> {
        let index = self.column_index(self.tiling.active?)?;
        let next = if backward {
            index.checked_sub(1)?
        } else {
            index + 1
        };
        self.tiling.columns.get(next).map(|c| c.id)
    }

    pub fn adjacent_tile(&self, backward: bool) -> Option<ElementId> {
        let active = self.tiling.active?;
        let column = &self.tiling.columns[self.column_index(active)?];
        let index = column.windows.iter().position(|w| w.id == active)?;
        let next = if backward {
            index.checked_sub(1)?
        } else {
            index + 1
        };
        column.windows.get(next).map(|w| w.id)
    }

    /// Move the active window after the selected window of its neighboring column.
    pub fn move_into_column(&mut self, backward: bool) -> bool {
        let Some(active) = self.tiling.active else {
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
        column.id = active;
        true
    }

    /// Split the active window into a new column immediately to the right.
    pub fn expel_from_column(&mut self) -> bool {
        let Some(active) = self.tiling.active else {
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
        self.tiling.columns.insert(
            source + 1,
            TiledColumn {
                id: active,
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
        } else {
            column.id = column.windows[index.saturating_sub(1)].id;
        }
        window
    }

    pub fn cycle_column_width(&mut self) {
        if let Some(index) = self.tiling.active.and_then(|id| self.column_index(id)) {
            let column = &mut self.tiling.columns[index];
            column.width = match column.width {
                ColumnWidth::Proportion(p) if p < 0.5 => ColumnWidth::Proportion(0.5),
                ColumnWidth::Proportion(p) if p < 1.0 => ColumnWidth::Proportion(1.0),
                _ => ColumnWidth::Proportion(1.0 / 3.0),
            };
        }
    }

    pub fn clear_tiling(&mut self) -> Vec<TiledWindow> {
        self.tiling.active = None;
        std::mem::take(&mut self.tiling.columns)
            .into_iter()
            .flat_map(|c| c.windows)
            .collect()
    }

    pub(super) fn prune_tiling(&mut self) {
        let active_index = self.tiling.active.and_then(|id| self.column_index(id));
        let mut fallback = None;
        for (index, column) in self.tiling.columns.iter_mut().enumerate() {
            let previous = column
                .windows
                .iter()
                .position(|w| w.id == column.id)
                .unwrap_or(0);
            column
                .windows
                .retain(|w| self.entries.iter().any(|entry| entry.id == w.id));
            if !column.windows.is_empty() {
                if !column.windows.iter().any(|w| w.id == column.id) {
                    column.id =
                        column.windows[previous.saturating_sub(1).min(column.windows.len() - 1)].id;
                }
                if Some(index) == active_index {
                    fallback = Some(column.id);
                }
            }
        }
        self.tiling.columns.retain(|c| !c.windows.is_empty());
        if !self.tiling.active.is_some_and(|id| self.is_tiled(id)) {
            self.tiling.active = fallback.or_else(|| {
                active_index.and_then(|index| {
                    self.tiling
                        .columns
                        .get(index.saturating_sub(1))
                        .or(self.tiling.columns.last())
                        .map(|c| c.id)
                })
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::stage::mock::TestWindow;

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
        stage.select_column(ids[0]);
        assert!(stage.tile(ids[2], (200, 100).into()));
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![ids[0], ids[2], ids[1]]
        );
        assert!(!stage.tile(ids[0], (1, 1).into()));
        assert_eq!(
            stage.tiled_columns()[0].windows[0].restore.size,
            (400, 300).into()
        );
        stage.remove(&windows[2]);
        assert_eq!(stage.active_column(), Some(ids[0]));
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
        assert_eq!(stage.active_column(), None);
        stage.verify_invariants();
    }

    proptest! {
        #[test]
        fn grouped_operations_preserve_membership_and_restore_geometry(
            ops in prop::collection::vec((0u8..8, 0usize..10), 0..150)
        ) {
            let mut stage = Stage::<TestWindow>::new();
            let mut windows: Vec<_> = (0..10).map(TestWindow::new).collect();
            for window in &windows { stage.map(window.clone(), (10, 20).into()); }
            let mut expected = std::collections::HashSet::new();
            for (op, index) in ops {
                let id = stage.id_of(&windows[index]).unwrap();
                match op {
                    0 => { stage.tile(id, (100, 200).into()); expected.insert(id); }
                    1 => { stage.select_column(id); }
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
                        stage.select_column(id);
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
                prop_assert_eq!(stage.tiled_columns().iter().map(|c| c.id).collect::<Vec<_>>(), model.clone());
                prop_assert_eq!(stage.active_column(), active);
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
        stage.select_column(ids[0]);
        assert_eq!(stage.adjacent_column(false), Some(ids[2]));
        stage.select_column(ids[1]);
        assert!(stage.expel_from_column());
        assert_eq!(stage.active_column(), Some(ids[1]));
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![ids[0], ids[2], ids[1]]
        );
        assert!(!stage.expel_from_column());
        stage.verify_invariants();
    }
}
