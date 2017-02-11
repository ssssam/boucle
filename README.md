# Boucle

Boucle is an incompetent glitch looper accompanist.

Version: 12 (using [negative
versioning](http://petermopar.blogspot.co.uk/2014/12/negative-versioning.html))

Maximum lines of code: 5000

## How to do it

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

[Vox Dynamic Looper](http://www.voxamps.com/vdl1)