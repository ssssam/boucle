//! Record a sequence of operation control messages.
//!
//! Incoming control messages are recorded when received, and turned
//! into a sequence of operations every time we render an audio buffer.

use crate::BeatFraction;
use crate::SamplePosition;
use crate::event::StateChange;
use crate::ops::Operation;
use crate::op_sequence;
use crate::op_sequence::OpSequence;

use log::*;

use std::cmp::max;
use std::collections::HashMap;
use std::time::{Instant};

struct RecordedEvent {
    time: SamplePosition,
    state_change: StateChange,
    operation: Operation,
}

pub struct EventRecorder {
    event_buffer: Vec<RecordedEvent>,

    sample_rate: u32,

    active_reverse: Option<op_sequence::Entry>,
    active_jumps: HashMap<BeatFraction, op_sequence::Entry>,
    active_repeats: HashMap<BeatFraction, op_sequence::Entry>,

    event_sync_time: Instant,
    event_sync_sample_position: SamplePosition,
}

impl EventRecorder {
    pub fn new(sample_rate: u32) -> Self {
        EventRecorder {
            event_buffer: Vec::new(),
            sample_rate,
            active_reverse: None,
            active_jumps: HashMap::new(),
            active_repeats: HashMap::new(),
            event_sync_time: Instant::now(),
            event_sync_sample_position: 0,
        }
    }

    pub fn set_event_sync_point(self: &mut Self, time: Instant, sample_position: SamplePosition) {
        info!("Update sync time: {:?}, sample position: {:?}", time, sample_position);
        self.event_sync_time = time;
        self.event_sync_sample_position = sample_position;
    }

    fn time_to_sample_position(self: &Self,
                               time: Instant) -> SamplePosition {
        let duration = time - self.event_sync_time;
        let duration_samples = duration.as_nanos() * (self.sample_rate as u128) / 1000000000;
        return self.event_sync_sample_position + duration_samples as usize;
    }

    // Record control events as they are received.
    pub fn record_event(self: &mut Self,
                        timestamp: std::time::Instant,
                        state_change: StateChange,
                        operation: Operation) {
        if state_change == StateChange::NoChange {
            return;
        }

        let time = self.time_to_sample_position(timestamp);
        info!("recorded event {:?} {:?} at pos {} clock {:?}", state_change, operation, time, timestamp);
        self.event_buffer.push(RecordedEvent {
            time, state_change, operation
        });
    }

    // Turn recorded events into Boucle operations, for a given time period.
    pub fn ops_for_period(self: &mut Self,
                          period_start: SamplePosition,
                          period_duration: SamplePosition) -> OpSequence {
        let mut op_sequence: OpSequence = OpSequence::new();

        debug!("ops_for_period: {:?} for {:?} (buffer length: {}", period_start, period_duration, self.event_buffer.len());
        let mut i = 0;
        while i < self.event_buffer.len() {
            let event = &self.event_buffer[i];

            let event_sample_position = event.time;

            if event_sample_position < (period_start + period_duration) {
                info!("Matched at {:#?}", event.time);
                match event.operation {
                    Operation::Reverse => {
                        if event.state_change == StateChange::On && self.active_reverse.is_none() {
                            info!("{:#?}: reverse on", event_sample_position);
                            self.active_reverse = Some(op_sequence::Entry {
                                start: event_sample_position,
                                duration: None,
                                operation: event.operation,
                            });
                        } else if event.state_change == StateChange::Off && matches!(self.active_reverse, Some(_)) {
                            let mut op_entry: op_sequence::Entry = self.active_reverse.take().unwrap();
                            info!("{:#?}: reverse off (sample pos {}, op start {})", event.time, event_sample_position, op_entry.start);
                            op_entry.duration = Some(event_sample_position - max(op_entry.start, period_start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched state change for {:?}", event.operation);
                        }
                    },

                    Operation::Repeat { loop_size } => {
                        if event.state_change == StateChange::On && !self.active_repeats.contains_key(&loop_size) {
                            info!("{:#?}: repeat({}) on", event_sample_position, loop_size);
                            self.active_repeats.insert(loop_size, op_sequence::Entry {
                                start: event_sample_position,
                                duration: None,
                                operation: event.operation,
                            });
                        } else if event.state_change == StateChange::Off && self.active_repeats.contains_key(&loop_size) {
                            info!("{:#?}: repeat({}) on", event_sample_position, loop_size);
                            let mut op_entry: op_sequence::Entry = self.active_repeats.remove(&loop_size).unwrap();
                            op_entry.duration = Some(event_sample_position - max(op_entry.start, period_start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched state change for {:?}", event.operation);
                        }
                    },

                    Operation::Jump { offset } => {
                        if event.state_change == StateChange::On && !self.active_jumps.contains_key(&offset) {
                            info!("{:#?}: jumps({}) on", event_sample_position, offset);
                            self.active_jumps.insert(offset, op_sequence::Entry {
                                start: event_sample_position,
                                duration: None,
                                operation: event.operation,
                            });
                        } else if event.state_change == StateChange::Off && self.active_jumps.contains_key(&offset) {
                            info!("{:#?}: jumps({}) on", event_sample_position, offset);
                            let mut op_entry: op_sequence::Entry = self.active_jumps.remove(&offset).unwrap();
                            op_entry.duration = Some(event_sample_position - max(op_entry.start, period_start));
                            op_sequence.push(op_entry);
                        } else {
                            warn!("Warning: mismatched state change for {:?}", event.operation);
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
