// Control Boucle using a piano keyboard.

use crate::ops::*;
use crate::op_sequence;
use crate::op_sequence::OpSequence;

use log::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};

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

pub enum Operation {
    NoOp,
    Reverse,
    Repeat { loop_size: f32 },
    Jump { offset: f32 },
}

struct RecordedMidiEvent {
    timestamp: Instant,
    status: u8,
    note: MidiNote,
}

pub struct PianoControl {
    event_buffer: Vec<RecordedMidiEvent>,

    beats_to_samples: f32,

    active_reverse: Option<op_sequence::Entry>,
    active_jumps: HashMap<MidiNote, op_sequence::Entry>,
    active_repeats: HashMap<MidiNote, op_sequence::Entry>,
}

// Keyboard map for OP-1.
mod op1 {
    use super::Operation;
    use super::note;
    use super::MidiNote;

    pub fn note_to_op(note: MidiNote) -> Operation {
        match note {
            note::NOTE_F4 => Operation::Jump { offset: -8.0 },
            note::NOTE_Gb4 => Operation::Jump { offset: -4.0 },
            note::NOTE_G4 => Operation::Jump { offset: -2.0 },
            note::NOTE_Ab4 => Operation::Jump { offset: -1.0 },
            note::NOTE_A4 => Operation::Jump { offset: -0.5 },
            note::NOTE_Bb4 => Operation::NoOp,
            note::NOTE_B4 => Operation::Jump { offset: -0.25 },

            note::NOTE_C5 => Operation::Repeat { loop_size: 0.0625 },
            note::NOTE_Db5 => Operation::Repeat { loop_size: 0.125 },
            note::NOTE_D5 => Operation::Repeat { loop_size: 0.25 },
            note::NOTE_Eb5 => Operation::Repeat { loop_size: 0.5 },

            note::NOTE_E5 => Operation::Reverse,
            note::NOTE_F5 => Operation::NoOp,

            note::NOTE_Gb5 => Operation::Repeat { loop_size: 1.0 },
            note::NOTE_G5 => Operation::Repeat { loop_size: 2.0 },
            note::NOTE_Ab5 => Operation::Repeat { loop_size: 4.0 },
            note::NOTE_A5 => Operation::Repeat { loop_size: 8.0 },

            note::NOTE_Bb5 => Operation::NoOp,

            note::NOTE_B5 => Operation::Jump { offset: 0.25 },
            note::NOTE_C6 => Operation::Jump { offset: 0.5 },
            note::NOTE_Db6 => Operation::Jump { offset: 1.0 },
            note::NOTE_D6 => Operation::Jump { offset: 2.0 },
            note::NOTE_Eb6 => Operation::Jump { offset: 4.0 },
            note::NOTE_E6 => Operation::Jump { offset: 8.0 },

            _ => Operation::NoOp,
        }
    }
}

// Keyboard map for Organelle.
mod organelle {
    use super::Operation;
    use super::note;
    use super::MidiNote;

    pub fn note_to_op(note: MidiNote) -> Operation {
        match note {
            note::NOTE_B5 => Operation::Reverse,

            _ => Operation::NoOp,
        }
    }
}

impl PianoControl {
    pub fn new(beats_to_samples: f32) -> Self {
        PianoControl {
            event_buffer: Vec::new(),
            beats_to_samples,
            active_reverse: None,
            active_jumps: HashMap::new(),
            active_repeats: HashMap::new(),
        }
    }

    // Record MIDI events as they are received.
    pub fn record_midi_event(self: &mut PianoControl,
                             timestamp: std::time::Instant,
                             midi_event_status: u8,
                             midi_event_note: u8) {
        if midi_event_note < note::NOTE_C4 || midi_event_note > note::NOTE_E6 {
            return;
        }

        if !is_note_on(midi_event_status) && !is_note_off(midi_event_status) {
            return;
        }

        info!("recorded event {} {} at clock {:?}", midi_event_status, midi_event_note, timestamp);
        self.event_buffer.push(RecordedMidiEvent {
            timestamp,
            status: midi_event_status,
            note: midi_event_note,
        });
    }

