use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use jack::{AsyncClient, Frames, MidiOut, Port, ProcessHandler};
use midi_msg::MidiMsg;

use crate::{
    data_types::note::Note,
    model::{project_state::ProjectState, project_time_info::ProjectTimeInfo},
};

use super::{
    sequence_translation::{self, chord_sequence_to_frame_offset, Event, FrameOffset, MidiEvent},
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

fn notes_on_at_point(sequence: &Vec<Event>, frames_through_bar: FrameOffset) -> HashSet<Note> {
    let mut live_notes = HashSet::new();
    for event in sequence
        .iter()
        .filter(|e| e.bar_offset_frames < frames_through_bar)
    {
        match &event.event {
            sequence_translation::MidiEvent::NoteOn(note) => live_notes.insert(*note),
            sequence_translation::MidiEvent::NoteOff(note) => live_notes.remove(note),
        };
    }
    live_notes
}

fn lingering_notes(
    old_events: &Vec<Event>,
    new_events: &Vec<Event>,
    frames_through_bar: FrameOffset,
) -> HashSet<Note> {
    let old_notes_on = notes_on_at_point(old_events, frames_through_bar);
    let new_notes_on = notes_on_at_point(new_events, frames_through_bar);
    return old_notes_on.difference(&new_notes_on).cloned().collect();
}

fn ghost_notes(
    old_events: &Vec<Event>,
    new_events: &Vec<Event>,
    frames_through_bar: FrameOffset,
) -> HashSet<Note> {
    let old_notes_on = notes_on_at_point(old_events, frames_through_bar);
    let new_notes_on = notes_on_at_point(new_events, frames_through_bar);
    return new_notes_on.difference(&old_notes_on).cloned().collect();
}

fn translate_to_raw(event: &MidiEvent) -> MidiMsg {
    match event {
        sequence_translation::MidiEvent::NoteOn(note) => MidiMsg::ChannelVoice {
            channel: midi_msg::Channel::Ch1,
            msg: midi_msg::ChannelVoiceMsg::NoteOn {
                note: note.clone().into(),
                velocity: 120,
            },
        },
        sequence_translation::MidiEvent::NoteOff(note) => MidiMsg::ChannelVoice {
            channel: midi_msg::Channel::Ch1,
            msg: midi_msg::ChannelVoiceMsg::NoteOff {
                note: note.clone().into(),
                velocity: 64,
            },
        },
    }
}

fn get_midi_events_for_next_n_frames(
    last_frame_time: Frames,
    n_frames: Frames,
    sequence: &Vec<Event>,
    old_sequence: &Vec<Event>,
    jack_timing_info: &TimingInfo,
    project_timing_info: &ProjectTimeInfo,
) -> Vec<(u32, MidiEvent)> {
    let mut upcoming_events: Vec<(u32, MidiEvent)> = vec![];
    let frames_through_bar = jack_timing_info
        .frames_per_bar(project_timing_info)
        .frames_through_bar(&last_frame_time);
    let lingering_notes = lingering_notes(old_sequence, sequence, frames_through_bar);
    let ghost_notes = ghost_notes(old_sequence, sequence, frames_through_bar);

    let old_note_off_messages = lingering_notes
        .iter()
        .map(|note| MidiEvent::NoteOff(note.clone()))
        .map(|midi_msg| (0, midi_msg));
    upcoming_events.extend(old_note_off_messages);

    let upcoming_notes = sequence
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
        .filter(|event| {
            if let MidiEvent::NoteOff(note_off) = event.event {
                return !ghost_notes.contains(&note_off);
            }
            true
        })
        .map(|event| {
            let time = frames_of_next_offset(
                last_frame_time,
                event.bar_offset_frames,
                jack_timing_info,
                project_timing_info,
            );
            assert!(time >= last_frame_time);
            let frames_to_go = time - last_frame_time;
            assert!(frames_to_go < n_frames);
            (frames_to_go, event.event.clone())
        });
    upcoming_events.extend(upcoming_notes);
    upcoming_events.sort_by_key(|(time, _midi_message)| *time);
    upcoming_events
}

impl ProcessHandler for JackProcessor {
    fn process(&mut self, _: &jack::Client, _process_scope: &jack::ProcessScope) -> jack::Control {
        let current_project_state = self.project_state.read().unwrap();
        let sequence = sequence_translation::chord_sequence_to_frame_offset(
            &current_project_state.chord_sequence,
            &self.jack_timing_info,
            &current_project_state.time,
        );

        let upcoming_events = get_midi_events_for_next_n_frames(
            _process_scope.last_frame_time(),
            _process_scope.n_frames(),
            &sequence,
            &self.current_events,
            &self.jack_timing_info,
            &current_project_state.time,
        );

        let mut chord_port_writer = self.chord_port.writer(_process_scope);
        for (time, upcoming_event) in upcoming_events {
            assert!(time < _process_scope.n_frames());
            let midi_msg = translate_to_raw(&upcoming_event);
            chord_port_writer
                .write(&jack::RawMidi {
                    time,
                    bytes: &midi_msg.to_midi(),
                })
                .unwrap();
        }

        jack::Control::Continue
    }
}

#[cfg(test)]
mod tests {

    use std::{collections::HashSet, vec};

    use crate::{
        data_types::{beats_per_minute::BeatsPerMinute, note::Note},
        jack::{
            jack_processor::{
                frames_of_next_offset, ghost_notes, is_upcoming_event, lingering_notes,
                notes_on_at_point,
            },
            sequence_translation::{Event, FrameOffset, MidiEvent},
            timing_info::{FramesPerSecond, TimingInfo},
        },
        model::project_time_info::ProjectTimeInfo,
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

        let event_for_bar = vec![
            Event {
                event: MidiEvent::NoteOn(Note::from(60)),
                bar_offset_frames: FrameOffset::from(0),
            },
            Event {
                event: MidiEvent::NoteOff(Note::from(60)),
                bar_offset_frames: FrameOffset::from(5),
            },
        ];

        let events = get_midi_events_for_next_n_frames(
            80,
            10, // processing two tatums
            &event_for_bar,
            &vec![],
            &jack_timing_info,
            &project_time_info,
        );

        assert_eq!(
            events,
            vec![
                (0, MidiEvent::NoteOn(Note::from(60))),
                (5, MidiEvent::NoteOff(Note::from(60))),
            ]
        );
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

        let event_for_bar = vec![
            Event {
                event: MidiEvent::NoteOn(Note::from(60)),
                bar_offset_frames: FrameOffset::from(0),
            },
            Event {
                event: MidiEvent::NoteOff(Note::from(60)),
                bar_offset_frames: FrameOffset::from(5),
            },
            Event {
                event: MidiEvent::NoteOn(Note::from(62)),
                bar_offset_frames: FrameOffset::from(10),
            },
            Event {
                event: MidiEvent::NoteOff(Note::from(62)),
                bar_offset_frames: FrameOffset::from(15),
            },
        ];

        let events = get_midi_events_for_next_n_frames(
            86,
            79, // just shy of a whole bar
            &event_for_bar,
            &vec![],
            &jack_timing_info,
            &project_time_info,
        );

        let start_of_next_frame = 160 - 86;
        assert_eq!(
            events,
            vec![
                (4, MidiEvent::NoteOn(Note::from(62))),  // Turn on 62
                (9, MidiEvent::NoteOff(Note::from(62))), // Turn off 62
                (start_of_next_frame, MidiEvent::NoteOn(Note::from(60))), // Turn on 60 at start of next bar
            ]
        );
    }

    #[test]
    fn test_get_midi_events_for_next_n_frames_turns_off_old_note() {
        // timing is 80 frames a bar, 20 frames a beat, 5 frames a Tatum
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };

        let old_events = vec![
            Event {
                event: MidiEvent::NoteOn(Note::from(70)),
                bar_offset_frames: FrameOffset::from(0),
            },
            Event {
                event: MidiEvent::NoteOff(Note::from(70)),
                bar_offset_frames: FrameOffset::from(5),
            },
        ];

        let event_for_bar = vec![
            Event {
                event: MidiEvent::NoteOn(Note::from(60)),
                bar_offset_frames: FrameOffset::from(0),
            },
            Event {
                event: MidiEvent::NoteOff(Note::from(60)),
                bar_offset_frames: FrameOffset::from(5),
            },
        ];

        let events = get_midi_events_for_next_n_frames(
            83,
            5,
            &event_for_bar,
            &old_events,
            &jack_timing_info,
            &project_time_info,
        );

        assert_eq!(events, vec![(0, MidiEvent::NoteOff(Note::from(70)))]);
    }

    #[test]
    fn test_notes_on_at_point_before_first_event() {
        let notes_on = notes_on_at_point(
            &vec![Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            }],
            FrameOffset::from(0),
        );
        assert_eq!(notes_on, HashSet::new());
    }

    #[test]
    fn test_notes_on_at_point_after_first_event() {
        let notes_on = notes_on_at_point(
            &vec![Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            }],
            FrameOffset::from(1),
        );
        assert_eq!(notes_on, HashSet::from([Note::from(60)]));
    }

    #[test]
    fn test_notes_on_after_point_after_first_on_and_off() {
        let sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(1),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];
        let notes_on = notes_on_at_point(&sequence, FrameOffset::from(2));
        assert_eq!(notes_on, HashSet::from([]));
    }

    #[test]
    fn test_notes_on_at_point_after_first_on_and_off() {
        let sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(1),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];
        let notes_on = notes_on_at_point(&sequence, FrameOffset::from(1));
        assert_eq!(notes_on, HashSet::from([Note::from(60)]));
    }

    #[test]
    fn test_notes_on_at_point_turn_two_notes_on_one_note_off() {
        let sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(62)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(1),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];
        let notes_on = notes_on_at_point(&sequence, FrameOffset::from(2));
        assert_eq!(notes_on, HashSet::from([Note::from(62)]));
    }

    #[test]
    fn test_lingering_notes_turns_old_notes_off() {
        let old_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(1),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];

        let new_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(61)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(1),
                event: MidiEvent::NoteOn(Note::from(61)),
            },
        ];

        assert_eq!(
            lingering_notes(&old_sequence, &new_sequence, FrameOffset::from(1)),
            HashSet::from([Note::from(60)])
        );
    }

    #[test]
    fn test_lingering_notes_halfway_through_note() {
        let old_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(2),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(4),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(6),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];

        let new_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(4),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(6),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];
        assert_eq!(
            lingering_notes(&old_sequence, &new_sequence, FrameOffset::from(1)),
            HashSet::from([Note::from(60)])
        );
    }

    #[test]
    fn test_ghost_note_includes_note_playing() {
        let old_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(70)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(5),
                event: MidiEvent::NoteOff(Note::from(70)),
            },
        ];

        let new_sequence = vec![
            Event {
                bar_offset_frames: FrameOffset::from(0),
                event: MidiEvent::NoteOn(Note::from(60)),
            },
            Event {
                bar_offset_frames: FrameOffset::from(6),
                event: MidiEvent::NoteOff(Note::from(60)),
            },
        ];
        assert_eq!(
            ghost_notes(&old_sequence, &new_sequence, FrameOffset::from(1)),
            HashSet::from([Note::from(60)])
        );
    }
}
