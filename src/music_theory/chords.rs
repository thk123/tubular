use crate::data_types::{chord_degree::ChordDegree, note::Note};

fn major_triad(root_note: Note) -> [Note; 3] {
    [root_note, root_note + 4, root_note + 7]
}

fn minor_triad(root_note: Note) -> [Note; 3] {
    [root_note, root_note + 3, root_note + 7]
}

fn diminished_triad(root_note: Note) -> [Note; 3] {
    [root_note, root_note + 3, root_note + 6]
}

pub(crate) fn chord_degreee_to_notes(chord_degree: &ChordDegree) -> [Note; 3] {
    let root_note = Note::from(60);
    return match chord_degree {
        ChordDegree::I => major_triad(root_note),
        ChordDegree::II => minor_triad(root_note + 2),
        ChordDegree::III => minor_triad(root_note + 4),
        ChordDegree::IV => major_triad(root_note + 5),
        ChordDegree::V => major_triad(root_note + 7),
        ChordDegree::VI => minor_triad(root_note + 9),
        ChordDegree::VII => diminished_triad(root_note + 11),
    };
}
