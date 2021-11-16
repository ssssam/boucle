mod patch_error;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use log::*;
use nannou_osc as osc;

use boucle::BeatFraction;
use boucle::Boucle;
use boucle::buffers::{Buffer, InputBuffer, LoopBuffers};
use boucle::event::StateChange;
use boucle::Operation;
use crate::patch_error::PatchError;

// These ports are defined in
// https://github.com/critterandguitari/Organelle_OS/blob/master/main.cpp
const RECEIVE_PORT: u16 = 4000;
const SEND_PORT: u16 = 4001;

// See https://www.critterandguitari.com/organelle
// and https://forum.critterandguitari.com/t/change-sample-rate-on-organelle/3840/7
const SAMPLE_RATE: u32 = 44100;

// Range of 256 valid BPM settings, maps nicely to knob 0-1023.
const MIN_BPM: f32 = 30.0;
const MAX_BPM: f32 = 286.0;
const DEFAULT_BPM: f32 = 60.0;

const MIN_LOOP_BEATS: f32 = 1.0;
const MAX_LOOP_BEATS: f32 = 32.0;
const DEFAULT_LOOP_BEATS: f32 = 8.0;

struct Patch {
    boucle_rc: Arc<Mutex<Boucle>>,
    buffers_rc: Arc<Mutex<LoopBuffers>>,

    receiver: osc::Receiver,
    sender: osc::Sender::<osc::Connected>,

    // FIXME: hardcoded for now - should be changable via the knobs.
    bpm: f32,
    loop_beats: f32,
}

type UpdateScreenFlag = bool;

fn map_key(key: i32) -> Operation {
    match key {
        0  /* Aux */ => Operation::NoOp,
        1  /* C4 */  => Operation::Jump { offset: BeatFraction::from(-8.0) },
        2            => Operation::Jump { offset: BeatFraction::from(-4.0) },
        3            => Operation::Jump { offset: BeatFraction::from(-2.0) },
        4            => Operation::Jump { offset: BeatFraction::from(-1.0) },
        5  /* E4 */  => Operation::Jump { offset: BeatFraction::from(-0.5) },
        6  /* F4 */  => Operation::Jump { offset: BeatFraction::from(-0.25) },
        7            => Operation::NoOp,
        8            => Operation::Repeat { loop_size: BeatFraction::from(0.0625) },
        9  /* G#4 */ => Operation::Repeat { loop_size: BeatFraction::from(0.125) },
        10           => Operation::Repeat { loop_size: BeatFraction::from(0.25) },
        11 /* Bb4 */ => Operation::Repeat { loop_size: BeatFraction::from(0.5) },
        12 /* B4 */  => Operation::Reverse,
        13 /* C5 */  => Operation::NoOp,
        14           => Operation::Repeat { loop_size: BeatFraction::from(1.0) },
        15 /* D5 */  => Operation::Repeat { loop_size: BeatFraction::from(2.0) },
        16           => Operation::Repeat { loop_size: BeatFraction::from(4.0) },
        17 /* E5 */  => Operation::Repeat { loop_size: BeatFraction::from(8.0) },
        18 /* F5 */  => Operation::Jump { offset: BeatFraction::from(0.25) },
        19 /* Gb5 */ => Operation::NoOp,
        20           => Operation::Jump { offset: BeatFraction::from(0.5) },
        21           => Operation::Jump { offset: BeatFraction::from(1.0) },
        22           => Operation::Jump { offset: BeatFraction::from(2.0) },
        23           => Operation::Jump { offset: BeatFraction::from(4.0) },
        24           => Operation::Jump { offset: BeatFraction::from(8.0) },
        _ => {
            warn!("Unmapped key: {}", key);
            Operation::NoOp
        },
    }
}


struct JackNotifications;

