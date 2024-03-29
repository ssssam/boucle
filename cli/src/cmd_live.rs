use std::thread::sleep;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait};
use portmidi::{PortMidi};

use boucle;
use boucle::buffers::{LoopBuffers, create_buffers};
use boucle::cpal_helpers;
use boucle::control_surface::midi::MidiControlSurface;
use boucle::Boucle;

use crate::app_config::AppConfig;
use crate::app_error::AppError;
use crate::wav::input_wav_to_buffer;

fn open_midi_in<'a>(midi_context: &'a portmidi::PortMidi, midi_in_port: i32) -> Result<Box<portmidi::InputPort<'a>>, portmidi::Error> {
    let midi_info = midi_context.device(midi_in_port)?;
    match midi_context.input_port(midi_info, 1024) {
        Ok(port) => return Ok(Box::new(port)),
        Err(error) => return Err(error),
    };
}

pub fn run_live(app_config: &AppConfig, midi_in_port: i32, audio_in_path: Option<&str>, input_device_name: Option<&str>,
                output_device_name: Option<&str>, loop_time_seconds: f32, bpm: f32) -> Result<(), AppError> {
    let midi_context = match PortMidi::new() {
        Ok(value) => value,
        Err(error) => return Err(AppError { message: format!("Cannot open PortMIDI: {}", error) }),
    };
    let midi_in = match open_midi_in(&midi_context, midi_in_port) {
        Ok(value) => value,
        Err(error) => return Err(AppError { message: format!("Cannot open MIDI input: {}", error) }),
    };

    let audio_host = cpal::default_host();

    let config = boucle::Config {
        sample_rate: app_config.sample_rate,
        beat_fraction_to_samples: (60.0 / bpm / 16.0) * (app_config.sample_rate as f32)
    };

    let buffer_size_samples: usize = (loop_time_seconds * app_config.sample_rate as f32).floor() as usize;
    let buffers = create_buffers(buffer_size_samples);
    let buf_rc: Arc<Mutex<LoopBuffers>> = Arc::new(Mutex::new(buffers));

    let boucle: boucle::Boucle = boucle::Boucle::new(&config, buffer_size_samples);
    let boucle_rc: Arc<Mutex<Boucle>> = Arc::new(Mutex::new(boucle));

    let audio_in_device;
    let _audio_in_stream;

    let audio_out_device = match output_device_name {
        Some(name) => audio_host.output_devices()?.find(|d| name == d.name().unwrap_or("".to_string()))
            .expect("no output device found matching name"),
        None => audio_host.default_output_device()
            .expect("no output device available"),
    };

    let supported_audio_config = cpal_helpers::get_audio_config(&boucle_rc.lock().unwrap(), &audio_out_device);
    let sample_format = supported_audio_config.sample_format();
    let input_audio_config: cpal::StreamConfig = supported_audio_config.clone().into();
    let output_audio_config: cpal::StreamConfig = supported_audio_config.into();

    if let Some(filename) = audio_in_path {
        input_wav_to_buffer(filename, &mut buf_rc.lock().unwrap())
            .expect("Failed to read input");
    } else {
        audio_in_device = match input_device_name {
            Some(name) => audio_host.input_devices()?
                .find(|d| name == d.name().unwrap_or("".to_string()))
                .expect("no input device found matching name"),
            None => audio_host.default_input_device()
                .expect("no input device available"),
        };

        let mut buffers = buf_rc.lock().unwrap();
        // We start playing wet A while recording B, so set A to silence.
        for i in 0..buffers.input_a.len() {
            buffers.input_a[i] = 0.0;
        }

        _audio_in_stream = match sample_format {
            cpal::SampleFormat::F32 => cpal_helpers::open_in_stream::<f32>(audio_in_device, input_audio_config, buf_rc.clone()),
            cpal::SampleFormat::I16 => cpal_helpers::open_in_stream::<i16>(audio_in_device, input_audio_config, buf_rc.clone()),
            cpal::SampleFormat::U16 => cpal_helpers::open_in_stream::<u16>(audio_in_device, input_audio_config, buf_rc.clone()),
        };
    };


    let _audio_out_stream = match sample_format {
        cpal::SampleFormat::F32 => cpal_helpers::open_out_stream::<f32>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
        cpal::SampleFormat::I16 => cpal_helpers::open_out_stream::<i16>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
        cpal::SampleFormat::U16 => cpal_helpers::open_out_stream::<u16>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
    };

    let interface = boucle::control_surface::midi::op1::Op1 {};

    while let Ok(_) = midi_in.poll() {
        if let Ok(Some(event)) = midi_in.read_n(1024) {
            let event2: &portmidi::MidiEvent = event.get(0).unwrap();

            let mut boucle = boucle_rc.lock().unwrap();
            let (state_change, operation) = interface.map_midi_message(
                event2.message.status,
                event2.message.data1,
            );

            boucle.event_recorder.record_event(Instant::now(), state_change, operation);
        }

        // there is no blocking receive method in PortMidi
        sleep(Duration::from_millis(10));
    }

    return Ok(())
}
