use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use ::jack::AsyncClient;
use jack::jack_processor::JackProcessor;
use model::{gui_state::GuiState, make_application_state, project_state::ProjectState};
use view_model::chord_sequencer_vm::ChordSequencerVm;

pub mod data_types;
pub mod jack;
pub mod model;
pub mod music_theory;
pub mod view;
pub mod view_model;

struct TubularApp {
    project_state: Arc<RwLock<ProjectState>>,
    gui_state: Rc<RefCell<GuiState>>,
    chord_sequencer_vm: ChordSequencerVm,
    jack_client: Option<AsyncClient<(), JackProcessor>>,
}

impl TubularApp {
    fn new() -> TubularApp {
        let (project_state, gui_state) = make_application_state();

        let gui_state_pointer = Rc::new(RefCell::new(gui_state));
        let project_state_pointer = Arc::new(RwLock::new(project_state));

        let chord_sequencer_vm =
            ChordSequencerVm::new(gui_state_pointer.clone(), project_state_pointer.clone());

        let jack_client = JackProcessor::activate_async(project_state_pointer.clone());

        TubularApp {
            project_state: project_state_pointer,
            gui_state: gui_state_pointer,
            chord_sequencer_vm,
            jack_client: Some(jack_client),
        }
    }
}

impl eframe::App for TubularApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        view::chord_sequencer::update(&mut self.chord_sequencer_vm, ctx, frame);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let jack_client = std::mem::take(&mut self.jack_client);
        jack_client.unwrap().deactivate().unwrap();
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "tubular",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(TubularApp::new())),
    )
}
