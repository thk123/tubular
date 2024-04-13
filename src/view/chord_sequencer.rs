use eframe::egui::{self, Color32, FontId, Key, RichText};

use crate::{
    data_types::{chord_degree::ChordDegree, tatum::Tatum},
    view_model::chord_sequencer_vm::ChordSequencerVm,
};

fn numeric_key_pressed(input_state: &egui::InputState) -> Option<ChordDegree> {
    if input_state.key_pressed(Key::Num0) {
        return None;
    }
    if input_state.key_pressed(Key::Num1) {
        return Some(ChordDegree::I);
    }
    if input_state.key_pressed(Key::Num2) {
        return Some(ChordDegree::II);
    }
    if input_state.key_pressed(Key::Num3) {
        return Some(ChordDegree::III);
    }
    if input_state.key_pressed(Key::Num4) {
        return Some(ChordDegree::IV);
    }
    if input_state.key_pressed(Key::Num5) {
        return Some(ChordDegree::V);
    }
    if input_state.key_pressed(Key::Num6) {
        return Some(ChordDegree::VI);
    }
    if input_state.key_pressed(Key::Num7) {
        return Some(ChordDegree::VII);
    }
    if input_state.key_pressed(Key::Num8) {
        return None;
    }
    if input_state.key_pressed(Key::Num9) {
        return None;
    }
    None
}

pub(crate) fn update(vm: &mut ChordSequencerVm, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        vm.move_left();
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        vm.move_right();
    }

    if let Some(chord_degree) = ctx.input(numeric_key_pressed) {
        vm.set_chord(Some(chord_degree))
    }

    if ctx.input(|i| i.key_pressed(Key::Backspace)) {
        vm.set_chord(None);
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (index, chord) in vm.chord_sequence().iter().enumerate() {
                let text = match chord {
                    Some(c) => c.to_string(),
                    None => ".".to_string(),
                };
                let centred_text = format!("{:^3}", text);
                let (bg_colour, fg_colour) =
                    if Tatum::try_from(index).unwrap() == vm.selected_chord() {
                        (Color32::BLACK, Color32::WHITE)
                    } else {
                        (Color32::WHITE, Color32::BLACK)
                    };
                let rich_text = RichText::new(centred_text)
                    .background_color(bg_colour)
                    .color(fg_colour)
                    .font(FontId::monospace(20.0));
                ui.label(rich_text);
            }
        });
    });
}
