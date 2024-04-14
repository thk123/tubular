use std::ops::Add;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub(crate) struct Note(u8);

impl Add<u8> for Note {
    type Output = Note;

    fn add(self, rhs: u8) -> Self::Output {
        Note(self.0 + rhs)
    }
}

impl From<u8> for Note {
    fn from(value: u8) -> Self {
        Note(value)
    }
}

impl From<Note> for u8 {
    fn from(value: Note) -> Self {
        value.0
    }
}
