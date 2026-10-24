use smithay::utils::{Logical, Rectangle, Size};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use crate::layout::scrolling::ColumnWidth;

use super::{ElementId, Stage, StageElement};

struct SavedTilePlacement {
    stack: usize,
    window: TiledWindow,
    column_id: ColumnId,
    width: ColumnWidth,
    selected: bool,
    active: bool,
}

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

#[derive(Clone, Debug, Default)]
pub struct TilingTopology {
    outputs: BTreeMap<String, SavedOutputTopology>,
}

#[derive(Clone, Debug, Default)]
struct SavedOutputTopology {
    columns: Vec<SavedColumnTopology>,
    active: Option<ColumnId>,
}

#[derive(Clone, Debug)]
struct SavedColumnTopology {
    id: ColumnId,
    selected: ElementId,
    width: ColumnWidth,
    windows: Vec<ElementId>,
}

#[derive(Default)]
pub(super) struct OutputTiling {
    pub(super) columns: Vec<TiledColumn>,
    pub(super) active: Option<ColumnId>,
    pub(super) restore_previous: Option<ColumnId>,
    pending_close_return: Option<(ColumnId, ColumnId)>,
    pub(super) layout_id: Option<TilingLayoutId>,
}

pub(super) struct Tiling {
    pub(super) outputs: BTreeMap<String, OutputTiling>,
    pub(super) active_output: String,
    next_column_id: u64,
    next_layout_id: u64,
}

impl Default for Tiling {
    fn default() -> Self {
        Self {
            outputs: BTreeMap::from([(String::new(), OutputTiling::default())]),
            active_output: String::new(),
            next_column_id: 0,
            next_layout_id: 0,
        }
    }
}

impl Tiling {
    fn active(&self) -> &OutputTiling {
        self.outputs
            .get(&self.active_output)
            .expect("active output tiling state missing")
    }

    fn active_mut(&mut self) -> &mut OutputTiling {
        self.outputs
            .get_mut(&self.active_output)
            .expect("active output tiling state missing")
    }
}

impl Deref for Tiling {
    type Target = OutputTiling;

    fn deref(&self) -> &Self::Target {
        self.active()
    }
}

impl DerefMut for Tiling {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.active_mut()
    }
}

