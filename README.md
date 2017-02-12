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

[iZotope Stutter Edit](https://www.izotope.com/en/products/create-and-design/stutter-edit.html) ([video](https://www.youtube.com/watch?v=68U2egYkoWs), [review](http://www.soundonsound.com/reviews/izotope-stutter-edit))

[Effectrix](http://www.kvraudio.com/product/effectrix-by-sugar-bytes/details) ([video](https://www.youtube.com/watch?v=lsk1mJ_vwZw))

[Roland Scooper](https://www.roland.com/global/products/scooper/) ([video](https://www.youtube.com/watch?v=l_e_IUgKlGQ))

[The Finger](https://www.native-instruments.com/en/products/komplete/effects/the-finger/) ([video](https://www.youtube.com/watch?v=wrj6pkQloJM))

Tweakbench [Dropout](http://www.tweakbench.com/dropout) and [Yoink](http://www.tweakbench.com/yoink)

[Vox Dynamic Looper](http://www.voxamps.com/vdl1)

## Architecture

The core of Boucle is an [LV2](http://lv2plug.in/) plugin. This plugins is a
delay buffer that can perform various transformations operation on its
playhead.

The operations are defined using the [LV2
Atoms](http://lv2plug.in/ns/ext/atom/) extension. This gives a set of abstract
privimites capable of defining streams of events, it's used as the basis for
the [LV2 MIDI](http://lv2plug.in/ns/ext/midi) extension for example.

Boucle operations can't be described meaningfully using MIDI primitives, but
there are various ways to map them to MIDI notes.

The Boucle engine can theoretically be used as a plugin in any LV2 host, but
since it uses a custom control protocol it probably needs to be used with a
MIDI->Boucle control mapping.

Its primary use case is to work live as a loop butcher, and tooling is provided
to make this easier. Currently the only tooling is a command-line tool, but a
graphical user interface would be welcome.

There are various similar plugins which contain their own mini sequencers, but
I don't like that so much. I would prefer if you could link up any
sequencer you want
([stepseq.lv2](https://github.com/x42/stepseq.lv2/)?
[Cythar](https://www.youtube.com/watch?v=gtM2DpA8Z54)?
[Non](http://non.tuxfamily.org/wiki/index.php?page=Non%20Sequencer)?
[Iannix](https://www.iannix.org/en/whatisiannix/)? to drive Boucle, or some
[generative algortihm](http://www.flexatone.org/article/athenaFeatAlgo). There
are probably none that can drive the custom control protocol, so we need thought
on how to hook up MIDI inputs.
