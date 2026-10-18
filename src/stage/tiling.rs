use smithay::utils::{Logical, Rectangle, Size};

use crate::layout::scrolling::ColumnWidth;

use super::{ElementId, Stage, StageElement};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TiledColumn {
    pub id: ElementId,
    pub width: ColumnWidth,
    pub restore: Rectangle<i32, Logical>,
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

    pub fn active_column(&self) -> Option<ElementId> {
        self.tiling.active
    }

    pub fn is_tiled(&self, id: ElementId) -> bool {
        self.tiling.columns.iter().any(|c| c.id == id)
    }

    /// Insert after the selected column, saving the original canvas rectangle once.
    pub fn tile(&mut self, id: ElementId, restore_size: Size<i32, Logical>) -> bool {
        if self.is_tiled(id) {
            return false;
        }
        let Some(entry) = self.entries.iter().find(|entry| entry.id == id) else {
            return false;
        };
        let size = restore_size;
        if size.w <= 0 || size.h <= 0 {
            return false;
        }
        let column = TiledColumn {
            id,
            width: ColumnWidth::Proportion(0.5),
            restore: Rectangle::new(entry.position, size),
        };
        let index = self
            .tiling
            .columns
            .iter()
            .position(|c| Some(c.id) == self.tiling.active)
            .map_or(self.tiling.columns.len(), |i| i + 1);
        self.tiling.columns.insert(index, column);
        self.tiling.active = Some(id);
        true
    }

    pub fn select_column(&mut self, id: ElementId) -> bool {
        if !self.is_tiled(id) {
            return false;
        }
        self.tiling.active = Some(id);
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
        self.tiling.columns.get(next).map(|c| c.id)
    }

    pub fn cycle_column_width(&mut self) {
        if let Some(column) = self
            .tiling
            .columns
            .iter_mut()
            .find(|c| Some(c.id) == self.tiling.active)
        {
            column.width = match column.width {
                ColumnWidth::Proportion(p) if p < 0.5 => ColumnWidth::Proportion(0.5),
                ColumnWidth::Proportion(p) if p < 1.0 => ColumnWidth::Proportion(1.0),
                _ => ColumnWidth::Proportion(1.0 / 3.0),
            };
        }
    }

    pub fn clear_tiling(&mut self) -> Vec<TiledColumn> {
        self.tiling.active = None;
        std::mem::take(&mut self.tiling.columns)
    }

    pub(super) fn prune_tiling(&mut self) {
        let previous = self
            .tiling
            .columns
            .iter()
            .position(|c| Some(c.id) == self.tiling.active);
        self.tiling
            .columns
            .retain(|c| self.entries.iter().any(|entry| entry.id == c.id));
        if !self
            .tiling
            .columns
            .iter()
            .any(|c| Some(c.id) == self.tiling.active)
        {
            self.tiling.active = previous.and_then(|i| {
                self.tiling
                    .columns
                    .get(i.saturating_sub(1))
                    .or(self.tiling.columns.last())
                    .map(|c| c.id)
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
        assert_eq!(stage.tiled_columns()[0].restore.size, (400, 300).into());
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
}
