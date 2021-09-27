use boucle;
use boucle::op_sequence;
use boucle::Sample;
use boucle::OpSequence;

use clap::{Arg, App};
use hound;
use midir::{MidiInput, Ignore};

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;use std::thread::sleep;
use std::time::Duration;

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

fn run_batch(audio_in: &str, audio_out: &str, operations_file: &str) {
    let op_sequence = read_ops(&operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        println!("{}", op);
    }

    println!("Reading input...");
    let mut reader = hound::WavReader::open(audio_in).unwrap(); //expect("Failed to read input");
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    let buffer: Vec<Sample> = reader.samples::<Sample>().map(|s| s.unwrap()).collect();

    let mut writer = hound::WavWriter::create(audio_out, spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(boucle::Config::default());
    boucle.process_buffer(&buffer, &op_sequence, &mut |s| writer.write_sample(s).unwrap());
    writer.finalize().unwrap();
}

fn run_live(midi_in_port: i32) -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("boucle input")?;
    midi_in.ignore(Ignore::None);

    let port = &midi_in.ports()[midi_in_port as usize];
    let in_port_name = midi_in.port_name(&port)?;

    midi_in.connect(&port, &in_port_name,
                    |ts, msg, _data| { println!("{}, {:?}", ts, msg) },
                    ())?;

    loop {
         sleep(Duration::from_millis(4 * 150));
    };
    return Ok(())
}

fn list_ports() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("boucle input")?;
    midi_in.ignore(Ignore::None);

    println!("Available MIDI input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
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
                 .short("p")))
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
            run_live(midi_port).unwrap();
        },
        ("list-ports", Some(_)) => {
            list_ports().unwrap();
        },
        _ => unreachable!()
    }
}
