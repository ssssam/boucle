//! Helpers to integrate Boucle core with CPAL audio library.
use std::sync::{Arc, Mutex};
use std::time::{Instant};

use cpal::traits::{DeviceTrait};
use log::*;

use crate::Boucle;
use crate::Config;
use crate::buffers::{Buffer, InputBuffer,LoopBuffers};

/// Return a valid cpal configuration for the given Boucle config.
/// Panic if no config is found.
pub fn get_audio_config(lib_config: &Config, device: &cpal::Device) -> cpal::SupportedStreamConfig {
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config")
        .with_sample_rate(cpal::SampleRate(lib_config.sample_rate));
    info!("audio config: {:?}", supported_config);
    return supported_config;
}

/// Open a cpal input stream for 'device', and start recording input into given buffers.
pub fn open_in_stream<T: cpal::Sample>(device: cpal::Device,
                                       config: cpal::StreamConfig,
                                       buffers_rc: Arc<Mutex<LoopBuffers>>) -> Box<cpal::Stream> {
    return Box::new(device.build_input_stream(
        &config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut buffers = buffers_rc.lock().unwrap();

            let buffer_length = buffers.input_a.len();

            let mut record_pos = buffers.record_pos.clone();

            {
                let mut record_buffer: &mut Buffer = match buffers.current_input {
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

/// Open a cpal output stream for 'device', and start processing the given buffers,
/// using the controller assigned to the given Boucle instance.
pub fn open_out_stream<T: cpal::Sample>(device: cpal::Device,
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
