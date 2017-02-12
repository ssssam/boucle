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

Build 4 testing:

	mkdir build
	meson ..
	mesonconf -Dprefix=`pwd`/../prefix
	ninja-build install

	# this should show you the plugins name
	LV2_PATH=`pwd`/../prefix/lib64/lv2 lv2ls

	# this should run it under jack
	LV2_PATH=`pwd`/../prefix/lib64/lv2 jalv http://afuera.me.uk/boucle

	# you can see the Boucle: ports in JACK now
	jack_lsp

	# generate a noise with JACK
	jack_metro --bpm 500 &

	jack_connect metro:500_bpm Boucle:in
	jack_connect Boucle:out system:playback_1

	# Ur noise comes out thru the plugin. It's horrible!

You're welcome.

Playing a BEAT:

    # run the loopy looper
    LV2_PATH=`pwd`/../prefix/lib64/lv2 jalv http://afuera.me.uk/boucle

    # Looping playback of yer audio file to Boucle
    gst-launch-1.0 multifilesrc \
        location=`pwd`/examples/ibeat.org-j1s-SynthArpBuildLoop-97bpm.mp3 \
        loop=true ! mad ! audioresample ! audioconvert ! \
      jackaudiosink client-name=demo port-pattern=Boucle:in

    jack_connect Boucle:out system:playback_1

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

The core of Boucle is an [LV2](http://lv2plug.in/) plugin. This implements a
beat-synchronised delay which can be synchronized and controlled over MIDI.

The Boucle plugin can be used as-is inside an LV2 host, like the
[http://ardour.org/](Ardour) audio sequencer. However, its primary use case is
to work live, and tooling is provided to make this easier. Currently the only
tooling is a command-line tool, but a graphical user interface would be
welcome.

There are various similar plugins which contain their own mini sequencers, but
I don't like that so much. Feel free to use any kind of sequencer
([Cythar](https://www.youtube.com/watch?v=gtM2DpA8Z54)?
[Non](http://non.tuxfamily.org/wiki/index.php?page=Non%20Sequencer)?
[Iannix](https://www.iannix.org/en/whatisiannix/)? to drive Boucle, or some
[generative algortihm](http://www.flexatone.org/article/athenaFeatAlgo) but
let's not try and build such things in.[1]

[1]. The downside to this approach is that most sequencers want to output
musical notes, and mapping those to operations on a delay buffer is going to
suck a bit. But the generic automation sequencing in DAWs is even less suited
to driving Boucle so we'll have to try and map our ideas to notes, at least
until the world decides to trade harmony for digital glitches. You'll know
when that happens because I'll be out picking up old pianos.
