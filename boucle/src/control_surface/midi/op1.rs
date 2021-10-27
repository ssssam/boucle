// Keyboard map for Teenage Engineering OP-1.

use crate::BeatFraction;
use crate::Operation;
use super::note;
use super::MidiControlSurface;
use super::MidiNote;

pub struct Op1 {
}

impl MidiControlSurface for Op1 {
    fn map_midi_note(self: &Self, note: MidiNote) -> Operation {
        match note {
            note::NOTE_F4 => Operation::Jump { offset: BeatFraction::from(-8.0) },
            note::NOTE_Gb4 => Operation::Jump { offset: BeatFraction::from(-4.0) },
            note::NOTE_G4 => Operation::Jump { offset: BeatFraction::from(-2.0) },
            note::NOTE_Ab4 => Operation::Jump { offset: BeatFraction::from(-1.0) },
            note::NOTE_A4 => Operation::Jump { offset: BeatFraction::from(-0.5) },
            note::NOTE_Bb4 => Operation::NoOp,
            note::NOTE_B4 => Operation::Jump { offset: BeatFraction::from(-0.25) },

            note::NOTE_C5 => Operation::Repeat { loop_size: BeatFraction::from(0.0625) },
            note::NOTE_Db5 => Operation::Repeat { loop_size: BeatFraction::from(0.125) },
            note::NOTE_D5 => Operation::Repeat { loop_size: BeatFraction::from(0.25) },
            note::NOTE_Eb5 => Operation::Repeat { loop_size: BeatFraction::from(0.5) },

            note::NOTE_E5 => Operation::Reverse,
            note::NOTE_F5 => Operation::NoOp,

            note::NOTE_Gb5 => Operation::Repeat { loop_size: BeatFraction::from(1.0) },
            note::NOTE_G5 => Operation::Repeat { loop_size: BeatFraction::from(2.0) },
            note::NOTE_Ab5 => Operation::Repeat { loop_size: BeatFraction::from(4.0) },
            note::NOTE_A5 => Operation::Repeat { loop_size: BeatFraction::from(8.0) },

            note::NOTE_Bb5 => Operation::NoOp,

            note::NOTE_B5 => Operation::Jump { offset: BeatFraction::from(0.25) },
            note::NOTE_C6 => Operation::Jump { offset: BeatFraction::from(0.5) },
            note::NOTE_Db6 => Operation::Jump { offset: BeatFraction::from(1.0) },
            note::NOTE_D6 => Operation::Jump { offset: BeatFraction::from(2.0) },
            note::NOTE_Eb6 => Operation::Jump { offset: BeatFraction::from(4.0) },
            note::NOTE_E6 => Operation::Jump { offset: BeatFraction::from(8.0) },

            _ => Operation::NoOp,
        }
    }
}
