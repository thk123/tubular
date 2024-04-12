use std::ops::{Index, IndexMut};

use crate::data_types::{chord_degree::ChordDegree, tatum::{self, Tatum}};

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
