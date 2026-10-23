use driftwm::config::{Action, Direction};

use super::DriftWm;

#[derive(Debug)]
pub(crate) enum TilingOperation<'a> {
    Action(&'a Action),
    MapWindow { tiled: bool },
    FreePlacement,
    FreeMove,
    FreeResize,
    Fullscreen,
    Fit,
    Fill,
    Pin,
    LayoutInvalidated,
}

impl TilingOperation<'_> {
    fn restores_canvas(&self) -> bool {
        match self {
            Self::Action(action) => match action {
                Action::ToggleTiling
                | Action::FocusColumn(_)
                | Action::CycleColumnWidth
                | Action::ToggleWideColumn
                | Action::MoveIntoColumn(_)
                | Action::ConsumeWindowIntoColumn
                | Action::ExpelWindowFromColumn
                | Action::ExpelFromColumn
                | Action::MoveColumn(_)
                | Action::MoveTiledWindow(_)
                | Action::Exec(_)
                | Action::ExecTerminal
                | Action::ExecLauncher
                | Action::Spawn(_)
                | Action::CloseWindow
                | Action::SwitchLayout(_)
                | Action::SetTrackpad(_)
                | Action::CenterNearest(
                    Direction::Left | Direction::Right | Direction::Up | Direction::Down,
                ) => false,
                Action::SuspendWindow
                | Action::NudgeWindow(_)
                | Action::MoveToSnap(_)
                | Action::GrowWindow(_)
                | Action::ShrinkWindow(_)
                | Action::ResizeToSnap(_)
                | Action::PanViewport(_)
                | Action::CenterWindow
                | Action::FocusNearest(_)
                | Action::CenterNearest(
                    Direction::UpLeft
                    | Direction::UpRight
                    | Direction::DownLeft
                    | Direction::DownRight,
                )
                | Action::CycleWindows { .. }
                | Action::HomeToggle
                | Action::GoToBookmark(_)
                | Action::SetBookmark(_)
                | Action::MoveToBookmark(_)
                | Action::ZoomIn
                | Action::ZoomOut
                | Action::ZoomReset
                | Action::ZoomToFit
                | Action::ZoomToFitSnapped
                | Action::ToggleFullscreen
                | Action::FitWindow
                | Action::FitWindowSnapped
                | Action::FillWindow
                | Action::SendToOutput(_)
                | Action::SendCursorToOutput(_)
                | Action::FocusCenter
                | Action::TogglePinToScreen
                | Action::ReloadConfig
                | Action::ToggleCursorPan
                | Action::Quit => true,
            },
            Self::MapWindow { tiled } => *tiled,
            Self::FreePlacement
            | Self::FreeMove
            | Self::FreeResize
            | Self::Fullscreen
            | Self::Fit
            | Self::Fill
            | Self::Pin
            | Self::LayoutInvalidated => true,
        }
    }
}

impl DriftWm {
    pub(crate) fn prepare_tiling_operation(&mut self, operation: TilingOperation<'_>) {
        let restores_canvas = match &operation {
            TilingOperation::Action(Action::SendToOutput(_)) => self
                .focused_element()
                .and_then(|element| self.stage.id_of(&element))
                .is_none_or(|id| !self.stage.is_tiled(id)),
            _ => operation.restores_canvas(),
        };
        if self.tiling_view.is_some() && restores_canvas {
            if matches!(&operation, TilingOperation::Fullscreen) {
                self.resume_tiling_after_fullscreen = true;
            }
            tracing::debug!(
                ?operation,
                "Restoring canvas before leaving scrolling tiling"
            );
            self.exit_tiling();
        }
    }

    pub(crate) fn resume_tiling_after_fullscreen(&mut self) {
        if !self.resume_tiling_after_fullscreen || self.stage.has_fullscreen() {
            return;
        }
        self.resume_tiling_after_fullscreen = false;
        self.toggle_tiling();
    }
}
