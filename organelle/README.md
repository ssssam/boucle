# Boucle for Organelle

The `boucle_organelle` program is intended to work on the [Organelle](https://www.critterandguitari.com/organelle)
from Critter and Guitari.

I test it on Organelle 1 and build using [this Buildroot tree](https://gitlab.com/samthursfield/organelle-ports/).

## Building

Assuming you have built the organelle-ports tree or unpacked the SDK, you can
build for Organelle like this:

    export SDK_PATH=/home/sam/src/organelle-ports/output/host
    env PKG_CONFIG_ALLOW_CROSS=1 PATH="$SDK_PATH/bin:$SDK_PATH/sbin:$PATH" CARGO_HOME=$SDK_PATH/share/cargo $SDK_PATH/bin/cargo build --release --target=armv7-unknown-linux-gnueabihf

## Manual testing

You can control the program on a developer machine using `oscsend` and `oscdump`.

This command will dump all messages the patch sends to the firmware:

    oscdump osc.udp://:4001

This command simulates pressing and releasing the 'aux' key.

    oscsend osc.udp://:4000 /key ii 0 100
    oscsend osc.udp://:4000 /key ii 0 0

The organ keys are numbered 1-24 from C4 to E6.

This command simulates setting all knobs to their maximum value. 

    oscsend osc.udp://:4000 /knobs iiiiii 1023 1023 1023 1023 1023

The order of values: is knobs 1-4, encoder (disabled by default) then
expression pedal.

