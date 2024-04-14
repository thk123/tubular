use std::{
    sync::{Arc, RwLock},
};

use jack::{AsyncClient, Frames, MidiOut, Port, ProcessHandler};
use midi_msg::MidiMsg;

use crate::model::{
    chord_sequence::ChordSequence, project_state::ProjectState, project_time_info::ProjectTimeInfo,
};

use super::{
    sequence_translation::{self, chord_sequence_to_frame_offset, Event, FrameOffset},
    timing_info::{FramesPerSecond, TimingInfo},
};

pub(crate) struct JackProcessor {
    project_state: Arc<RwLock<ProjectState>>,
    chord_port: Port<MidiOut>,
    jack_timing_info: TimingInfo,
    current_events: Vec<Event>,
}

impl JackProcessor {
    pub(crate) fn activate_async(
        project_state: Arc<RwLock<ProjectState>>,
    ) -> AsyncClient<(), JackProcessor> {
        let (client, _status) =
            jack::Client::new("tubular", jack::ClientOptions::NO_START_SERVER).unwrap();
        let chord_port = client.register_port("chords", jack::MidiOut).unwrap();

        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(client.sample_rate()),
        };

        let starting_events = chord_sequence_to_frame_offset(
            &project_state.read().unwrap().chord_sequence,
            &jack_timing_info,
            &project_state.read().unwrap().time,
        );

        let client_handler = JackProcessor {
            project_state,
            chord_port,
            jack_timing_info,
            current_events: starting_events,
        };

        client.activate_async((), client_handler).unwrap()
    }
}

fn frames_of_next_offset(
    last_frame_time: Frames,
    frames_through_bar: FrameOffset,
    jack_timing_info: &TimingInfo,
    project_timing_info: &ProjectTimeInfo,
) -> Frames {
    let frames_per_bar = jack_timing_info.frames_per_bar(project_timing_info);
    let frames_since_start_of_last_bar = frames_per_bar.frames_through_bar(&last_frame_time);

    let frames_til_next_bar = frames_per_bar - frames_since_start_of_last_bar;
    let start_frame_of_next_bar = last_frame_time + frames_til_next_bar;
    let start_frame_of_current_bar = last_frame_time - frames_since_start_of_last_bar;
    let time_in_current_bar = start_frame_of_current_bar + frames_through_bar;
    let time_in_next_bar = start_frame_of_next_bar + frames_through_bar;
    if time_in_current_bar >= last_frame_time {
        return time_in_current_bar;
    }
    time_in_next_bar
}

fn is_upcoming_event(
    event_time_through_bar: FrameOffset,
    last_frame_time: Frames,
    n_frames: Frames,
    jack_timing_info: &TimingInfo,
    project_timing_info: &ProjectTimeInfo,
) -> bool {
    let event_frame_time = frames_of_next_offset(
        last_frame_time,
        event_time_through_bar,
        jack_timing_info,
        project_timing_info,
    );
    assert!(event_frame_time >= last_frame_time);
    event_frame_time - last_frame_time < n_frames
}

fn get_midi_events_for_next_n_frames(
    last_frame_time: Frames,
    n_frames: Frames,
    current_chord_sequence: &ChordSequence,
    jack_timing_info: &TimingInfo,
    project_timing_info: &ProjectTimeInfo,
) -> Vec<(u32, MidiMsg)> {
    let sequence = sequence_translation::chord_sequence_to_frame_offset(
        current_chord_sequence,
        jack_timing_info,
        project_timing_info,
    );

    // TODO: if the sequence has changed we might have lingering notes that need to be turned off

    let mut upcoming_events: Vec<(u32, MidiMsg)> = sequence
        .iter()
        .filter(|&event| {
            is_upcoming_event(
                event.bar_offset_frames,
                last_frame_time,
                n_frames,
                jack_timing_info,
                project_timing_info,
            )
        })
        .map(|event| {
            let midi_event = match event.event {
                sequence_translation::MidiEvent::NoteOn(note) => MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch1,
                    msg: midi_msg::ChannelVoiceMsg::NoteOn {
                        note: note.into(),
                        velocity: 120,
                    },
                },
                sequence_translation::MidiEvent::NoteOff(note) => MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch1,
                    msg: midi_msg::ChannelVoiceMsg::NoteOff {
                        note: note.into(),
                        velocity: 64,
                    },
                },
            };
            let time = frames_of_next_offset(
                last_frame_time,
                event.bar_offset_frames,
                jack_timing_info,
                project_timing_info,
            );
            assert!(time >= last_frame_time);
            let frames_to_go = time - last_frame_time;
            assert!(frames_to_go < n_frames);
            (frames_to_go, midi_event)
        })
        .collect();

    upcoming_events.sort_by_key(|(time, _midi_message)| *time);
    upcoming_events
}

