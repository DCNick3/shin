#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FrameId(u32);

impl FrameId {
    #[inline]
    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

pub struct UpdateTracker {
    frame_id: FrameId,
}

impl UpdateTracker {
    #[inline]
    pub fn new() -> Self {
        Self {
            frame_id: FrameId::default(),
        }
    }

    #[inline]
    pub fn needs_update(&self, frame_id: FrameId) -> bool {
        self.frame_id < frame_id
    }

    #[inline]
    pub fn update(&mut self, frame_id: FrameId) -> bool {
        let result = self.needs_update(frame_id);

        self.frame_id = frame_id;

        result
    }
}
