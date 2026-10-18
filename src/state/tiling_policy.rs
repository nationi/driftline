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
                | Action::GrowWindow(_)
                | Action::ShrinkWindow(_)
                | Action::PanViewport(_)
                | Action::CenterWindow
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
        if self.tiling_view.is_some() && operation.restores_canvas() {
            tracing::debug!(
                ?operation,
                "Restoring canvas before leaving scrolling tiling"
            );
            self.exit_tiling();
        }
    }
}
