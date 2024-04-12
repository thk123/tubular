use std::{ops::{Index, IndexMut}, slice::Iter};

use crate::data_types::{chord_degree::ChordDegree, tatum::{self, Tatum, TATUM_SUBDIVDISONS_PER_BAR}};

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ChordSequence {
    chords: Vec<Option<ChordDegree>>,
}

impl Default for ChordSequence {
    fn default() -> Self {
        Self {
            chords: vec![None; tatum::TATUM_SUBDIVDISONS_PER_BAR],
        }
    }
}

impl ChordSequence{
    pub fn new(chords: Vec<Option<ChordDegree>>) -> Result<ChordSequence, &'static str>{
        if chords.len() > TATUM_SUBDIVDISONS_PER_BAR { return Err("Invalid chord sequence"); }
        if chords.len() == TATUM_SUBDIVDISONS_PER_BAR { Ok(ChordSequence{chords})}
        else {
            let mut chord_sequence = ChordSequence::default();
            for (index, &c) in chords.iter().enumerate()
            {
                chord_sequence.chords[index] = c;
            }
            Ok(chord_sequence)
        }
    }

    pub fn iter(&self) -> Iter<Option<ChordDegree>> {
        self.chords.iter()
    }
}

#[test]
fn new_chord_sequence_from_too_long_array() {
    assert!(ChordSequence::new(Vec::from([None; 17])).is_err());
}

#[test]
fn new_chord_sequence_from_right_length_array() {
    assert_eq!(ChordSequence::new(Vec::from([Some(ChordDegree::II); 16])).unwrap().chords, Vec::from([Some(ChordDegree::II); 16]));
}

#[test]
fn new_chord_sequence_from_short_array() {
    let mut expected_array = Vec::from([Some(ChordDegree::II); 4]);
    expected_array.extend([None; 12]);
    assert_eq!(ChordSequence::new(Vec::from([Some(ChordDegree::II); 4])).unwrap().chords, expected_array);
}

impl Index<Tatum> for ChordSequence {
    type Output = Option<ChordDegree>;

    fn index(&self, index: Tatum) -> &Self::Output {
        let tatum_as_usize: usize = index.into();
        &self.chords[tatum_as_usize]
    }
}

impl IndexMut<Tatum> for ChordSequence {
    fn index_mut(&mut self, index: Tatum) -> &mut Self::Output {
        let tatum_as_usize: usize = index.into();
        &mut self.chords[tatum_as_usize]
    }
}

#[test]
fn make_chord_sequence_right_length()
{
    assert_eq!(ChordSequence::default().chords.len(), 16);
}

#[test]
fn get_chord_from_sequence()
{
    let sequence = ChordSequence{chords: vec![Some(ChordDegree::I)]};
    assert_eq!(sequence[Tatum::try_from(0).unwrap()], Some(ChordDegree::I));
}

#[test]
fn set_chord_mutates_chord()
{
    let mut sequence = ChordSequence::default();
    sequence[Tatum::try_from(0).unwrap()] = Some(ChordDegree::I);
    assert_eq!(sequence.chords[0], Some(ChordDegree::I));
}
