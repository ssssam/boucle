mod tests;

use boucle;
use boucle::Boucle;
use boucle::op_sequence;
use boucle::OpSequence;

use clap::{Arg, App};
use cpal::traits::{DeviceTrait, HostTrait};
use dasp::{Sample};
use hound;
use log::*;
use portmidi::{PortMidi};

use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;use std::thread::sleep;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(PartialEq)]
enum InputBuffer { A, B }

struct LoopBuffers {
    input_a: boucle::Buffer,
    input_b: boucle::Buffer,
    current_input: InputBuffer,
    current_output: InputBuffer,
    record_pos: boucle::SamplePosition,
    play_clock: boucle::SamplePosition,
}

#[derive(Debug)]
struct AppError {
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<portmidi::Error> for AppError {
    fn from(error: portmidi::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<cpal::DevicesError> for AppError {
    fn from(error: cpal::DevicesError) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<cpal::DeviceNameError> for AppError {
    fn from(error: cpal::DeviceNameError) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<hound::Error> for AppError {
    fn from(error: hound::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

fn create_buffers(buffer_size_samples: usize) -> LoopBuffers {
    let this = LoopBuffers {
        input_a: vec!(0.0; buffer_size_samples),
        input_b: vec!(0.0; buffer_size_samples),
        current_input: InputBuffer::B,
        current_output: InputBuffer::A,
        record_pos: 0,
        play_clock: 0,
    };
    return this;
}

fn read_ops(sample_rate: u32, file_name: &str) -> Result<OpSequence, io::Error> {
    let mut text = String::new();
    let mut op_sequence = OpSequence::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let (start_seconds, duration_seconds, op) = boucle::ops::new_from_string(line).expect("Failed to parse line");
        op_sequence.push(op_sequence::Entry {
            start: (start_seconds * sample_rate as f64) as usize,
            duration: Some((duration_seconds * sample_rate as f64) as usize),
            op
        });
    }
    return Ok(op_sequence);
}

fn input_wav_to_buffer(audio_in_path: &str, buffers: &mut LoopBuffers) -> Result<(), AppError> {
    let reader = hound::WavReader::open(audio_in_path)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    info!("Read input {}: {:?}", audio_in_path, spec);
    let wav_buffer: boucle::Buffer = match spec.sample_format {
        hound::SampleFormat::Int => {
            let samples = reader
                .into_samples()
                .filter_map(Result::ok);
            samples.map(|s: i32| s.to_sample::<boucle::Sample>()).collect()
        },
        hound::SampleFormat::Float => {
            let samples = reader
                .into_samples()
                .filter_map(Result::ok);
            samples.map(|s: f32| s.to_sample::<boucle::Sample>()).collect()
        },
    };

    for i in 0..buffers.input_a.len() {
        // Sin wave
        //buffer[i] = f32::sin((i as f32) / 10.0) * 0.2;
        if i < wav_buffer.len() {
            buffers.input_a[i] = wav_buffer[i];
            buffers.input_b[i] = wav_buffer[i];
        } else {
            buffers.input_a[i] = 0.0;
            buffers.input_b[i] = 0.0;
        }
    };

    return Ok(());
}

fn run_batch(audio_in_path: &str, audio_out: &str, loop_time_seconds: f32, operations_file: &str) {
    let op_sequence = read_ops(SAMPLE_RATE, &operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        debug!("{}", op);
    }

    let buffer_size_samples: usize = (loop_time_seconds * SAMPLE_RATE as f32).floor() as usize;
    let mut buffers = create_buffers(buffer_size_samples);

    input_wav_to_buffer(audio_in_path, &mut buffers).expect("Failed to read input");

    let out_spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    let mut writer = hound::WavWriter::create(audio_out, out_spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(&boucle::Config::default());
    boucle.process_buffer(&buffers.input_a, 0, buffers.input_a.len(), &op_sequence, &mut |s| {
        let s_i16 = s.to_sample::<i16>();
        writer.write_sample(s_i16).unwrap();
    });
    writer.finalize().unwrap();
}

const SAMPLE_RATE: u32 = 44100;

fn get_audio_config(device: &cpal::Device) -> cpal::SupportedStreamConfig {
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config")
        .with_sample_rate(cpal::SampleRate(SAMPLE_RATE));
    info!("audio config: {:?}", supported_config);
    return supported_config;
}

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

fn run_live(midi_in_port: i32, audio_in_path: Option<&str>, input_device_name: Option<&str>,
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
        sample_rate: SAMPLE_RATE as u64,
        beats_to_samples: (60.0 / bpm) * (SAMPLE_RATE as f32)
    };

    let boucle: boucle::Boucle = boucle::Boucle::new(&config);
    let boucle_rc: Arc<Mutex<Boucle>> = Arc::new(Mutex::new(boucle));

    let buffer_size_samples: usize = (loop_time_seconds * SAMPLE_RATE as f32).floor() as usize;
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

    let supported_audio_config = get_audio_config(&audio_out_device);
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

fn run_list_ports() -> Result<(), AppError> {
    let host = cpal::default_host();
    println!("Available audio input devices for host {}:", host.id().name());
    for dev in host.input_devices()? {
        println!(" • {}", dev.name()?);
    }

    println!();
    println!("Available audio output devices for host {}:", host.id().name());
    for dev in host.output_devices()? {
        println!(" • {}", dev.name()?);
    }

    println!();
    println!("Available MIDI input ports:");
    let midi_context = PortMidi::new()?;
    for dev in midi_context.devices()? {
        println!(" • {}", dev);
    }

    return Ok(())
}


fn parse_f32_option(string: Option<&str>) -> Option<f32> {
    return match string {
        Some(text) => Some(text.parse::<f32>().unwrap()),
        None => None
    };
}

fn calculate_loop_time(seconds: Option<f32>, beats: Option<f32>, bpm: Option<f32>) -> Result<f32, String> {
    if let Some(value) = seconds {
        return Ok(value);
    } else if let Some(value) = beats {
        if let Some(multiplier) = bpm {
            let loop_seconds: f32 = value * (60.0 / multiplier);
            info!("Loop length: {} * (60.0 / {}) = {}", value, multiplier, loop_seconds);
            return Ok(loop_seconds);
        } else {
            return Err("Loop size in beats requires a BPM".to_string());
        }
    } else {
        return Err("Must specify loop size in either seconds or beats".to_string());
    };
}

fn main() {
    env_logger::init();

    let app_m = App::new("Boucle looper")
        .version("1.0")
        .subcommand(App::new("live")
            .arg(Arg::with_name("input-file")
                 .long("input-file")
                 .short("f")
                 .help("Read loop buffer from FILE")
                 .takes_value(true)
                 .value_name("FILE"))
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
            .arg(Arg::with_name("midi-port")
                 .long("midi-port")
                 .short("p")
                 .help("MIDI port to read from")
                 .takes_value(true)
                 .value_name("PORT"))
            .arg(Arg::with_name("bpm")
                 .long("bpm")
                 .help("Beats per minute")
                 .takes_value(true)
                 .value_name("BPM"))
            .arg(Arg::with_name("loop-time-seconds")
                 .long("loop-time-seconds")
                 .short("s")
                 .help("Loop length, in seconds")
                 .takes_value(true)
                 .value_name("SECONDS"))
            .arg(Arg::with_name("loop-time-beats")
                 .long("loop-time-beats")
                 .short("b")
                 .help("Loop length, in beats (requires `--bpm`)")
                 .takes_value(true)
                 .value_name("BEATS")))
        .subcommand(App::new("batch")
            .arg(Arg::with_name("INPUT")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("OUTPUT")
                 .required(true)
                 .index(2))
            .arg(Arg::with_name("bpm")
                 .long("bpm")
                 .help("Beats per minute")
                 .takes_value(true)
                 .value_name("BPM"))
            .arg(Arg::with_name("loop-time-seconds")
                 .long("loop-time-seconds")
                 .short("s")
                 .help("Loop length, in seconds")
                 .takes_value(true)
                 .value_name("SECONDS"))
            .arg(Arg::with_name("loop-time-beats")
                 .long("loop-time-beats")
                 .short("b")
                 .help("Loop length, in beats (requires `--bpm`)")
                 .takes_value(true)
                 .value_name("BEATS")))
        .subcommand(App::new("list-ports"))
        .get_matches();

    match app_m.subcommand() {
        ("batch", Some(sub_m)) => {
            let loop_time_seconds: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-seconds"));
            let loop_time_beats: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-beats"));
            let bpm: Option<f32> = parse_f32_option(sub_m.value_of("bpm"));
            let loop_time = match calculate_loop_time(loop_time_seconds, loop_time_beats, bpm) {
                Ok(value) => value,
                Err(string) => panic!("{}", string),
            };

            let audio_in = sub_m.value_of("INPUT").unwrap();
            let audio_out = sub_m.value_of("OUTPUT").unwrap();
            let operations_file = "ops.test";
            run_batch(audio_in, audio_out, loop_time, operations_file);
        },
        ("live", Some(sub_m)) => {
            let loop_time_seconds: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-seconds"));
            let loop_time_beats: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-beats"));
            let bpm: Option<f32> = parse_f32_option(sub_m.value_of("bpm"));
            let loop_time = match calculate_loop_time(loop_time_seconds, loop_time_beats, bpm) {
                Ok(value) => value,
                Err(string) => panic!("{}", string),
            };

            let midi_port: i32 = sub_m.value_of("midi-port").unwrap_or("0").
                                    parse::<i32>().unwrap();
            let input_file = sub_m.value_of("input-file");
            let input_device_name = sub_m.value_of("input-device");
            let output_device_name = sub_m.value_of("output-device");
            run_live(midi_port, input_file, input_device_name, output_device_name, loop_time, bpm.unwrap_or(60.0)).unwrap();
        },
        ("list-ports", Some(_)) => {
            run_list_ports().unwrap();
        },
        _ => unreachable!()
    }
}
