use cpal::traits::{DeviceTrait, HostTrait};
use portmidi::{PortMidi};

use crate::app_error::*;

pub fn run_list_ports() -> Result<(), AppError> {
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