impl<W: StageElement> Stage<W> {
    /// Capture column order, stacking, width and selection for a later tiling
    /// entry. Window restore rectangles are refreshed from their current canvas
    /// geometry when the topology is reapplied.
    pub fn tiling_topology(&self) -> TilingTopology {
        TilingTopology {
            outputs: self
                .tiling
                .outputs
                .iter()
                .filter(|(_, layout)| !layout.columns.is_empty())
                .map(|(output, layout)| {
                    (
                        output.clone(),
                        SavedOutputTopology {
                            columns: layout
                                .columns
                                .iter()
                                .map(|column| SavedColumnTopology {
                                    id: column.id,
                                    selected: column.selected,
                                    width: column.width,
                                    windows: column.windows.iter().map(|tile| tile.id).collect(),
                                })
                                .collect(),
                            active: layout.active,
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn tiling_topology_output<'a>(
        &self,
        topology: &'a TilingTopology,
        id: ElementId,
    ) -> Option<&'a str> {
        topology.outputs.iter().find_map(|(output, saved)| {
            saved
                .columns
                .iter()
                .any(|column| column.windows.contains(&id))
                .then_some(output.as_str())
        })
    }

    /// Reapply saved topology to the freshly enrolled live windows. Stale or
    /// ineligible IDs are ignored, while newly opened windows keep their new
    /// singleton columns after the remembered columns.
    pub fn restore_tiling_topology(&mut self, topology: &TilingTopology) {
        let mut live = BTreeMap::new();
        for (output, layout) in &self.tiling.outputs {
            for column in &layout.columns {
                for window in &column.windows {
                    live.insert(window.id, (output.clone(), *window));
                }
            }
        }

        for (output, layout) in &mut self.tiling.outputs {
            let Some(saved) = topology.outputs.get(output) else {
                continue;
            };
            let mut restored = Vec::new();
            let mut included = std::collections::HashSet::new();
            for column in &saved.columns {
                let windows: Vec<_> = column
                    .windows
                    .iter()
                    .filter_map(|id| {
                        let (owner, window) = live.get(id)?;
                        (owner == output && included.insert(*id)).then_some(*window)
                    })
                    .collect();
                if windows.is_empty() {
                    continue;
                }
                let selected = windows
                    .iter()
                    .find(|window| window.id == column.selected)
                    .map_or(windows[0].id, |window| window.id);
                restored.push(TiledColumn {
                    id: column.id,
                    selected,
                    width: column.width,
                    windows,
                });
            }
            for current in &layout.columns {
                if current
                    .windows
                    .iter()
                    .all(|window| included.contains(&window.id))
                {
                    continue;
                }
                for window in &current.windows {
                    if included.insert(window.id) {
                        restored.push(TiledColumn {
                            id: current.id,
                            selected: window.id,
                            width: current.width,
                            windows: vec![*window],
                        });
                    }
                }
            }
            layout.active = saved
                .active
                .filter(|id| restored.iter().any(|column| column.id == *id))
                .or_else(|| {
                    layout
                        .active
                        .filter(|id| restored.iter().any(|column| column.id == *id))
                })
                .or_else(|| restored.last().map(|column| column.id));
            layout.columns = restored;
            layout.restore_previous = None;
            layout.pending_close_return = None;
        }
    }

    /// Rebuild tiled windows from session placements after their process-local
    /// element IDs have been recreated. Placements for missing windows or
    /// disconnected outputs are ignored; newly tiled windows remain afterward.
    pub fn restore_tiling_placements(
        &mut self,
        placements: &[(ElementId, String, usize, usize, ColumnWidth, bool, bool)],
    ) {
        for (output, layout) in &mut self.tiling.outputs {
            let mut current = BTreeMap::new();
            for column in &layout.columns {
                for window in &column.windows {
                    current.insert(window.id, (*window, column.id, column.width));
                }
            }
            let mut saved: BTreeMap<usize, Vec<SavedTilePlacement>> = BTreeMap::new();
            for (id, owner, column, stack, width, selected, active) in placements {
                if owner == output
                    && let Some((window, column_id, _)) = current.get(id)
                {
                    saved.entry(*column).or_default().push(SavedTilePlacement {
                        stack: *stack,
                        window: *window,
                        column_id: *column_id,
                        width: *width,
                        selected: *selected,
                        active: *active,
                    });
                }
            }
            let mut included = std::collections::HashSet::new();
            let mut restored = Vec::new();
            let mut active = None;
            for (_, mut members) in saved {
                members.sort_by_key(|member| member.stack);
                let Some(first) = members.first() else {
                    continue;
                };
                let id = first.column_id;
                let mut width = first.width;
                if !matches!(width, ColumnWidth::Fixed(value) if value > 0)
                    && !matches!(width, ColumnWidth::Proportion(value) if value.is_finite() && value > 0.0)
                {
                    width = ColumnWidth::default();
                }
                let windows: Vec<_> = members
                    .iter()
                    .filter_map(|member| included.insert(member.window.id).then_some(member.window))
                    .collect();
                if windows.is_empty() {
                    continue;
                }
                let selected = members
                    .iter()
                    .find(|member| member.selected && included.contains(&member.window.id))
                    .map_or(windows[0].id, |member| member.window.id);
                let column_id = id;
                if members.iter().any(|member| member.active) {
                    active = Some(column_id);
                }
                restored.push(TiledColumn {
                    id: column_id,
                    selected,
                    width,
                    windows,
                });
            }
            for column in &layout.columns {
                for window in &column.windows {
                    if included.insert(window.id) {
                        restored.push(TiledColumn {
                            id: column.id,
                            selected: window.id,
                            width: column.width,
                            windows: vec![*window],
                        });
                    }
                }
            }
            if !restored.is_empty() {
                layout.active = active
                    .filter(|id| restored.iter().any(|column| column.id == *id))
                    .or_else(|| restored.last().map(|column| column.id));
                layout.columns = restored;
                layout.restore_previous = None;
                layout.pending_close_return = None;
            }
        }
    }

    /// Selects the output whose layout is used by the existing focused-layout APIs.
    pub fn activate_tiling_output(&mut self, output: &str) {
        self.tiling.outputs.entry(output.to_owned()).or_default();
        self.tiling.active_output = output.to_owned();
    }

    pub fn active_tiling_output(&self) -> &str {
        &self.tiling.active_output
    }

    pub fn tiling_layout_id_for_output(&self, output: &str) -> Option<TilingLayoutId> {
        self.tiling.outputs.get(output)?.layout_id
    }

    pub fn tiled_columns_for_output(&self, output: &str) -> Option<&[TiledColumn]> {
        Some(&self.tiling.outputs.get(output)?.columns)
    }

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

    pub fn begin_tiling_for_output(&mut self, output: &str) -> TilingLayoutId {
        self.activate_tiling_output(output);
        self.begin_tiling()
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

    pub fn active_column_for_output(&self, output: &str) -> Option<ColumnId> {
        self.tiling.outputs.get(output)?.active
    }

    pub fn column(&self, id: ColumnId) -> Option<&TiledColumn> {
        self.tiling.columns.iter().find(|c| c.id == id)
    }

    pub fn set_column_width(&mut self, id: ColumnId, width: ColumnWidth) -> bool {
        if !matches!(width, ColumnWidth::Fixed(width) if width > 0)
            && !matches!(width, ColumnWidth::Proportion(fraction) if fraction.is_finite() && fraction > 0.0)
        {
            return false;
        }
        let Some(column) = self
            .tiling
            .columns
            .iter_mut()
            .find(|column| column.id == id)
        else {
            return false;
        };
        if column.width == width {
            return false;
        }
        column.width = width;
        true
    }

    pub fn active_tile(&self) -> Option<ElementId> {
        self.column(self.tiling.active?).map(|c| c.selected)
    }

    pub fn column_of(&self, window: ElementId) -> Option<ColumnId> {
        self.tiling.outputs.values().find_map(|layout| {
            layout
                .columns
                .iter()
                .find(|column| column.windows.iter().any(|tile| tile.id == window))
                .map(|column| column.id)
        })
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
        self.tiling
            .outputs
            .values()
            .flat_map(|layout| layout.columns.iter().flat_map(|column| &column.windows))
    }

    pub fn tiled_windows_for_output<'a>(
        &'a self,
        output: &str,
    ) -> impl Iterator<Item = &'a TiledWindow> + 'a {
        self.tiling
            .outputs
            .get(output)
            .into_iter()
            .flat_map(|layout| layout.columns.iter().flat_map(|column| &column.windows))
    }

    pub fn tiling_output_for_window(&self, id: ElementId) -> Option<&str> {
        self.tiling.outputs.iter().find_map(|(output, layout)| {
            layout
                .columns
                .iter()
                .any(|column| column.windows.iter().any(|tile| tile.id == id))
                .then_some(output.as_str())
        })
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

    fn output_for_tile(&self, id: ElementId) -> Option<String> {
        self.tiling.outputs.iter().find_map(|(output, layout)| {
            layout
                .columns
                .iter()
                .any(|column| column.windows.iter().any(|tile| tile.id == id))
                .then(|| output.clone())
        })
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
        let Some(output) = self.output_for_tile(id) else {
            return false;
        };
        self.activate_tiling_output(&output);
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

    /// Swap two tiled windows in the same output layout, preserving their
    /// columns' selected-window identities and widths.
    pub fn swap_tiled_windows(&mut self, first: ElementId, second: ElementId) -> bool {
        if first == second {
            return false;
        }
        let Some(output) = self.output_for_tile(first) else {
            return false;
        };
        if self.output_for_tile(second).as_deref() != Some(output.as_str()) {
            return false;
        }
        self.activate_tiling_output(&output);
        let Some(first_column) = self.column_index(first) else {
            return false;
        };
        let Some(second_column) = self.column_index(second) else {
            return false;
        };
        let active_before = self.active_tile();
        let first_column_id = self.tiling.columns[first_column].id;
        let second_column_id = self.tiling.columns[second_column].id;
        if first_column == second_column {
            let column = &mut self.tiling.columns[first_column];
            let a = column
                .windows
                .iter()
                .position(|tile| tile.id == first)
                .unwrap();
            let b = column
                .windows
                .iter()
                .position(|tile| tile.id == second)
                .unwrap();
            column.windows.swap(a, b);
        } else {
            let (left, right) = if first_column < second_column {
                let (left, right) = self.tiling.columns.split_at_mut(second_column);
                (&mut left[first_column], &mut right[0])
            } else {
                let (left, right) = self.tiling.columns.split_at_mut(first_column);
                (&mut right[0], &mut left[second_column])
            };
            let a = left
                .windows
                .iter()
                .position(|tile| tile.id == first)
                .unwrap();
            let b = right
                .windows
                .iter()
                .position(|tile| tile.id == second)
                .unwrap();
            std::mem::swap(&mut left.windows[a], &mut right.windows[b]);

            for column in [first_column, second_column] {
                let tiled = &mut self.tiling.columns[column];
                if !tiled.windows.iter().any(|tile| tile.id == tiled.selected) {
                    tiled.selected = tiled.windows[0].id;
                }
            }
            match active_before {
                Some(id) if id == first => {
                    self.tiling.columns[second_column].selected = first;
                    self.tiling.active = Some(second_column_id);
                }
                Some(id) if id == second => {
                    self.tiling.columns[first_column].selected = second;
                    self.tiling.active = Some(first_column_id);
                }
                _ => {}
            }
        }
        let layout = self.tiling.active_mut();
        layout.restore_previous = None;
        layout.pending_close_return = None;
        true
    }

    /// Move a tiled window into the column containing `target`, immediately
    /// before or after that window. The moved window becomes selected.
    pub fn insert_tiled_window_at(
        &mut self,
        source: ElementId,
        target: ElementId,
        before: bool,
    ) -> bool {
        if source == target {
            return false;
        }
        let Some(output) = self.output_for_tile(source) else {
            return false;
        };
        if self.output_for_tile(target).as_deref() != Some(output.as_str()) {
            return false;
        }
        self.activate_tiling_output(&output);
        let Some(source_column) = self.column_index(source) else {
            return false;
        };
        let Some(target_column_id) = self
            .column_index(target)
            .map(|index| self.tiling.columns[index].id)
        else {
            return false;
        };
        let moved = self.take_tiled_window(source_column, source);
        let Some(target_column) = self
            .tiling
            .columns
            .iter_mut()
            .find(|column| column.id == target_column_id)
        else {
            return false;
        };
        let target_index = target_column
            .windows
            .iter()
            .position(|window| window.id == target)
            .expect("target column no longer contains its window");
        target_column
            .windows
            .insert(target_index + usize::from(!before), moved);
        target_column.selected = source;
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = Some(target_column_id);
        true
    }

    /// Pull a window out of its stack into a new column before `before`, or at
    /// the end of the strip when no following column is supplied.
    pub fn detach_tiled_window(&mut self, source: ElementId, before: Option<ElementId>) -> bool {
        let Some(output) = self.output_for_tile(source) else {
            return false;
        };
        if before
            .is_some_and(|target| self.output_for_tile(target).as_deref() != Some(output.as_str()))
        {
            return false;
        }
        self.activate_tiling_output(&output);
        let Some(source_column) = self.column_index(source) else {
            return false;
        };
        if self.tiling.columns[source_column].windows.len() < 2 {
            return false;
        }
        let width = self.tiling.columns[source_column].width;
        let target_column_id = before
            .and_then(|target| self.column_index(target))
            .map(|index| self.tiling.columns[index].id);
        let moved = self.take_tiled_window(source_column, source);
        let insert_at = target_column_id
            .and_then(|id| {
                self.tiling
                    .columns
                    .iter()
                    .position(|column| column.id == id)
            })
            .unwrap_or(self.tiling.columns.len());
        let id = self.next_column_id();
        self.tiling.columns.insert(
            insert_at,
            TiledColumn {
                id,
                selected: source,
                width,
                windows: vec![moved],
            },
        );
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = Some(id);
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
        let column_id = column.id;
        self.tiling.restore_previous = None;
        self.tiling.pending_close_return = None;
        self.tiling.active = Some(column_id);
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

    pub fn toggle_wide_column(&mut self) {
        if let Some(index) = self
            .tiling
            .active
            .and_then(|id| self.tiling.columns.iter().position(|c| c.id == id))
        {
            let column = &mut self.tiling.columns[index];
            column.width = match column.width {
                ColumnWidth::Proportion(p) if p >= 1.0 => ColumnWidth::default(),
                _ => ColumnWidth::Proportion(1.0),
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

    /// Clear one output's tiling layout without disturbing other outputs.
    pub fn clear_tiling_for_output(&mut self, output: &str) -> Vec<TiledWindow> {
        self.activate_tiling_output(output);
        self.clear_tiling()
    }

    /// Append one output's columns to another layout while retaining window,
    /// column and restore-rectangle identities.
    pub fn transfer_tiling_output(&mut self, from: &str, to: &str) -> Vec<TiledWindow> {
        if from == to || !self.tiling.outputs.contains_key(from) {
            return Vec::new();
        }

        self.begin_tiling_for_output(to);
        let Some(mut source) = self.tiling.outputs.remove(from) else {
            return Vec::new();
        };
        let moved = source
            .columns
            .iter()
            .flat_map(|column| column.windows.iter().copied())
            .collect();
        let target = self.tiling.outputs.get_mut(to).unwrap();
        if target.columns.is_empty() {
            target.active = source.active;
            target.restore_previous = source.restore_previous;
        }
        target.columns.append(&mut source.columns);
        target.pending_close_return = None;
        self.tiling.active_output = to.to_owned();
        moved
    }

    /// Move one tiled window to another output's strip, preserving its element
    /// and restore identities. A singleton column moves intact; a window taken
    /// from a stack becomes its own column on the destination.
    pub fn move_tiled_window_to_output(&mut self, id: ElementId, to: &str) -> bool {
        let Some(from) = self.output_for_tile(id) else {
            return false;
        };
        if from == to {
            return false;
        }

        self.begin_tiling_for_output(to);
        self.activate_tiling_output(&from);
        let source_index = self.column_index(id).unwrap();
        let source_column = &self.tiling.columns[source_index];
        let source_column_id = source_column.id;
        let width = source_column.width;
        let moving_column = source_column.windows.len() == 1;
        let moved = if moving_column {
            self.tiling.columns.remove(source_index)
        } else {
            let window = self.take_tiled_window(source_index, id);
            TiledColumn {
                id: self.next_column_id(),
                selected: id,
                width,
                windows: vec![window],
            }
        };

        let source = self.tiling.active_mut();
        source.restore_previous = None;
        source.pending_close_return = None;
        if moving_column && source.active == Some(source_column_id) {
            source.active = source
                .columns
                .get(source_index)
                .or_else(|| source.columns.last())
                .map(|column| column.id);
        }

        self.activate_tiling_output(to);
        let destination = self.tiling.active_mut();
        let insert_at = destination
            .active
            .and_then(|active| {
                destination
                    .columns
                    .iter()
                    .position(|column| column.id == active)
                    .map(|index| index + 1)
            })
            .unwrap_or(destination.columns.len());
        let moved_id = moved.id;
        destination.columns.insert(insert_at, moved);
        destination.active = Some(moved_id);
        destination.restore_previous = None;
        destination.pending_close_return = None;
        self.tiling.active_output = to.to_owned();
        true
    }

    /// Remove layout membership while preserving the element and its identity.
    pub fn untile(&mut self, id: ElementId) -> Option<TiledWindow> {
        let output = self.output_for_tile(id)?;
        self.activate_tiling_output(&output);
        let tile = self.tiled_windows().find(|tile| tile.id == id).copied()?;
        let live = self
            .entries
            .iter()
            .map(|e| e.id)
            .filter(|candidate| *candidate != id)
            .collect();
        self.prune_tiling_members(&live);
        Some(tile)
    }

    pub(super) fn prune_tiling(&mut self) {
        let live = self.entries.iter().map(|e| e.id).collect();
        self.prune_tiling_members(&live);
    }

    fn prune_tiling_members(&mut self, live: &std::collections::HashSet<ElementId>) {
        for layout in self.tiling.outputs.values_mut() {
            prune_output_tiling(layout, live);
        }
    }
}

fn prune_output_tiling(layout: &mut OutputTiling, live: &std::collections::HashSet<ElementId>) {
    let previous_column = layout
        .columns
        .iter()
        .position(|c| Some(c.id) == layout.active);
    let surviving_before = previous_column.map_or(0, |index| {
        layout.columns[..index]
            .iter()
            .filter(|c| c.windows.iter().any(|w| live.contains(&w.id)))
            .count()
    });
    if previous_column.is_some_and(|index| surviving_before != index) {
        layout.restore_previous = None;
        layout.pending_close_return = None;
    }
    for column in &mut layout.columns {
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
    layout.columns.retain(|c| !c.windows.is_empty());
    if !layout.columns.iter().any(|c| Some(c.id) == layout.active) {
        let previous = layout
            .restore_previous
            .take()
            .filter(|id| layout.columns.iter().any(|c| c.id == *id));
        if let Some(previous) = previous
            && let Some(closed) = layout.active
        {
            layout.pending_close_return = Some((closed, previous));
        }
        layout.active = previous.or_else(|| {
            layout
                .columns
                .get(surviving_before)
                .or(layout.columns.last())
                .map(|c| c.id)
        });
    }
    if layout
        .restore_previous
        .is_some_and(|id| !layout.columns.iter().any(|c| c.id == id))
    {
        layout.restore_previous = None;
        layout.pending_close_return = None;
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::stage::mock::TestWindow;

    #[test]
    fn output_layouts_keep_columns_selection_and_identity_separate() {
        let mut stage = Stage::<TestWindow>::new();
        let first = TestWindow::new(1);
        let second = TestWindow::new(2);
        stage.map(first.clone(), (0, 0).into());
        stage.map(second.clone(), (0, 0).into());
        let first_id = stage.id_of(&first).unwrap();
        let second_id = stage.id_of(&second).unwrap();

        let first_layout = stage.begin_tiling_for_output("DP-1");
        assert!(stage.tile(first_id, (800, 600).into()));
        let first_column = stage.active_column().unwrap();

        let second_layout = stage.begin_tiling_for_output("HDMI-A-1");
        assert_ne!(first_layout, second_layout);
        assert!(stage.tile(second_id, (800, 600).into()));
        let second_column = stage.active_column().unwrap();

        assert_ne!(first_column, second_column);
        assert_eq!(stage.active_tiling_output(), "HDMI-A-1");
        assert_eq!(stage.active_tile(), Some(second_id));
        assert_eq!(
            stage.tiling_layout_id_for_output("DP-1"),
            Some(first_layout)
        );
        assert_eq!(
            stage.tiled_columns_for_output("DP-1").unwrap()[0].selected,
            first_id
        );
        assert_eq!(stage.tiled_windows().count(), 2);
        assert!(stage.is_tiled(first_id));
        assert!(stage.is_tiled(second_id));

        assert!(stage.select_tile(first_id));
        assert_eq!(stage.active_tiling_output(), "DP-1");
        assert_eq!(stage.active_tile(), Some(first_id));
        stage.verify_invariants();

        assert_eq!(stage.clear_tiling_for_output("DP-1").len(), 1);
        assert!(!stage.is_tiled(first_id));
        assert!(stage.is_tiled(second_id));
        assert_eq!(stage.clear_tiling_for_output("HDMI-A-1").len(), 1);
        stage.verify_invariants();
    }

    #[test]
    fn session_placements_rebuild_columns_stacks_selection_and_output() {
        let mut stage = Stage::<TestWindow>::new();
        let windows: Vec<_> = (1..=4).map(TestWindow::new).collect();
        for window in &windows {
            stage.map(window.clone(), (10, 20).into());
        }
        let ids: Vec<_> = windows.iter().map(|w| stage.id_of(w).unwrap()).collect();
        stage.begin_tiling_for_output("DP-1");
        for id in ids.iter().copied() {
            assert!(stage.tile(id, (400, 300).into()));
        }
        stage.restore_tiling_placements(&[
            (
                ids[0],
                "DP-1".into(),
                0,
                0,
                ColumnWidth::Fixed(700),
                false,
                false,
            ),
            (
                ids[1],
                "DP-1".into(),
                0,
                1,
                ColumnWidth::Fixed(700),
                true,
                false,
            ),
            (
                ids[2],
                "DP-1".into(),
                1,
                0,
                ColumnWidth::Proportion(0.75),
                true,
                true,
            ),
            (
                ids[3],
                "HDMI-A-1".into(),
                0,
                0,
                ColumnWidth::default(),
                true,
                true,
            ),
        ]);

        let columns = stage.tiled_columns_for_output("DP-1").unwrap();
        assert_eq!(
            columns.len(),
            3,
            "unknown output placement falls back safely"
        );
        assert_eq!(
            columns[0].windows.iter().map(|w| w.id).collect::<Vec<_>>(),
            ids[..2]
        );
        assert_eq!(columns[0].selected, ids[1]);
        assert_eq!(columns[0].width, ColumnWidth::Fixed(700));
        assert_eq!(columns[1].windows[0].id, ids[2]);
        assert_eq!(columns[1].width, ColumnWidth::Proportion(0.75));
        assert_eq!(columns[2].windows[0].id, ids[3]);
        assert_eq!(stage.active_tiling_output(), "DP-1");
        assert_eq!(stage.active_tile(), Some(ids[2]));
        stage.verify_invariants();
    }

    #[test]
    fn transferring_an_output_appends_columns_and_preserves_restore_state() {
        let mut stage = Stage::<TestWindow>::new();
        let first = TestWindow::new(1);
        let second = TestWindow::new(2);
        let survivor = TestWindow::new(3);
        for window in [&first, &second, &survivor] {
            stage.map(window.clone(), (10, 20).into());
        }
        let first_id = stage.id_of(&first).unwrap();
        let second_id = stage.id_of(&second).unwrap();
        let survivor_id = stage.id_of(&survivor).unwrap();

        let from_layout = stage.begin_tiling_for_output("gone");
        stage.tile(first_id, (400, 300).into());
        stage.tile(second_id, (350, 250).into());
        let moved_column_ids: Vec<_> = stage.tiled_columns().iter().map(|c| c.id).collect();
        let restore = stage
            .tiled_windows()
            .find(|window| window.id == first_id)
            .unwrap()
            .restore;

        let survivor_layout = stage.begin_tiling_for_output("survivor");
        stage.tile(survivor_id, (500, 400).into());
        let survivor_column = stage.active_column().unwrap();

        let moved = stage.transfer_tiling_output("gone", "survivor");
        assert_eq!(moved.len(), 2);
        assert_eq!(stage.active_tiling_output(), "survivor");
        assert_eq!(stage.tiling_layout_id(), Some(survivor_layout));
        assert_eq!(stage.tiling_layout_id_for_output("gone"), None);
        assert_eq!(
            stage
                .tiled_columns()
                .iter()
                .map(|column| column.id)
                .collect::<Vec<_>>(),
            [vec![survivor_column], moved_column_ids].concat()
        );
        assert_eq!(stage.tiling_output_for_window(first_id), Some("survivor"));
        assert_eq!(stage.tiling_output_for_window(second_id), Some("survivor"));
        assert_eq!(
            stage
                .tiled_windows()
                .find(|window| window.id == first_id)
                .unwrap()
                .restore,
            restore
        );
        assert_ne!(from_layout, survivor_layout);
        stage.verify_invariants();
    }

    #[test]
    fn moving_a_stacked_tile_to_another_output_splits_a_stable_window() {
        let mut stage = Stage::<TestWindow>::new();
        let first = TestWindow::new(1);
        let second = TestWindow::new(2);
        let destination = TestWindow::new(3);
        for window in [&first, &second, &destination] {
            stage.map(window.clone(), (10, 20).into());
        }
        let first_id = stage.id_of(&first).unwrap();
        let second_id = stage.id_of(&second).unwrap();
        let destination_id = stage.id_of(&destination).unwrap();

        stage.begin_tiling_for_output("source");
        stage.tile(first_id, (400, 300).into());
        stage.tile(second_id, (350, 250).into());
        stage.select_tile(first_id);
        assert!(stage.consume_window_into_column());
        stage.select_tile(second_id);
        let source_column = stage.column_of(first_id).unwrap();
        let restore = stage
            .tiled_windows()
            .find(|window| window.id == second_id)
            .unwrap()
            .restore;

        stage.begin_tiling_for_output("destination");
        stage.tile(destination_id, (500, 400).into());
        let destination_column = stage.active_column().unwrap();

        assert!(stage.move_tiled_window_to_output(second_id, "destination"));

        assert_eq!(stage.tiling_output_for_window(first_id), Some("source"));
        assert_eq!(
            stage.tiling_output_for_window(second_id),
            Some("destination")
        );
        assert_eq!(stage.column_of(first_id), Some(source_column));
        assert_ne!(stage.column_of(second_id), Some(source_column));
        assert_eq!(stage.active_tiling_output(), "destination");
        assert_eq!(stage.active_tile(), Some(second_id));
        assert_eq!(stage.tiled_columns().len(), 2);
        assert_eq!(stage.tiled_columns()[0].id, destination_column);
        assert_eq!(
            stage
                .tiled_windows()
                .find(|window| window.id == second_id)
                .unwrap()
                .restore,
            restore
        );
        stage.verify_invariants();
    }

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

    #[test]
    fn untile_preserves_identity_and_repairs_selection_before_reenrollment() {
        let mut stage = Stage::<TestWindow>::new();
        let a = TestWindow::new(1);
        let b = TestWindow::new(2);
        for window in [&a, &b] {
            stage.map(window.clone(), (10, 20).into());
            stage.tile(stage.id_of(window).unwrap(), (100, 200).into());
        }
        let aid = stage.id_of(&a).unwrap();
        let bid = stage.id_of(&b).unwrap();
        let layout = stage.tiling_layout_id();
        let old_column = stage.active_column();
        let tile = stage.untile(bid).unwrap();
        assert_eq!(
            tile.restore,
            Rectangle::new((10, 20).into(), (100, 200).into())
        );
        assert_eq!(stage.id_of(&b), Some(bid));
        assert_eq!(stage.active_tile(), Some(aid));
        assert_eq!(stage.untile(bid), None);
        stage.untile(aid).unwrap();
        assert!(stage.tiled_columns().is_empty());
        assert_eq!(stage.tiling_layout_id(), layout);
        assert!(stage.tile(bid, (300, 400).into()));
        assert_ne!(stage.active_column(), old_column);
        assert_eq!(stage.active_tile(), Some(bid));
        stage.verify_invariants();
    }

    proptest! {
        #[test]
        fn grouped_operations_preserve_membership_and_restore_geometry(
            ops in prop::collection::vec((0u8..13, 0usize..10), 0..150)
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
                    11 => { stage.untile(id); expected.remove(&id); }
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
