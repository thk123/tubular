use std::{cell::RefCell, rc::Rc};

use crate::{
    data_types::{chord_degree::ChordDegree, tatum::Tatum},
    model::{chord_sequence::ChordSequence, gui_state::GuiState, project_state::ProjectState},
};

pub(crate) struct ChordSequencerVm {
    gui_state: Rc<RefCell<GuiState>>,
    project_state: Rc<RefCell<ProjectState>>,
}

impl ChordSequencerVm {
    pub fn new(
        gui_state: Rc<RefCell<GuiState>>,
        project_state: Rc<RefCell<ProjectState>>,
    ) -> ChordSequencerVm {
        ChordSequencerVm {
            gui_state,
            project_state,
        }
    }

    pub fn move_left(&mut self) {
        self.change_chord(-1);
    }
    pub fn move_right(&mut self) {
        self.change_chord(1);
    }
    pub fn set_chord(&mut self, chord_degree: Option<ChordDegree>) {
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
    let (project_state, gui_state) = make_application_state();
    let mut vm = ChordSequencerVm::new(
        Rc::new(RefCell::new(gui_state)),
        Rc::new(RefCell::new(project_state)),
    );
    vm.move_left();
    assert_eq!(
        vm.gui_state.as_ref().borrow().selected_chord,
        Tatum::try_from(15).unwrap()
    );
}

#[test]
fn test_move_right() {
    let (project_state, gui_state) = make_application_state();
    let mut vm = ChordSequencerVm::new(
        Rc::new(RefCell::new(gui_state)),
        Rc::new(RefCell::new(project_state)),
    );
    vm.move_right();
    assert_eq!(
        vm.gui_state.as_ref().borrow().selected_chord,
        Tatum::try_from(1).unwrap()
    );
}

#[test]
fn test_set_chord() {
    let (project_state, gui_state) = make_application_state();
    let mut vm = ChordSequencerVm::new(
        Rc::new(RefCell::new(gui_state)),
        Rc::new(RefCell::new(project_state)),
    );
    vm.move_right();
    vm.set_chord(Some(ChordDegree::II));
    assert_eq!(
        vm.project_state.as_ref().borrow().chord_sequence[Tatum::try_from(1).unwrap()],
        Some(ChordDegree::II)
    );
}

#[test]
fn get_chord_sequence() {
    let (mut project_state, gui_state) = make_application_state();
    let chord_sequence = ChordSequence::new(Vec::from([Some(ChordDegree::II)])).unwrap();
    project_state.chord_sequence = chord_sequence.clone();
    let mut vm = ChordSequencerVm::new(
        Rc::new(RefCell::new(gui_state)),
        Rc::new(RefCell::new(project_state)),
    );
    assert_eq!(&vm.chord_sequence(), &chord_sequence);
}

#[test]
fn get_selected_chord() {
    let (project_state, mut gui_state) = make_application_state();
    gui_state.selected_chord = Tatum::try_from(10).unwrap();
    let mut vm = ChordSequencerVm::new(
        Rc::new(RefCell::new(gui_state)),
        Rc::new(RefCell::new(project_state)),
    );
    assert_eq!(vm.selected_chord(), Tatum::try_from(10).unwrap());
}
