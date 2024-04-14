use crate::data_types::beats_per_minute::BeatsPerMinute;

pub(crate) struct ProjectTimeInfo {
    pub(crate) bpm: BeatsPerMinute,
    pub(crate) beats_per_bar: u32,
}

impl Default for ProjectTimeInfo {
    fn default() -> Self {
        Self {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        }
    }
}
