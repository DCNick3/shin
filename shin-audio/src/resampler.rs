use kira::{interpolate_frame, Frame};

#[derive(Debug, Clone, Copy, PartialEq)]
struct RecentFrame {
    /// A frame of audio.
    frame: Frame,
    /// The current frame index of the source sound at the
    /// time this frame was pushed to the resampler.
    frame_index: u32,
}

pub(super) struct Resampler {
    frames: [RecentFrame; 4],
}

impl Resampler {
    pub fn new(starting_frame_index: u32) -> Self {
        Self {
            frames: [RecentFrame {
                frame: Frame::ZERO,
                frame_index: starting_frame_index,
            }; 4],
        }
    }

    pub fn push_frame(&mut self, frame: Frame, sample_index: u32) {
        for i in 0..self.frames.len() - 1 {
            self.frames[i] = self.frames[i + 1];
        }
        self.frames[self.frames.len() - 1] = RecentFrame {
            frame,
            frame_index: sample_index,
        };
    }

    pub fn get(&self, fractional_position: f32) -> Frame {
        interpolate_frame(
            self.frames[0].frame,
            self.frames[1].frame,
            self.frames[2].frame,
            self.frames[3].frame,
            fractional_position,
        )
    }

    /// Returns the index of the frame in the source sound
    /// that the user is currently hearing from this resampler.
    ///
    /// This is not the same as the most recently pushed frame.
    /// The user mainly hears a frame between `self.frames[1]` and
    /// `self.frames[2]`. `self.frames[0]` and `self.frames[3]`
    /// are used to provide additional information to the interpolation
    /// algorithm to get a smoother result.
    #[allow(unused)] // TODO: use to implement BGMSYNC
    pub fn current_frame_index(&self) -> u32 {
        self.frames[1].frame_index
    }

    pub fn outputting_silence(&self) -> bool {
        self.frames
            .iter()
            .all(|RecentFrame { frame, .. }| *frame == Frame::ZERO)
    }
}
