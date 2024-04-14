use std::ops::{Add, Sub};

use eframe::Frame;
use jack::Frames;

use crate::{
    data_types::tatum::{self, Tatum},
    model::project_time_info::ProjectTimeInfo,
};

use super::sequence_translation::FrameOffset;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerSecond(usize);

impl From<usize> for FramesPerSecond {
    fn from(value: usize) -> Self {
        FramesPerSecond(value)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerBar(Frames);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerBeat(Frames);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct FramesPerTatum(Frames);

impl From<FramesPerTatum> for Frames {
    fn from(value: FramesPerTatum) -> Self {
        value.0
    }
}

impl From<FramesPerBar> for Frames {
    fn from(value: FramesPerBar) -> Self {
        value.0
    }
}

impl FramesPerBar {
    pub(crate) fn frames_through_bar(&self, total_frames: &Frames) -> FrameOffset {
        let frames_through_bar = total_frames.rem_euclid(self.0);
        FrameOffset::from(frames_through_bar)
    }
}

impl Sub<FrameOffset> for FramesPerBar {
    type Output = FrameOffset;

    fn sub(self, rhs: FrameOffset) -> Self::Output {
        let rhs_as_number: u32 = rhs.into();
        if rhs_as_number > self.0 {
            panic!("Frame offset bigger than frames per bar");
        }
        let offset: u32 = self.0 - rhs_as_number;
        FrameOffset::from(offset)
    }
}

impl Add<FrameOffset> for Frames {
    type Output = Frames;

    fn add(self, rhs: FrameOffset) -> Self::Output {
        let offset: u32 = rhs.into();
        self + offset
    }
}

impl Sub<FrameOffset> for Frames {
    type Output = Frames;

    fn sub(self, rhs: FrameOffset) -> Self::Output {
        let offset: u32 = rhs.into();
        self - offset
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

    pub fn frames_per_bar(&self, time_info: &ProjectTimeInfo) -> FramesPerBar {
        let frames_per_beat = self.frames_per_beat(&time_info);
        FramesPerBar(frames_per_beat.0 * time_info.beats_per_bar)
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
