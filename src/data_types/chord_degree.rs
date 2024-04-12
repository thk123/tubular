use std::fmt;


#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ChordDegree {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
}

impl fmt::Display for ChordDegree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
