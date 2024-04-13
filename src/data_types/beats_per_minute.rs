pub(crate) struct BeatsPerMinute(f32);
pub(crate) struct BeatsPerSecond(f32);

impl From<u32> for BeatsPerMinute {
    fn from(value: u32) -> Self {
        BeatsPerMinute(value as f32)
    }
}

impl BeatsPerMinute {
    pub(crate) fn beats_per_second(&self) -> BeatsPerSecond {
        BeatsPerSecond(self.0 / 60.0)
    }
}

impl From<BeatsPerSecond> for f32 {
    fn from(value: BeatsPerSecond) -> Self {
        value.0
    }
}
