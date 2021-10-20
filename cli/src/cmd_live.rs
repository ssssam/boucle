use std::thread::sleep;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait};
use log::*;
use portmidi::{PortMidi};

use boucle;
use boucle::cpal_helpers;
use boucle::Boucle;

use crate::app_config::AppConfig;
use crate::buffers::{InputBuffer, LoopBuffers, create_buffers, input_wav_to_buffer};
use crate::app_error::AppError;

fn open_in_stream<T: cpal::Sample>(device: cpal::Device,
                                   config: cpal::StreamConfig,
                                   buffers_rc: Arc<Mutex<LoopBuffers>>) -> Box<cpal::Stream> {
    return Box::new(device.build_input_stream(
        &config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut buffers = buffers_rc.lock().unwrap();

            let buffer_length = buffers.input_a.len();

            let mut record_pos = buffers.record_pos.clone();

            {
                let mut record_buffer: &mut boucle::Buffer = match buffers.current_input {
                    InputBuffer::A => &mut buffers.input_a,
                    InputBuffer::B => &mut buffers.input_b,
                };

                for &s in data {
                    record_buffer[record_pos] = cpal::Sample::from(&s);
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
        },
        move |err| { warn!("{}", err) }
    ).unwrap());
}

fn open_out_stream<T: cpal::Sample>(device: cpal::Device,
                                    config: cpal::StreamConfig,
                                    boucle_rc: Arc<Mutex<Boucle>>,
                                    buffers_rc: Arc<Mutex<LoopBuffers>>) -> Box<cpal::Stream> {
    return Box::new(device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut boucle = boucle_rc.lock().unwrap();
            let mut buffers = buffers_rc.lock().unwrap();

            let buffer_length = buffers.input_a.len();

            let mut play_clock = buffers.play_clock.clone();
            {
                let mut in_buffer = match buffers.current_output {
                    InputBuffer::A => &buffers.input_a,
                    InputBuffer::B => &buffers.input_b,
                };

                let mut out_pos = 0;
                let play_pos = play_clock % buffer_length;
                let span = std::cmp::min(buffer_length - play_pos, data.len());
                debug!("Play clock {} pos {}/{} span {} (total data {})", play_clock, play_pos, buffer_length, span, data.len());

                let ops = boucle.controller.ops_for_period(play_clock, span);
                boucle.process_buffer(&in_buffer, play_clock, span,
                                      &ops, &mut |s| {
                    data[out_pos] = cpal::Sample::from(&s);
                    out_pos += 1;
                });
                play_clock += span;

                if out_pos < data.len() {
                    // Flip buffer and continue
                    let span_2 = data.len() - span;
                    debug!("play buffer flip");

                    if buffers.current_output == InputBuffer::A {
                        buffers.current_output = InputBuffer::B;
                        in_buffer = &mut buffers.input_b;
                    } else {
                        buffers.current_output = InputBuffer::A;
                        in_buffer = &mut buffers.input_a;
                    }

                    let ops = boucle.controller.ops_for_period(play_clock, span_2);
                    boucle.process_buffer(&in_buffer, play_clock, span_2,
                                          &ops, &mut |s| {
                        data[out_pos] = cpal::Sample::from(&s);
                        out_pos += 1;
                    });
                    play_clock += span_2;
                }
            }

            buffers.play_clock = play_clock;

            // Performer responds to what they hear.
            // The MIDI events we receive are therefore treated as relative
            // to the last thing the performer heard.
            boucle.controller.set_event_sync_point(Instant::now(), play_clock);
        },
        move |err| { warn!("{}", err) }
    ).unwrap());
}

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
        beats_to_samples: (60.0 / bpm) * (app_config.sample_rate as f32)
    };

    let boucle: boucle::Boucle = boucle::Boucle::new(&config);
    let boucle_rc: Arc<Mutex<Boucle>> = Arc::new(Mutex::new(boucle));

    let buffer_size_samples: usize = (loop_time_seconds * app_config.sample_rate as f32).floor() as usize;
    let buffers = create_buffers(buffer_size_samples);
    let buf_rc: Arc<Mutex<LoopBuffers>> = Arc::new(Mutex::new(buffers));

    let audio_in_device;
    let _audio_in_stream;

    let audio_out_device = match output_device_name {
        Some(name) => audio_host.output_devices()?.find(|d| name == d.name().unwrap_or("".to_string()))
            .expect("no output device found matching name"),
        None => audio_host.default_output_device()
            .expect("no output device available"),
    };

    let supported_audio_config = cpal_helpers::get_audio_config(&config, &audio_out_device);
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
            cpal::SampleFormat::F32 => open_in_stream::<f32>(audio_in_device, input_audio_config, buf_rc.clone()),
            cpal::SampleFormat::I16 => open_in_stream::<i16>(audio_in_device, input_audio_config, buf_rc.clone()),
            cpal::SampleFormat::U16 => open_in_stream::<u16>(audio_in_device, input_audio_config, buf_rc.clone()),
        };
    };


    let _audio_out_stream = match sample_format {
        cpal::SampleFormat::F32 => open_out_stream::<f32>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
        cpal::SampleFormat::I16 => open_out_stream::<i16>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
        cpal::SampleFormat::U16 => open_out_stream::<u16>(audio_out_device, output_audio_config, boucle_rc.clone(), buf_rc.clone()),
    };

    while let Ok(_) = midi_in.poll() {
        if let Ok(Some(event)) = midi_in.read_n(1024) {
            let event2: &portmidi::MidiEvent = event.get(0).unwrap();

            let mut boucle = boucle_rc.lock().unwrap();
            boucle.controller.record_midi_event(Instant::now(), event2.message.status, event2.message.data1);
        }

        // there is no blocking receive method in PortMidi
        sleep(Duration::from_millis(10));
    }

    return Ok(())
}
