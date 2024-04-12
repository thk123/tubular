use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::{
    data_types::{chord_degree::ChordDegree, tatum::Tatum},
    model::{chord_sequence::ChordSequence, gui_state::GuiState, project_state::ProjectState},
};

#[derive(Default)]
pub(crate) struct ChordSequencerVm {
    gui_state: Rc<RefCell<GuiState>>,
    project_state: Rc<RefCell<ProjectState>>,
}

impl ChordSequencerVm {
    pub fn move_left(self: &mut Self) {
        self.change_chord(-1);
    }
    pub fn move_right(self: &mut Self) {
        self.change_chord(1);
    }
    pub fn set_chord(self: &mut Self, chord_degree: Option<ChordDegree>) {
        self.project_state
            .as_ref()
            .borrow_mut()
            .update_chord_sequence(
                self.gui_state.as_ref().borrow().selected_chord,
                chord_degree,
            );
    }

    pub fn chord_sequence(&mut self) -> ChordSequence {
        // TODO: why do we have to clone the sequence, ideally want to extend the lifetime of this reference
        return self.project_state.as_ref().borrow().chord_sequence.clone();
    }

    pub fn selected_chord(&mut self) -> Tatum {
        self.gui_state.as_ref().borrow().selected_chord
    }

    fn change_chord(&mut self, delta: i32) {
        let new_selected_modulo_chord = self.gui_state.as_ref().borrow().selected_chord.add(delta);
        self.gui_state.as_ref().borrow_mut().selected_chord = new_selected_modulo_chord;
    }
}

#[test]
fn test_move_left() {
    let mut vm = ChordSequencerVm::default();
    vm.move_left();
    assert_eq!(
        vm.gui_state.as_ref().borrow().selected_chord,
        Tatum::try_from(15).unwrap()
    );
}

#[test]
fn test_move_right() {
    let mut vm = ChordSequencerVm::default();
    vm.move_right();
    assert_eq!(
        vm.gui_state.as_ref().borrow().selected_chord,
        Tatum::try_from(1).unwrap()
    );
}

#[test]
fn test_set_chord() {
    let mut vm = ChordSequencerVm::default();
    vm.move_right();
    vm.set_chord(Some(ChordDegree::II));
    assert_eq!(
        vm.project_state.as_ref().borrow().chord_sequence[Tatum::try_from(1).unwrap()],
        Some(ChordDegree::II)
    );
}

#[test]
fn get_chord_sequence() {
    let mut vm = ChordSequencerVm::default();
    let chord_sequence = ChordSequence::new(Vec::from([Some(ChordDegree::II)])).unwrap();
    vm.project_state.borrow_mut().chord_sequence = chord_sequence.clone();
    assert_eq!(&vm.chord_sequence(), &chord_sequence);
}

#[test]
fn get_selected_chord() {
    let mut vm = ChordSequencerVm::default();
    vm.gui_state.borrow_mut().selected_chord = Tatum::try_from(10).unwrap();
    assert_eq!(vm.selected_chord(), Tatum::try_from(10).unwrap());
}
