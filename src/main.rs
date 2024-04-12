use std::{cell::RefCell, rc::Rc};

use model::{gui_state::{GuiState}, make_application_state, project_state::{ProjectState}};
use view_model::chord_sequencer_vm::{ChordSequencerVm};

pub mod view;
pub mod view_model;
pub mod model;
pub mod data_types;

struct TubularApp
{
    project_state: Rc<RefCell<ProjectState>>,
    gui_state: Rc<RefCell<GuiState>>,
    chord_sequencer_vm: ChordSequencerVm,
}

impl TubularApp {
    fn new() -> TubularApp {
        let (project_state, gui_state) = make_application_state();

        let gui_state_pointer = Rc::new(RefCell::new(gui_state));
        let project_state_pointer = Rc::new(RefCell::new(project_state));

        let chord_sequencer_vm = ChordSequencerVm::new(gui_state_pointer.clone(), project_state_pointer.clone());

        TubularApp {
            project_state: project_state_pointer,
            gui_state: gui_state_pointer,
            chord_sequencer_vm,
        }
    }
}

impl eframe::App for TubularApp
{
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        view::chord_sequencer::update(&mut self.chord_sequencer_vm, ctx, frame);
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "tubular",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(TubularApp::new())))
}
