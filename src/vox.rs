//! Sample-clocked VOX hold advancement after caller-owned qualification.

/// Remaining hold and the previous emitted sample's in-span native offset.
#[derive(Default)]
pub struct State {
    remaining_frames: u32,
    prior_offset: u32,
}

impl State {
    /// Consume elapsed native frames and reset the hold at a qualifying sample.
    pub fn advance(&mut self, native_offset: u32, qualified: bool, hang_frames: u32) {
        let elapsed = native_offset.saturating_sub(self.prior_offset);
        self.remaining_frames = self.remaining_frames.saturating_sub(elapsed);
        if qualified {
            self.remaining_frames = hang_frames;
        }
        self.prior_offset = native_offset;
    }

    /// Consume the trailing native frames and prepare for the next span.
    pub fn finish(&mut self, native_frames: u32) -> bool {
        let trailing = native_frames.saturating_sub(self.prior_offset);
        self.remaining_frames = self.remaining_frames.saturating_sub(trailing);
        self.prior_offset = 0;
        self.remaining_frames != 0
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::State;

    #[test]
    fn hold_countdown_matches_irregular_partitioning() {
        let mut whole = State::default();
        whole.advance(5, true, 10);
        assert!(whole.finish(8));
        assert!(!whole.finish(7));

        let mut split = State::default();
        split.advance(5, true, 10);
        assert!(split.finish(5));
        assert!(split.finish(3));
        assert!(!split.finish(7));
    }

    #[test]
    fn unqualified_samples_consume_hold_without_resetting_it() {
        let mut state = State::default();
        assert!(!state.finish(0));
        state.advance(2, true, 10);
        state.advance(5, false, 10);
        assert!(state.finish(10));
        assert!(!state.finish(2));
    }
}