impl jack::NotificationHandler for JackNotifications {
    fn thread_init(&self, _: &jack::Client) {
        info!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        info!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        info!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        // FIXME: We need to handle this.
        info!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        info!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        info!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        info!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        info!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        info!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        info!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

impl Patch {
    pub fn new() -> Result<Self, PatchError> {
        let boucle_config = boucle::Config {
            sample_rate: SAMPLE_RATE,
            beat_fraction_to_samples: (60.0 / DEFAULT_BPM / 16.0) * (SAMPLE_RATE as f32)
        };

        let boucle: boucle::Boucle = boucle::Boucle::new(&boucle_config);

        let max_buffer_time = ((60.0 / MIN_BPM) * MAX_LOOP_BEATS).ceil() as usize;
        info!("Maximium buffer time: {} seconds", max_buffer_time);
        let buffers = boucle::buffers::create_buffers(max_buffer_time * SAMPLE_RATE as usize);

        let receiver = osc::receiver(RECEIVE_PORT)?;
        let send_addr = format!("{}:{}", "127.0.0.1", SEND_PORT);
        let sender = osc::sender()?
            .connect(send_addr)?;

        return Ok(Patch {
            boucle_rc: Arc::new(Mutex::new(boucle)),
            buffers_rc: Arc::new(Mutex::new(buffers)),
            receiver,
            sender,
            bpm: DEFAULT_BPM,
            loop_beats: DEFAULT_LOOP_BEATS,
        });
    }

    pub fn run(self: &mut Self) -> Result<(), PatchError> {
        let (client, _status) =
            jack::Client::new("boucle", jack::ClientOptions::NO_START_SERVER).unwrap();

        let in_port = client
            .register_port("boucle_in", jack::AudioIn::default())
            .unwrap();
        let mut out_port = client
            .register_port("boucle_out", jack::AudioOut::default())
            .unwrap();

        self.signal_loaded();
        self.update_screen();

        let boucle_rc = self.boucle_rc.clone();
        let buffers_rc = self.buffers_rc.clone();
        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let mut buffers = buffers_rc.lock().unwrap();

            let buffer_length = buffers.input_a.len();

            let mut record_pos = buffers.record_pos.clone();

            // Read input into buffer
            {
                let mut record_buffer: &mut Buffer = match buffers.current_input {
                    InputBuffer::A => &mut buffers.input_a,
                    InputBuffer::B => &mut buffers.input_b,
                };

                for &s in in_port.as_slice(ps) {
                    record_buffer[record_pos] = s;
                    record_pos += 1;
                    if record_pos >= buffer_length {
                        if buffers.current_input == InputBuffer::A {
                            buffers.current_input = InputBuffer::B;
                            record_buffer = &mut buffers.input_b;
                        } else {
                            buffers.current_input = InputBuffer::A;
                            record_buffer = &mut buffers.input_a;
                        }
                        debug!("Record buffer flip");
                        record_pos = 0;
                    }
                }
            }

            buffers.record_pos = record_pos;

            let mut boucle = boucle_rc.lock().unwrap();
            let mut play_clock = buffers.play_clock.clone();

            let out_buf = out_port.as_mut_slice(ps);
            {
                let mut in_buffer = match buffers.current_output {
                    InputBuffer::A => &buffers.input_a,
                    InputBuffer::B => &buffers.input_b,
                };

                let mut out_pos = 0;
                let play_pos = play_clock % buffer_length;
                let span = std::cmp::min(buffer_length - play_pos, out_buf.len());
                debug!("Play clock {} pos {}/{} span {} (total data {})", play_clock, play_pos, buffer_length, span, out_buf.len());

                let ops = boucle.event_recorder.ops_for_period(play_clock, span);
                boucle.process_buffer(&in_buffer, play_clock, span,
                                      &ops, &mut |s| {
                    out_buf[out_pos] = s;
                    out_pos += 1;
                });
                play_clock += span;

                if out_pos < out_buf.len() {
                    // Flip buffer and continue
                    let span_2 = out_buf.len() - span;
                    debug!("play buffer flip");

                    if buffers.current_output == InputBuffer::A {
                        buffers.current_output = InputBuffer::B;
                        in_buffer = &mut buffers.input_b;
                    } else {
                        buffers.current_output = InputBuffer::A;
                        in_buffer = &mut buffers.input_a;
                    }

                    let ops = boucle.event_recorder.ops_for_period(play_clock, span_2);
                    boucle.process_buffer(&in_buffer, play_clock, span_2,
                                          &ops, &mut |s| {
                        out_buf[out_pos] = s;
                        out_pos += 1;
                    });
                    play_clock += span_2;
                }
            }

            buffers.play_clock = play_clock;

            // Performer responds to what they hear.
            // The MIDI events we receive are therefore treated as relative
            // to the last thing the performer heard.
            boucle.event_recorder.set_event_sync_point(Instant::now(), play_clock);
            jack::Control::Continue
        };

        let active_client = client.activate_async(
            JackNotifications,
            jack::ClosureProcessHandler::new(process_callback),
        );

        loop {
            let update_screen = self.process_events();

            if update_screen {
                self.update_screen();
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn handle_key(self: &mut Self, key: i32, pressed: bool) -> UpdateScreenFlag {
        info!("Key {} {}", key, pressed);
        let mut boucle = self.boucle_rc.lock().unwrap();
        let operation = map_key(key);
        let state_change = match pressed {
            false => StateChange::Off,
            true => StateChange::On,
        };
        boucle.event_recorder.record_event(Instant::now(), state_change, operation);
        return false;
    }

    fn handle_knobs(self: &mut Self, positions: [i32; 6]) -> UpdateScreenFlag {
        info!("Knobs {} {} {} {} {} {}", positions[0], positions[1], positions[2],
              positions[3], positions[4], positions[5]);

        let mut update_screen = false;

        fn scale_from_1024(min: f32, max: f32, value: i32) -> f32 {
            let total = max - min;
            let steps = 1024.0 / total;
            return (value as f32 / 1024.0) * steps + min;
        }

        let new_bpm = scale_from_1024(MIN_BPM, MAX_BPM, positions[0]);
        if new_bpm != self.bpm {
            self.bpm = new_bpm;
            update_screen = true;
        }

        let new_loop_beats = scale_from_1024(MIN_LOOP_BEATS, MAX_LOOP_BEATS, positions[0]);
        if new_loop_beats != self.loop_beats {
            self.loop_beats = new_loop_beats;
            update_screen = true;
        }

        return update_screen;
    }


    fn handle_osc(self: &mut Self, message: &osc::Message) -> UpdateScreenFlag {
        fn args(message: &osc::Message) -> &[osc::Type] {
            match &message.args {
                Some(args) => args.as_slice(),
                None => &[],
            }
        }

        match message.addr.as_str() {
            "/keys" => {
                if let [osc::Type::Int(key), osc::Type::Int(pressed)] = args(message) {
                    if *key >= 0 && *key <= 24 {
                        return self.handle_key(*key, *pressed >= 1);
                    }
                }
            },
            "/knobs" => {
                if let [osc::Type::Int(k1), osc::Type::Int(k2), osc::Type::Int(k3),
                        osc::Type::Int(k4), osc::Type::Int(k5),osc::Type::Int(k6)] = args(message) {
                    if *k1 >= 0 && *k1 < 1024 &&
                       *k2 >= 0 && *k2 < 1024 &&
                       *k3 >= 0 && *k3 < 1024 &&
                       *k4 >= 0 && *k4 < 1024 &&
                       *k5 >= 0 && *k5 < 1024 &&
                       *k6 >= 0 && *k6 < 1024 {
                        return self.handle_knobs([*k1, *k2, *k3, *k4, *k5, *k6]);
                    }
                }
            },
            _ => {},
        }
        warn!("Unhandled OSC message: {:?}", message);
        return false;
    }

    fn signal_loaded(self: &Self) {
        let packet = ("/patchLoaded".to_string(), vec![]);
        self.sender.send(packet).ok();
    }

    fn update_screen(self: &Self) {
        let header = format!("BPM: {}  Loop: {}", self.bpm, self.loop_beats);

        let addr = "/oled/line/1".to_string();
        let args = vec![osc::Type::String(header.to_string())];

        let packet = (addr, args);
        self.sender.send(packet).ok();
    }

    fn process_events(self: &mut Self) -> UpdateScreenFlag {
        // Receive OSC events
        let mut received: Vec<osc::Message> = Vec::new();
        for (packet, addr) in self.receiver.try_iter() {
            debug!("Received OSC: {}: {:?}", addr, packet);
            match packet {
                osc::Packet::Message(message) => received.push(message),
                osc::Packet::Bundle(_) => warn!("Unhandled OSC bundle"),
            }
        }

        // Handle OSC events
        let mut update_screen: UpdateScreenFlag = false;
        for message in received {
            update_screen |= self.handle_osc(&message);
        }

        return update_screen;
    }
}

pub fn main() {
    env_logger::init();

    let mut patch = Patch::new()
        .map_err(|e| panic!("{}", e.message))
        .unwrap();
    patch.run()
        .map_err(|e| panic!("{}", e.message))
        .unwrap();
}
