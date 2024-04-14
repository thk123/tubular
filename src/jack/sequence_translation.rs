use std::ops::Mul;

use crate::{
    data_types::{
        chord_degree::{self, ChordDegree},
        note::Note,
        tatum::Tatum,
    },
    model::{chord_sequence::ChordSequence, project_time_info::ProjectTimeInfo},
    music_theory::chords::chord_degreee_to_notes,
};

use super::timing_info::{FramesPerTatum, TimingInfo};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub(crate) struct FrameOffset(u32);

impl From<u32> for FrameOffset {
    fn from(value: u32) -> Self {
        FrameOffset(value)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum MidiEvent {
    NoteOn(Note),
    NoteOff(Note),
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Event {
    bar_offset_frames: FrameOffset,
    event: MidiEvent,
}

impl Mul<FramesPerTatum> for Tatum {
    type Output = FrameOffset;

    fn mul(self, rhs: FramesPerTatum) -> Self::Output {
        let tatum_index: usize = self.into();
        let frames_per_tatum: u32 = rhs.into();
        FrameOffset(tatum_index as u32 * frames_per_tatum)
    }
}

fn get_time_of_event_relative_to_bar(
    tatums_into_bar: Tatum,
    project_time_info: &ProjectTimeInfo,
    timing_info: &TimingInfo,
) -> FrameOffset {
    tatums_into_bar * timing_info.frames_per_tatum(&project_time_info)
}

type EventTypeCreator = fn(Note) -> MidiEvent;

fn event_for_chord(
    chord: &ChordDegree,
    time: FrameOffset,
    event_type: EventTypeCreator,
) -> [Event; 3] {
    let notes = chord_degreee_to_notes(chord);
    notes.map(|note| event_type(note)).map(|midi_event| Event {
        bar_offset_frames: time,
        event: midi_event,
    })
}

pub(crate) fn chord_sequence_to_frame_offset(
    sequence: &ChordSequence,
    timing_info: &TimingInfo,
    project_time_info: &ProjectTimeInfo,
) -> Vec<Event> {
    let mut last_chord = None;
    let mut events = vec![];
    for (index, chord) in sequence.iter().enumerate() {
        let tatum = Tatum::try_from(index).unwrap();
        let event_time = get_time_of_event_relative_to_bar(tatum, &project_time_info, &timing_info);
        if let Some(last_chord_played) = last_chord {
            let midi_events = event_for_chord(last_chord_played, event_time, MidiEvent::NoteOff);
            events.extend(midi_events);
            last_chord = None;
        }

        if let Some(chord_played) = chord {
            last_chord = Some(chord_played);
            let midi_events = event_for_chord(chord_played, event_time, MidiEvent::NoteOn);
            events.extend(midi_events);
        }
    }
    if let Some(last_chord_played) = last_chord {
        let midi_events = event_for_chord(
            last_chord_played,
            timing_info.frames_end_of_bar(&project_time_info),
            MidiEvent::NoteOff,
        );
        events.extend(midi_events);
    }
    events
}

#[cfg(test)]
mod tests {
    use crate::{
        data_types::{
            beats_per_minute::BeatsPerMinute, chord_degree::ChordDegree, note::Note, tatum::Tatum,
        },
        jack::{
            sequence_translation::{chord_sequence_to_frame_offset, Event, FrameOffset, MidiEvent},
            timing_info::{FramesPerSecond, TimingInfo},
        },
        model::{chord_sequence::ChordSequence, project_time_info::ProjectTimeInfo},
    };

    #[test]
    fn test_chord_sequence_to_frame_offset() {
        let sequence = ChordSequence::new(vec![Some(ChordDegree::I)]).unwrap();
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };
        let events =
            chord_sequence_to_frame_offset(&sequence, &jack_timing_info, &project_time_info);

        assert_eq!(
            events,
            vec![
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(60))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(64))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(67))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(60))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(64))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(67))
                },
            ]
        )
    }

    #[test]
    fn test_chord_sequence_chords_in_adjacent_tatums_turn_off_old_chord() {
        let sequence =
            ChordSequence::new(vec![Some(ChordDegree::I), Some(ChordDegree::II)]).unwrap();
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };
        let events =
            chord_sequence_to_frame_offset(&sequence, &jack_timing_info, &project_time_info);

        assert_eq!(
            events,
            vec![
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(60))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(64))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(0),
                    event: MidiEvent::NoteOn(Note::from(67))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(60))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(64))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOff(Note::from(67))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOn(Note::from(62))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOn(Note::from(65))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(5),
                    event: MidiEvent::NoteOn(Note::from(69))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(10),
                    event: MidiEvent::NoteOff(Note::from(62))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(10),
                    event: MidiEvent::NoteOff(Note::from(65))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(10),
                    event: MidiEvent::NoteOff(Note::from(69))
                },
            ]
        )
    }

    #[test]
    fn test_chord_sequence_with_chord_at_end_on() {
        let mut sequence = ChordSequence::default();
        sequence[Tatum::try_from(15).unwrap()] = Some(ChordDegree::II);

        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };
        let events =
            chord_sequence_to_frame_offset(&sequence, &jack_timing_info, &project_time_info);

        assert_eq!(
            events,
            vec![
                Event {
                    bar_offset_frames: FrameOffset::from(75),
                    event: MidiEvent::NoteOn(Note::from(62))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(75),
                    event: MidiEvent::NoteOn(Note::from(65))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(75),
                    event: MidiEvent::NoteOn(Note::from(69))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(79),
                    event: MidiEvent::NoteOff(Note::from(62))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(79),
                    event: MidiEvent::NoteOff(Note::from(65))
                },
                Event {
                    bar_offset_frames: FrameOffset::from(79),
                    event: MidiEvent::NoteOff(Note::from(69))
                },
            ]
        )
    }
}
