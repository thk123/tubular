use crate::data_types::{chord_degree::ChordDegree, tatum::Tatum};

use super::{chord_sequence::ChordSequence, project_time_info::ProjectTimeInfo};

#[derive(Default)]
pub(crate) struct ProjectState {
    pub chord_sequence: ChordSequence,
    pub time: ProjectTimeInfo,
}

impl ProjectState {
    pub fn update_chord_sequence(&mut self, chord_position: Tatum, new_chord: Option<ChordDegree>) {
        self.chord_sequence[chord_position] = new_chord;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_types::{chord_degree::ChordDegree, tatum::Tatum},
        model::{
            chord_sequence::ChordSequence, project_state::ProjectState,
            project_time_info::ProjectTimeInfo,
        },
    };

    #[test]
    fn update_chord_sequence_with_new_chord() {
        let mut project_state = ProjectState {
            chord_sequence: ChordSequence::default(),
            time: ProjectTimeInfo::default(),
        };
        let chord_pos = Tatum::try_from(0).unwrap();
        project_state.update_chord_sequence(chord_pos, Some(ChordDegree::II));
        assert_eq!(
            project_state.chord_sequence[chord_pos],
            Some(ChordDegree::II)
        );
    }

    #[test]
    fn update_chord_sequence_with_removing_chord() {
        let mut project_state = ProjectState {
            chord_sequence: ChordSequence::new(Vec::from([Some(ChordDegree::II)])).unwrap(),
            time: ProjectTimeInfo::default(),
        };
        let chord_pos = Tatum::try_from(0).unwrap();
        project_state.update_chord_sequence(chord_pos, None);
        assert_eq!(project_state.chord_sequence[chord_pos], None);
    }
}
