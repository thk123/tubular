use eframe::egui::{self, Color32, FontId, RichText};

use crate::{data_types::tatum::Tatum, view_model::chord_sequencer_vm::ChordSequencerVm};

pub(crate) fn update(vm: &mut ChordSequencerVm, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (index, chord) in vm
                .chord_sequence()
                .iter()
                .enumerate()
            {
                let text = match chord {
                    Some(c) => c.to_string(),
                    None => ".".to_string(),
                };
                let centred_text = format!("{:^3}", text);
                let (bg_colour, fg_colour) = if Tatum::try_from(index).unwrap() == vm.selected_chord() {
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
