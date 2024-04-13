use jack::Frames;

use crate::model::project_time_info::ProjectTimeInfo;

struct TimingInfo {
    frames_per_second: Frames,
}

impl TimingInfo {
    pub fn frames_per_beat(&self, time_info: &ProjectTimeInfo) -> Frames {
        let beats_per_second: f32 = time_info.bpm.beats_per_second().into();
        ((self.frames_per_second as f32) / beats_per_second).round() as Frames
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_types::beats_per_minute::BeatsPerMinute, jack::timing_info::TimingInfo,
        model::project_time_info::ProjectTimeInfo,
    };

    #[test]
    fn test_frames_per_beat() {
        let project_time_info = ProjectTimeInfo {
            bpm: BeatsPerMinute::from(120),
        };
        let jack_timing_info = TimingInfo {
            frames_per_second: 30,
        };
        assert_eq!(jack_timing_info.frames_per_beat(&project_time_info), 15)
    }
}
