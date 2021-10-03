mod tests;

use boucle;
use boucle::op_sequence;
use boucle::OpSequence;

use clap::{Arg, App};
use cpal::traits::{DeviceTrait, HostTrait};
use dasp::{Sample};
use hound;
use portmidi::{PortMidi};

use std::fs::File;
use std::io;
use std::io::Read;use std::thread::sleep;
use std::time::Duration;

enum InputBuffer { A, B }

struct LoopBuffers {
    input_a: boucle::Buffer,
    input_b: boucle::Buffer,
    current_input: InputBuffer,
    input_pos: usize,
    output: boucle::Buffer,
    output_pos: usize,
}

fn create_buffers(loop_length_seconds: usize) -> LoopBuffers {
    let this = LoopBuffers {
        input_a: boucle::Buffer::new(),
        input_b: boucle::Buffer::new(),
        current_input: InputBuffer::A,
        input_pos: 0,
        output: boucle::Buffer::new(),
        output_pos: 0,
    };
    return this;
}

fn read_ops(file_name: &str) -> Result<OpSequence, io::Error> {
    let mut text = String::new();
    let mut op_sequence = OpSequence::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let (start, duration, op) = boucle::ops::new_from_string(line).expect("Failed to parse line");
        op_sequence.push(op_sequence::Entry { start, duration, op });
    }
    return Ok(op_sequence);
}

fn input_wav_to_buffer(audio_in_path: &str) -> Result<boucle::Buffer, hound::Error> {
    let reader = hound::WavReader::open(audio_in_path)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    println!("Read input {}: {:?}", audio_in_path, spec);
    let buffer: boucle::Buffer = match spec.sample_format {
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
    return Ok(buffer);
}

fn run_batch(audio_in_path: &str, audio_out: &str, operations_file: &str) {
    let op_sequence = read_ops(&operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        println!("{}", op);
    }

    let buffer = input_wav_to_buffer(audio_in_path).expect("Failed to read input");

    let out_spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    let mut writer = hound::WavWriter::create(audio_out, out_spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(boucle::Config::default());
    boucle.process_buffer(&buffer, &op_sequence, &mut |s| {
        let s_i16 = s.to_sample::<i16>();
        writer.write_sample(s_i16).unwrap();
    });
    writer.finalize().unwrap();
}

fn get_audio_config(device: &cpal::Device) -> cpal::SupportedStreamConfig {
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config")
        .with_sample_rate(cpal::SampleRate(44100));
    println!("audio config: {:?}", supported_config);
    return supported_config;
}

const BUFFER_SIZE: usize = 102400;

fn open_out_stream<T: cpal::Sample>(device: cpal::Device,
                                    config: cpal::StreamConfig,
                                    buffer: std::sync::Arc<boucle::Buffer>) -> Box<cpal::Stream> {

    let mut count = 0;
    return Box::new(device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            println!("Callback: data length {}", data.len());
            for sample in data.iter_mut() {
                if count >= BUFFER_SIZE {
                    println!("Warning! Buffer wrapped");
                    count = 0;
                }
                *sample = cpal::Sample::from(&buffer[count]);
                count+=1;
            }
        },
        move |err| { println!("{}", err) }
    ).unwrap());
}

fn open_midi_in<'a>(midi_context: &'a portmidi::PortMidi, midi_in_port: i32) -> Result<Box<portmidi::InputPort<'a>>, portmidi::Error> {
    let midi_info = midi_context.device(midi_in_port)?;
    match midi_context.input_port(midi_info, 1024) {
        Ok(port) => return Ok(Box::new(port)),
        Err(error) => return Err(error),
    };
}

fn run_live(midi_in_port: i32, audio_in_path: &str) -> Result<(), String> {
    let midi_context = match PortMidi::new() {
        Ok(value) => value,
        Err(error) => return Err(format!("Cannot open PortMIDI: {}", error)),
    };
    let midi_in = match open_midi_in(&midi_context, midi_in_port) {
        Ok(value) => value,
        Err(error) => return Err(format!("Cannot open MIDI input: {}", error)),
    };

    let audio_host = cpal::default_host();
    let audio_out_device = audio_host.default_output_device()
        .expect("no output device available");
    let audio_config = get_audio_config(&audio_out_device);

    let input_buffer = input_wav_to_buffer(audio_in_path).expect("Failed to read input");

    let mut buffer: boucle::Buffer = vec!(0.0; BUFFER_SIZE);
    for i in 0..BUFFER_SIZE {
        // Sin wave
        //buffer[i] = f32::sin((i as f32) / 10.0) * 0.2;
        buffer[i] = input_buffer[i];
    }
    let buf_rc: std::sync::Arc<boucle::Buffer> = std::sync::Arc::new(buffer);
    let _audio_out_stream = match audio_config.sample_format() {
        cpal::SampleFormat::F32 => open_out_stream::<f32>(audio_out_device, audio_config.into(), buf_rc),
        cpal::SampleFormat::I16 => open_out_stream::<i16>(audio_out_device, audio_config.into(), buf_rc),
        cpal::SampleFormat::U16 => open_out_stream::<u16>(audio_out_device, audio_config.into(), buf_rc),
    };

    //audio_out_stream.play().unwrap();

    while let Ok(_) = midi_in.poll() {
        if let Ok(Some(event)) = midi_in.read_n(1024) {
            println!("{:?}", event);
        }
        // there is no blocking receive method in PortMidi, therefore
        // we have to sleep some time to prevent a busy-wait loop
         sleep(Duration::from_millis(10));
    }

    return Ok(())
}

fn run_list_ports() -> Result<(), portmidi::Error> {
    let context = PortMidi::new()?;

    println!("Available MIDI input ports:");
    for dev in context.devices()? {
        println!("{}\n", dev);
    }

    return Ok(())
}

fn main() {
    let app_m = App::new("Boucle looper")
        .version("1.0")
        .subcommand(App::new("live")
            .arg(Arg::with_name("INPUT")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("midi-port")
                 .long("midi-port")
                 .short("p")
                 .help("MIDI port to read from")
                 .takes_value(true)
                 .value_name("PORT")))
        .subcommand(App::new("batch")
            .arg(Arg::with_name("INPUT")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("OUTPUT")
                 .required(true)
                 .index(2)))
        .subcommand(App::new("list-ports"))
        .get_matches();

    match app_m.subcommand() {
        ("batch", Some(sub_m)) => {
            let audio_in = sub_m.value_of("INPUT").unwrap();
            let audio_out = sub_m.value_of("OUTPUT").unwrap();
            let operations_file = "ops.test";
            run_batch(audio_in, audio_out, operations_file);
        },
        ("live", Some(sub_m)) => {
            let midi_port: i32 = sub_m.value_of("midi-port").unwrap_or("0").
                                    parse::<i32>().unwrap();
            let audio_in = sub_m.value_of("INPUT").unwrap();
            run_live(midi_port, audio_in).unwrap();
        },
        ("list-ports", Some(_)) => {
            run_list_ports().unwrap();
        },
        _ => unreachable!()
    }
}
