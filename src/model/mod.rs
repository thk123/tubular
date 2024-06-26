use self::{gui_state::GuiState, project_state::ProjectState};

pub mod chord_sequence;
pub mod gui_state;
pub mod project_state;
pub mod project_time_info;

pub(crate) fn make_application_state() -> (ProjectState, GuiState) {
    (ProjectState::default(), GuiState::default())
}
