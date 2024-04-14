use jack::Frames;

use crate::{
    data_types::tatum::{self, Tatum},
    model::project_time_info::ProjectTimeInfo,
};

use super::sequence_translation::FrameOffset;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerSecond(Frames);

impl From<Frames> for FramesPerSecond {
    fn from(value: Frames) -> Self {
        FramesPerSecond(value)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerBeat(Frames);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerTatum(Frames);

impl From<FramesPerTatum> for Frames {
    fn from(value: FramesPerTatum) -> Self {
        value.0
    }
}

pub(crate) struct TimingInfo {
    pub(crate) frames_per_second: FramesPerSecond,
}

impl TimingInfo {
    pub fn frames_per_beat(&self, time_info: &ProjectTimeInfo) -> FramesPerBeat {
        let beats_per_second: f32 = time_info.bpm.beats_per_second().into();
        FramesPerBeat(((self.frames_per_second.0 as f32) / beats_per_second).round() as u32)
    }

    pub fn frames_per_tatum(&self, time_info: &ProjectTimeInfo) -> FramesPerTatum {
        let frames_per_beat = self.frames_per_beat(&time_info);
        let tatums_per_beat = (tatum::TATUM_SUBDIVDISONS_PER_BAR as u32) / time_info.beats_per_bar;
        FramesPerTatum(frames_per_beat.0 / tatums_per_beat)
    }

    pub fn frames_end_of_bar(&self, time_info: &ProjectTimeInfo) -> FrameOffset {
        FrameOffset::from((self.frames_per_beat(time_info).0 * time_info.beats_per_bar) - 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_types::beats_per_minute::BeatsPerMinute,
        jack::{
            sequence_translation::FrameOffset,
            timing_info::{FramesPerBeat, FramesPerSecond, FramesPerTatum, TimingInfo},
        },
        model::project_time_info::ProjectTimeInfo,
    };

    #[test]
    fn test_frames_per_beat() {
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(30),
        };
        assert_eq!(
            jack_timing_info.frames_per_beat(&project_time_info),
            FramesPerBeat(15)
        )
    }

    #[test]
    fn test_frames_per_tatum() {
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };
        assert_eq!(
            jack_timing_info.frames_per_tatum(&project_time_info),
            FramesPerTatum(5)
        )
    }

    #[test]
    fn test_frames_at_end_of_bar() {
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
            beats_per_bar: 4,
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: FramesPerSecond::from(40),
        };
        assert_eq!(
            jack_timing_info.frames_end_of_bar(&project_time_info),
            FrameOffset::from(79)
        );
    }
}
