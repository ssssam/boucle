# Boucle

Boucle is an incompetent glitch looper accompanist.

Version: 12 (using [negative
versioning](http://petermopar.blogspot.co.uk/2014/12/negative-versioning.html))

Maximum lines of code: 5000

## How to do it

Use cases:

 * destructing the boring loops that your drum machine creates (don't let
   the drum machine discover what you're doing though), beat synced via MIDI
 * Boucle can interact with a live instrument performance, although you will
   have to use a click if you want the beat to be particularly synced
 * glitching audio tracks in a DAW
 * record 2 or more loops with stop/start controls and you have a new kinda
   loop station

Build 4 testing:

	mkdir build
	meson ..
	mesonconf -Dprefix=`pwd`/../prefix
	ninja-build install

Test that thing with a file as input:

    LV2_PATH=`pwd`/../prefix/lib64/lv2 ../cli/boucle_cli.py \
        --input ../examples/ibeat.org-j1s-SynthArpBuildLoop-97bpm.mp3 \
        --tempo=97 --loop-length=16

Let's drive it with a virtual MIDI keyboard:

    jack-keyboard &
    LV2_PATH=`pwd`/../prefix/lib64/lv2 ../cli/boucle_cli.py \
        --control jack-keyboard:midi_out \
        --input ../examples/ibeat.org-j1s-SynthArpBuildLoop-97bpm.mp3 \
        --tempo=97 --loop-length=16
        
Hooray!

## Similar things

[Ciat-Lonbarde Cocoquantus](http://ciat-lonbarde.net/cocoquantus/index.html)

[dblue Glitch 2](http://illformed.com/glitch/)

Freakshow Industries [Backmask](https://freakshowindustries.com/backmask)

[iZotope Stutter Edit](https://www.izotope.com/en/products/create-and-design/stutter-edit.html) ([video](https://www.youtube.com/watch?v=68U2egYkoWs), [review](http://www.soundonsound.com/reviews/izotope-stutter-edit))

[Effectrix](http://www.kvraudio.com/product/effectrix-by-sugar-bytes/details) ([video](https://www.youtube.com/watch?v=lsk1mJ_vwZw))

[Roland Scooper](https://www.roland.com/global/products/scooper/) ([video](https://www.youtube.com/watch?v=l_e_IUgKlGQ))

[The Finger](https://www.native-instruments.com/en/products/komplete/effects/the-finger/) ([video](https://www.youtube.com/watch?v=wrj6pkQloJM))

Tweakbench [Dropout](http://www.tweakbench.com/dropout) and [Yoink](http://www.tweakbench.com/yoink)

[Vox Dynamic Looper](http://www.voxamps.com/vdl1)

## Architecture

The core of Boucle is inside an [LV2](http://lv2plug.in/) plugin. This plugin
provides a delay buffer and can perform various transformation operations on
the position of playhead while playing from it.

The core has audio input and output ports, and a control port for sending
transformation operations. The operations are defined using a custom protocol
built on the [LV2 Atoms](http://lv2plug.in/ns/ext/atom/) extension.

The Boucle LV2 plugin also contains the MIDI bridge. This could theoretically
be separated into a different plugin, but we would hit the issue that JACK
doesn't understand LV2 Atom ports so it would be a pain in the ass trying to
to link the MIDI bridge to the core plugin in most cases.

To make Boucle more "playable", the MIDI bridge handles tempo syncing so that
it can map notes to things like "stutter for 1 beat".

Boucle's primary use case is to work live as a loop butcher, and tooling is
provided to make this easier. Currently the only tooling is a command-line
tool, but a graphical user interface would be welcome.

Boucle could also work with a sequencer. There's no sequencer UI that
currently supports generating Boucle events, but you can connect anything
to the MIDI bridge ([stepseq.lv2](https://github.com/x42/stepseq.lv2/)?
[Cythar](https://www.youtube.com/watch?v=gtM2DpA8Z54)?
[Non](http://non.tuxfamily.org/wiki/index.php?page=Non%20Sequencer)?
[Iannix](https://www.iannix.org/en/whatisiannix/), even some
[generative algorithm](http://www.flexatone.org/article/athenaFeatAlgo).
