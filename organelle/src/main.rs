mod patch_error;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use cpal::traits::{DeviceTrait, HostTrait};
use log::*;
use nannou_osc as osc;

use boucle::Boucle;
use boucle::buffers::LoopBuffers;
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

impl Patch {
    pub fn new() -> Result<Self, PatchError> {
        let boucle_config = boucle::Config {
            sample_rate: SAMPLE_RATE,
            beats_to_samples: (60.0 / DEFAULT_BPM) * (SAMPLE_RATE as f32)
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

    pub fn run(self: &Self) -> Result<(), PatchError> {
        let audio_host = cpal::default_host();

        let audio_in = audio_host.default_input_device()
            .ok_or(PatchError { message: "Unable to open default input device".to_string() })?;
        let audio_out = audio_host.default_output_device()
            .ok_or(PatchError { message: "Unable to open default input device".to_string() })?;
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

        loop {
            self.process_events();

            thread::sleep(time::Duration::from_millis(10));
        }
    }

    pub fn handle_osc(self: &Self, message: osc::Message) {
        match message.addr.as_str() {
            "/keys" => {
                info!("Keys!");
            },
            "/knobs" => {
                info!("Knobs!");
            },
            _ => {
                warn!("Unhandled OSC message: {:?}", message);
            },
        }
    }

    pub fn process_events(self: &Self) {
        // Receive OSC events
        for (packet, addr) in self.receiver.try_iter() {
            debug!("Received OSC: {}: {:?}", addr, packet);
            match packet {
                osc::Packet::Message(message) => self.handle_osc(message),
                osc::Packet::Bundle(_) => warn!("Unhandled OSC bundle")
            }
        }

        let osc_addr = "/oled/line/1".to_string();
        let args = vec![osc::Type::String("Hello from Boucle looper".to_string())];
        let packet = (osc_addr, args);
        self.sender.send(packet).ok();
    }
}

pub fn main() {
    env_logger::init();

    let patch = Patch::new()
        .map_err(|e| panic!("{}", e.message))
        .unwrap();
    patch.run();
}
