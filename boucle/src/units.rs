use std::convert::From;
use std::fmt;

/// This is the sample format used inside the audio engine.
pub type Sample = f32;

pub type SamplePosition = usize;
pub type SampleOffset = isize;

/// Fixed point representation of a 16th of a beat.
///
/// Corresponds to 𝅘𝅥𝅱 (64th note / hemidemisemiquaver)
///
/// We use this instead of f32, as the latter cannot be a valid hashmap key.
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct BeatFraction {
    value: i32,
}

impl BeatFraction {
    pub fn as_sample_offset(self: &Self, beat_fraction_to_samples: f32) -> SampleOffset {
        (self.value as f32 * beat_fraction_to_samples).floor() as SampleOffset
    }

    pub fn as_sample_position(self: &Self, beat_fraction_to_samples: f32) -> SamplePosition {
        (self.value as f32 * beat_fraction_to_samples).floor() as SamplePosition
    }
}

impl From<f32> for BeatFraction {
    fn from(value: f32) -> Self {
        BeatFraction { value: (value * 16.0).floor() as i32 }
    }
}

impl fmt::Display for BeatFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
