// Predefined patterns you can play.

/*pub trait Pattern {
    fn get() -> 
}*/
use crate::SamplePosition;
use crate::ops;
use crate::op_sequence;

type Beats = u32;

pub trait Pattern {
    fn op_for_beat(self: &mut Self, beat: Beats, beats_to_samples: u32) -> Option<op_sequence::Entry>;
}

pub struct CheckersReverse {
    period: Beats,
    duration: Beats,

}

impl CheckersReverse {
    fn new(_bpm: Beats, _sample_rate: i32) -> CheckersReverse {
        CheckersReverse {
            period: 2,   // Reverse every 2nd beat.
            duration: 1,
        }
    }
}

impl Pattern for CheckersReverse {
    fn op_for_beat(self: &mut Self, beat: Beats, beats_to_samples: u32) -> Option<op_sequence::Entry> {
        if (beat % self.period) == 0 {
            Some(op_sequence::Entry {
                start: (beat * beats_to_samples) as SamplePosition,
                duration: Some((self.duration * beats_to_samples) as SamplePosition),
                op: Box::new(ops::ReverseOp {}),
            })
        } else {
            None
        }
    }
}

/*
pub struct RandomReverse {
    probability: f32,
    min_duration: Beats,
    max_duration: Beats,
}

impl RandomReverse {
    fn new() -> RandomReverse {
        RandomReverse { probability = 50; }    // 50% chance of reverse
    }
}

impl Pattern for RandomReverse {
    fn op_for_beat(self: &mut Self, beat: Beats, beats_to_blocks: i32) -> Option<op_sequence::Entry> {
        if (beat % period) == 0 {
            Some(op_sequence::Entry {
                start: beat * self.beats_to_blocks,
                duration: self.duration * self.beats_to_blocks,
                op: ops::ReverseOp {};
            })
        } else {
            None()
        }
    }
}
impl Iterator for RandomReverse {

}
*/
