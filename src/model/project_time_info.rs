use crate::data_types::beats_per_minute::BeatsPerMinute;

pub(crate) struct ProjectTimeInfo {
    pub(crate) bpm: BeatsPerMinute,
}

impl Default for ProjectTimeInfo {
    fn default() -> Self {
        Self {
            bpm: BeatsPerMinute::from(120),
        }
    }
}