    // Turn recorded MIDI events into Boucle operations, for a given time period.
    pub fn ops_for_period(self: &mut PianoControl,
                          loop_start_time: Instant,
                          period_start: Instant,
                          period_duration: Duration) -> OpSequence {
        let mut op_sequence: OpSequence = OpSequence::new();

        debug!("ops_for_period: {:?} for {:?} (buffer length: {}", period_start, period_duration, self.event_buffer.len());
        let mut i = 0;
        while i < self.event_buffer.len() {
            let event = &self.event_buffer[i];

            // FIXME: need to process events from before buffer started, so we don't drop
            // events if we miss an audio buffer.
            if event.timestamp >= period_start && event.timestamp < (period_start + period_duration) {
                let op: Operation = op1::note_to_op(event.note);
                info!("Matched at {:#?}", event.timestamp);
                match op {
                    Operation::Reverse => {
                        if is_note_on(event.status) && self.active_reverse.is_none() {
                            info!("{:#?}: reverse on", event.timestamp);
                            self.active_reverse = Some(op_sequence::Entry {
                                start: event.timestamp,
                                duration: None,
                                op: Box::new(ReverseOp {})
                            });
                        } else if is_note_off(event.status) && matches!(self.active_reverse, Some(_)) {
                            info!("{:#?}: reverse off", event.timestamp);
                            let mut op_entry: op_sequence::Entry = self.active_reverse.take().unwrap();
                            op_entry.duration = Some(event.timestamp.duration_since(op_entry.start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched note on/off for {:?}", event.note);
                        }
                    },

                    Operation::Repeat { loop_size } => {
                        if is_note_on(event.status) && !self.active_repeats.contains_key(&event.note) {
                            info!("{:#?}: repeat({}) on", event.timestamp, loop_size);
                            self.active_repeats.insert(event.note, op_sequence::Entry {
                                start: event.timestamp,
                                duration: None,
                                op: Box::new(RepeatOp {
                                    loop_size: (loop_size * self.beats_to_samples).floor() as usize
                                })
                            });
                        } else if is_note_off(event.status) && self.active_repeats.contains_key(&event.note) {
                            info!("{:#?}: repeat({}) on", event.timestamp, loop_size);
                            let mut op_entry: op_sequence::Entry = self.active_repeats.remove(&event.note).unwrap();
                            op_entry.duration = Some(event.timestamp.duration_since(op_entry.start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched note on/off for {:?}", event.note);
                        }
                    },

                    Operation::Jump { offset } => {
                        if is_note_on(event.status) && !self.active_jumps.contains_key(&event.note) {
                            info!("{:#?}: jumps({}) on", event.timestamp, offset);
                            self.active_jumps.insert(event.note, op_sequence::Entry {
                                start: event.timestamp,
                                duration: None,
                                op: Box::new(JumpOp {
                                    offset: (offset * self.beats_to_samples).floor() as isize
                                })
                            });
                        } else if is_note_off(event.status) && self.active_jumps.contains_key(&event.note) {
                            info!("{:#?}: repeat({}) on", event.timestamp, offset);
                            let mut op_entry: op_sequence::Entry = self.active_jumps.remove(&event.note).unwrap();
                            op_entry.duration = Some(event.timestamp.duration_since(op_entry.start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched note on/off for {:?}", event.note);
                        }
                    },
                    _ => {}
                }

                self.event_buffer.remove(i);
            } else {
                i += 1;
            }
        };

        // Include all ops which are still active at end, including any that started in the past
        if matches!(self.active_reverse, Some(_)) {
            let op_entry: op_sequence::Entry = self.active_reverse.as_ref().unwrap().clone();
            debug!("{:#?}: reverse on since ", op_entry.start);
            op_sequence.push(op_entry);
        }

        for op_entry in self.active_repeats.values() {
            debug!("{:#?}: repeat on since", op_entry.start);
            op_sequence.push(op_entry.clone());
        }

        for op_entry in self.active_jumps.values() {
            debug!("{:#?}: jumps on since", op_entry.start);
            op_sequence.push(op_entry.clone());
        }
        return op_sequence;
    }
}
