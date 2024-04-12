// TODO: in future Tatum should be derived from the ChordSequence
pub(crate) const TATUM_SUBDIVDISONS_PER_BAR: usize = 16;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tatum(usize);

impl TryFrom<usize> for Tatum {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= TATUM_SUBDIVDISONS_PER_BAR {
            return Err("Tatum index that is larger than number of subdivisions");
        }
        return Ok(Tatum(value));
    }
}

impl From<Tatum> for usize
{
    fn from(value: Tatum) -> Self {
        value.0
    }
}

impl Tatum {
    pub fn add(&self, offset: i32) -> Tatum {
        let new_selected_chord = self.0 as i32 + offset;
        let new_selected_modulo_chord = new_selected_chord.rem_euclid(TATUM_SUBDIVDISONS_PER_BAR as i32);
        Tatum::try_from(new_selected_modulo_chord as usize).unwrap()
    }
}

#[test]
fn test_create_valid_tatums()
{
    assert_eq!(Tatum::try_from(0).unwrap().0, 0);
    assert_eq!(Tatum::try_from(15).unwrap().0, 15);
}

#[test]
fn test_create_invalid_tatum()
{
    assert!(Tatum::try_from(16).is_err());
}

#[test]
fn add_offset_to_tatum_increments()
{
    assert_eq!(Tatum(1).add(1).0, 2);
}

#[test]
fn subtract_offset_to_tatum_decrements()
{
    assert_eq!(Tatum(1).add(-1).0, 0);
}

#[test]
fn subtract_offset_to_tatum_wraps_around()
{
    assert_eq!(Tatum(0).add(-1).0, 15);
}

#[test]
fn add_offset_to_tatum_wraps_around()
{
    assert_eq!(Tatum(15).add(1).0, 0);
}

#[test]
fn add_large_offset_to_tatum_wraps_around()
{
    assert_eq!(Tatum(15).add(32).0, 15);
}
