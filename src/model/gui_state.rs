use crate::data_types::tatum::Tatum;

pub(crate) struct GuiState {
    pub selected_chord: Tatum,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            selected_chord: Tatum::try_from(0).unwrap(),
        }
    }
}
