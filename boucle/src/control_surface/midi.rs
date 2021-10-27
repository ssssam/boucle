// Helpers to map various interface types to Boucle operations.

pub mod op1;

use crate::event::StateChange;
use crate::ops::Operation;

type MidiNote = u8;

#[allow(non_upper_case_globals)]
#[allow(unused)]
mod note {
    use super::MidiNote;

    pub const NOTE_C4: MidiNote = 48;
    pub const NOTE_Db4: MidiNote = 49;
    pub const NOTE_D4: MidiNote = 50;
    pub const NOTE_Eb4: MidiNote = 51;
    pub const NOTE_E4: MidiNote = 52;
    pub const NOTE_F4: MidiNote = 53;
    pub const NOTE_Gb4: MidiNote = 54;
    pub const NOTE_G4: MidiNote = 55;
    pub const NOTE_Ab4: MidiNote = 56;
    pub const NOTE_A4: MidiNote = 57;
    pub const NOTE_Bb4: MidiNote = 58;
    pub const NOTE_B4: MidiNote = 59;
    pub const NOTE_C5: MidiNote = 60;
    pub const NOTE_Db5: MidiNote = 61;
    pub const NOTE_D5: MidiNote = 62;
    pub const NOTE_Eb5: MidiNote = 63;
    pub const NOTE_E5: MidiNote = 64;
    pub const NOTE_F5: MidiNote = 65;
    pub const NOTE_Gb5: MidiNote = 66;
    pub const NOTE_G5: MidiNote = 67;
    pub const NOTE_Ab5: MidiNote = 68;
    pub const NOTE_A5: MidiNote = 69;
    pub const NOTE_Bb5: MidiNote = 70;
    pub const NOTE_B5: MidiNote = 71;
    pub const NOTE_C6: MidiNote = 72;
    pub const NOTE_Db6: MidiNote = 73;
    pub const NOTE_D6: MidiNote = 74;
    pub const NOTE_Eb6: MidiNote = 75;
    pub const NOTE_E6: MidiNote = 76;
}

fn is_note_on(midi_status: u8) -> bool {
    return (midi_status & 0xF0) == 0x90;
}

fn is_note_off(midi_status: u8) -> bool {
    return (midi_status & 0xF0) == 0x80;
}

/// Trait to share code between control surfaces that process MIDI events.
///
/// In most cases a MIDI control surface only needs to implement map_midi_note().
pub trait MidiControlSurface {
    fn map_midi_message(self: &Self, status: u8, data1: u8) -> (StateChange, Operation) {
        if is_note_on(status) {
            return (StateChange::On, self.map_midi_note(data1));
        } else if is_note_off(status) {
            return (StateChange::Off, self.map_midi_note(data1));
        } else {
            return (StateChange::NoChange, Operation::NoOp);
        }
    }

    fn map_midi_note(self: &Self, _note: MidiNote) -> Operation {
        return Operation::NoOp;
    }
}