impl ProcessHandler for JackProcessor {
    fn process(&mut self, _: &jack::Client, _process_scope: &jack::ProcessScope) -> jack::Control {
        let current_project_state = self.project_state.read().unwrap();
        let _sequence = sequence_translation::chord_sequence_to_frame_offset(
            &current_project_state.chord_sequence,
            &self.jack_timing_info,
            &current_project_state.time,
        );

        let upcoming_events = get_midi_events_for_next_n_frames(
            _process_scope.last_frame_time(),
            _process_scope.n_frames(),
            &current_project_state.chord_sequence,
            &self.jack_timing_info,
            &current_project_state.time,
        );

        let mut chord_port_writer = self.chord_port.writer(_process_scope);
        for (time, upcoming_event) in upcoming_events {
            assert!(time < _process_scope.n_frames());
            chord_port_writer
                .write(&jack::RawMidi {
                    time,
                    bytes: &upcoming_event.to_midi(),
                })
                .unwrap();
        }

        jack::Control::Continue
    }
}

#[cfg(test)]
mod tests {

    

    use crate::{
        data_types::{beats_per_minute::BeatsPerMinute, chord_degree::ChordDegree},
        jack::{
            jack_processor::{frames_of_next_offset, is_upcoming_event},
            sequence_translation::FrameOffset,
            timing_info::{FramesPerSecond, TimingInfo},
        },
        model::{
            chord_sequence::{ChordSequence},
            project_time_info::ProjectTimeInfo,
        },
    };

    use super::get_midi_events_for_next_n_frames;

    #[test]
    fn test_frame_offset() {
        // timing is 80 frames a bar

        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };

        assert_eq!(
            frames_of_next_offset(
                90,
                FrameOffset::from(5),
                &jack_timing_info,
                &project_time_info
            ),
            165
        );

        assert_eq!(
            frames_of_next_offset(
                90,
                FrameOffset::from(15),
                &jack_timing_info,
                &project_time_info
            ),
            95
        );

        assert_eq!(
            frames_of_next_offset(
                79,
                FrameOffset::from(0),
                &jack_timing_info,
                &project_time_info
            ),
            80
        );

        assert_eq!(
            frames_of_next_offset(
                80,
                FrameOffset::from(0),
                &jack_timing_info,
                &project_time_info
            ),
            80
        );
        assert_eq!(
            frames_of_next_offset(
                80,
                FrameOffset::from(1),
                &jack_timing_info,
                &project_time_info
            ),
            81
        );
    }

    #[test]
    fn test_is_upcoming_event() {
        // timing is 80 frames a bar

        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };

        assert!(is_upcoming_event(
            FrameOffset::from(0),
            80,
            10,
            &jack_timing_info,
            &project_time_info
        ));
        assert!(is_upcoming_event(
            FrameOffset::from(0),
            71,
            10,
            &jack_timing_info,
            &project_time_info
        ));
        assert!(!is_upcoming_event(
            FrameOffset::from(10),
            80,
            10,
            &jack_timing_info,
            &project_time_info
        ));
    }

    #[test]
    fn test_get_midi_events_for_next_n_frames() {
        // timing is 80 frames a bar, 20 frames a beat, 5 frames a Tatum
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };

        let chord_sequence =
            ChordSequence::new([Some(ChordDegree::I), None, Some(ChordDegree::II)].to_vec())
                .unwrap();

        let events = get_midi_events_for_next_n_frames(
            80,
            10, // processing two tatums
            &chord_sequence,
            &jack_timing_info,
            &project_time_info,
        );

        let times: Vec<u32> = events.iter().map(|(time, _)| *time).collect();
        assert_eq!(times, [0, 0, 0, 5, 5, 5]);

        let midi_statuses: Vec<u8> = events
            .iter()
            .map(|(_, midi_event)| midi_event.to_midi()[0])
            .collect();
        assert_eq!(midi_statuses, [0x90, 0x90, 0x90, 0x80, 0x80, 0x80]);

        let midi_notes: Vec<u8> = events
            .iter()
            .map(|(_, midi_event)| midi_event.to_midi()[1])
            .collect();
        assert_eq!(midi_notes, [60, 64, 67, 60, 64, 67]);
    }

    #[test]
    fn test_get_midi_events_for_next_n_frames_event_from_start_of_next_bar() {
        // timing is 80 frames a bar, 20 frames a beat, 5 frames a Tatum
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };

        let chord_sequence =
            ChordSequence::new([Some(ChordDegree::I), None, Some(ChordDegree::II)].to_vec())
                .unwrap();

        let events = get_midi_events_for_next_n_frames(
            86,
            79, // just shy of a whole bar
            &chord_sequence,
            &jack_timing_info,
            &project_time_info,
        );

        let times: Vec<u32> = events.iter().map(|(time, _)| *time).collect();
        let start_of_next_frame = 160 - 86;
        assert_eq!(
            times,
            [
                4, // Turn on II
                4,
                4,
                9, // Turn of II
                9,
                9,
                start_of_next_frame, // Turn on I at start of next bar
                start_of_next_frame,
                start_of_next_frame
            ]
        );

        let midi_statuses: Vec<u8> = events
            .iter()
            .map(|(_, midi_event)| midi_event.to_midi()[0])
            .collect();
        assert_eq!(
            midi_statuses,
            [0x90, 0x90, 0x90, 0x80, 0x80, 0x80, 0x90, 0x90, 0x90]
        );

        let midi_notes: Vec<u8> = events
            .iter()
            .map(|(_, midi_event)| midi_event.to_midi()[1])
            .collect();
        assert_eq!(midi_notes, [62, 65, 69, 62, 65, 69, 60, 64, 67,]);
    }
}
