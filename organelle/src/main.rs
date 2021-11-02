mod patch_error;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Arg, App};
use cpal::traits::{DeviceTrait, HostTrait};
use log::*;
use nannou_osc as osc;

use boucle::BeatFraction;
use boucle::Boucle;
use boucle::buffers::LoopBuffers;
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

    pub fn run(self: &mut Self,
               input_device_name: Option<&str>,
               output_device_name: Option<&str>) -> Result<(), PatchError> {
        let audio_host = cpal::default_host();
        info!("cpal host: {:?}, input: {:?}, output: {:?}", audio_host.id(), input_device_name, output_device_name);

        let audio_out = match output_device_name {
            Some(name) => audio_host.output_devices()?
                .find(|d| {
                    info!("matching {} with {:?}", name, d.name());
                    name == d.name().unwrap_or("".to_string())
                })
                .expect(format!("no output device found matching '{}'", name).as_str()),
            None => audio_host.default_output_device()
                .expect("no output device available"),
        };

        let audio_in = match input_device_name {
            Some(name) => audio_host.input_devices()?
                .find(|d| name == d.name().unwrap_or("".to_string()))
                .expect(format!("no input device found matching '{}'", name).as_str()),
            None => audio_host.default_input_device()
                .expect("no input device available"),
        };

        let supported_audio_config = boucle::cpal_helpers::get_audio_config(&self.boucle_rc.lock().unwrap(), &audio_out);

        let sample_format = supported_audio_config.sample_format();
        info!("Sample format: {:?}", sample_format);

        let input_audio_config: cpal::StreamConfig = supported_audio_config.clone().into();
        let _audio_in_stream = match sample_format {
            // FIXME: do we need to support all these? Organelle should give us same each time.
            cpal::SampleFormat::F32 => boucle::cpal_helpers::open_in_stream::<f32>(audio_in, input_audio_config, self.buffers_rc.clone()),
            cpal::SampleFormat::I16 => boucle::cpal_helpers::open_in_stream::<i16>(audio_in, input_audio_config, self.buffers_rc.clone()),
            cpal::SampleFormat::U16 => boucle::cpal_helpers::open_in_stream::<u16>(audio_in, input_audio_config, self.buffers_rc.clone()),
        };

        let output_audio_config: cpal::StreamConfig = supported_audio_config.into();
        let _audio_out_stream = match sample_format {
            cpal::SampleFormat::F32 => boucle::cpal_helpers::open_out_stream::<f32>(audio_out, output_audio_config, self.boucle_rc.clone(), self.buffers_rc.clone()),
            cpal::SampleFormat::I16 => boucle::cpal_helpers::open_out_stream::<i16>(audio_out, output_audio_config, self.boucle_rc.clone(), self.buffers_rc.clone()),
            cpal::SampleFormat::U16 => boucle::cpal_helpers::open_out_stream::<u16>(audio_out, output_audio_config, self.boucle_rc.clone(), self.buffers_rc.clone()),
        };

        self.signal_loaded();
        self.update_screen();
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

    let app_m = App::new("Boucle looper for Organelle")
        .version("1.0")
        .arg(Arg::with_name("input-device")
             .long("input-device")
             .short("i")
             .help("Record audio from device")
             .takes_value(true)
             .value_name("NAME"))
        .arg(Arg::with_name("output-device")
             .long("output-device")
             .short("o")
             .help("Play audio to device")
             .takes_value(true)
             .value_name("NAME"))
        .get_matches();

    let audio_in = app_m.value_of("input-device");
    let audio_out = app_m.value_of("output-device");

    let mut patch = Patch::new()
        .map_err(|e| panic!("{}", e.message))
        .unwrap();
    patch.run(audio_in, audio_out)
        .map_err(|e| panic!("{}", e.message))
        .unwrap();
}
