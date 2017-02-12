#!/usr/bin/env python3

'''Commandline interface for working with Boucle looper.

Currently only supports playing over JACK. On a modern Linux desktop you
probably have to block Pulseaudio and start the JACK daemon manually like this:

    pasuspender -- jackd -d alsa

'''

import gi
gi.require_version('Gst', '1.0')
from gi.repository import Gst

# Available from Pip: `pip install JACK-Client`
import jack

import argparse
import logging
import os
import signal
import subprocess
import sys
import time


BOUCLE_URL = "http://afuera.me.uk/boucle"

def argument_parser():
    parser = argparse.ArgumentParser(description="Boucle looper misbehaver CLI wrapper")

    parser.add_argument(
            '--control', '-c', type=str, choices=['midi', 'random'], action='append',
            help="what drives you? Default: midi and random.")

    parser.add_argument(
            '--input', '-i', type=str, action='append',
            help="a JACK port, or an audio file to loop. Default is to connect "
                 "to all system:capture_* ports. Multiple inputs are allowed.")
    parser.add_argument(
            '--output', '-o', type=str, action='append',
            help="JACK port for audio output. Default is to connect to all "
                 "system:playback_* ports. Multiple outputs are allowed.")

    parser.add_argument(
            '--loop-length', '-l', default=8.0, type=float,
            help="length of loop (beats)")

    # These two are either-or
    parser.add_argument(
            '--tempo', '-t', default=49.58, type=float,
            help="tempo that we play at (beats per minute)")
    parser.add_argument(
            '--tempo-from-midi', '-m', default=False, type=bool,
            help="sync to MIDI input (requires --control=midi)")

    parser.add_argument(
            '--click', '-k', default=False, action='store_true',
            help="enable an audible click sound on each beat")

    # For random controller
    parser.add_argument(
            '--randomness', '-r', default=0.5, type=float,
            help="for random controller: how random are you")

    return parser


def plugin_process(plugin_params):
    '''Load the Boucle plugin.'''

    # It's ugly to just call out to `jalv` to do this, but it works. Perhaps in
    # future we could use this: https://github.com/moddevices/mod-host

    command = ['jalv']
    for param, value in plugin_params.items():
        command.extend(['-c', '%s=%s' % (param, value)])
    command +=[BOUCLE_URL]

    logging.info("Running plugin: " + ' '.join(command))
    process = subprocess.Popen(command)
    return process


def audio_file_play_loop(path, jack_port):
    '''Play an audio file on a loop.

    Returns a GStreamer pipeline.

    '''

    # FIXME: it's less fragile to build the pipeline using Gst.ElementFactory.make()
    pipeline = Gst.parse_launch(
        'multifilesrc location="%s" loop=true ! decodebin ! audioresample ! '
        'audioconvert ! jackaudiosink client-name=demo '
        'port-pattern="%s"' % (path, jack_port))
    pipeline.set_state(Gst.State.PLAYING)
    return pipeline


def await_jack_port(jack_client, port_name, timeout=None):
    start_time = time.time()
    logging.info("Waiting for port %s to appear" % port_name);
    while True:
        try:
            jack_client.get_port_by_name(port_name)
            break
        except jack.JackError as e:
            # There's no way to get a notification when a JACK port appears, so to
            # wait for the plugin to start we do a busy loop... ah well.
            if timeout and (time.time() - start_time) > timeout:
                raise RuntimeError("Port %s did not appear within %i seconds",
                        port_name, timeout)
            time.sleep(0.1)
    logging.info("Port %s appeared after %f seconds", port_name, time.time() - start_time)


def click_process(tempo):
    '''Start a process to generate a click.'''

    # Yes, again we cheat and use a program from jack-example-clients.

    command = ['jack_metro', '--bpm', str(tempo)]
    logging.info("Starting metronome: " + ' '.join(command))
    return subprocess.Popen(command)


def main():
    args = argument_parser().parse_args()

    jack_client = jack.Client("boucle_cli")

    args.control = args.control or ['midi', 'random']

    if 'random' in args.control:
        jack_client.midi_outports.register("control_midi_out")

    subprocesses = []
    gstreamer_pipelines = []

    plugin_params = {
        'tempo': args.tempo,
        'loop_length': args.loop_length
    }

    try:
        subprocesses.append(plugin_process(plugin_params))
        await_jack_port(jack_client, "Boucle:in", timeout=5)

        if args.input:
            jack_input_ports = jack_client.get_ports(is_audio=True, is_input=True)
            jack_input_port_names = [port.name for port in jack_input_ports]
            for i in args.input:
                if i in jack_input_port_names:
                    jack_client.connect(input_port, "Boucle:in")
                elif os.path.exists(i):
                    Gst.init()
                    gstreamer_pipelines.append(audio_file_play_loop(i, "Boucle:in"))
                else:
                    raise RuntimeError ("Invalid input %s. Please pass a valid JACK port "
                                        "or audio file." % i)
        else:
            for input_port in jack_client.get_ports(name_pattern="system:capture_.*"):
                jack_client.connect(input_port, "Boucle:in")

        # FIXME: we make no effort to ensure that the metronome is in sync with
        # any audio files that are playing. We should probably slave the
        # jackaudiosink and the metro to the JACK transport, then start the
        # transport when both are ready.
        if args.click:
            click_port = 'metro:%s_bpm' % args.tempo
            subprocesses.append(click_process(args.tempo))
            await_jack_port(jack_client, click_port, timeout=5)
        else:
            click_port = None

        if args.output:
            for output_port in args.output:
                jack_client.connect("Boucle:out", output_port)
                if click_port:
                    jack_client.connect(click_port, output_port)
        else:
            for output_port in jack_client.get_ports(name_pattern="system:playback_.*"):
                jack_client.connect("Boucle:out", output_port)
                if click_port:
                    jack_client.connect(click_port, output_port)

        signal.pause()
    finally:
        for pipeline in gstreamer_pipelines:
            pipeline.set_state(Gst.State.NULL)

        for subprocess in subprocesses:
            subprocess.terminate()
            subprocess.wait()


try:
    logging.basicConfig(stream=sys.stderr, level=logging.DEBUG)
    main()
except RuntimeError as e:
    sys.stderr.write("%s\n" % e)
    sys.exit(1)
