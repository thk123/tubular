use std::sync::{Arc, RwLock};

use jack::{AsyncClient, MidiOut, Port, ProcessHandler};

use crate::model::project_state::ProjectState;

pub(crate) struct JackProcessor {
    project_state: Arc<RwLock<ProjectState>>,
    chord_port: Port<MidiOut>,
}

impl JackProcessor {
    pub(crate) fn activate_async(
        project_state: Arc<RwLock<ProjectState>>,
    ) -> AsyncClient<(), JackProcessor> {
        let (client, _status) =
            jack::Client::new("tubular", jack::ClientOptions::NO_START_SERVER).unwrap();
        let chord_port = client
            .register_port("chords", jack::MidiOut::default())
            .unwrap();

        let client_handler = JackProcessor {
            project_state,
            chord_port,
        };

        client.activate_async((), client_handler).unwrap()
    }
}

impl ProcessHandler for JackProcessor {
    fn process(&mut self, _: &jack::Client, _process_scope: &jack::ProcessScope) -> jack::Control {
        jack::Control::Continue
    }
}
