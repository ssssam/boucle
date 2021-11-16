#!/bin/bash

set -em

echo "boucle run.sh"

FW_SCRIPTS_DIR=/root/fw_dir/scripts

$FW_SCRIPTS_DIR/start-jack.sh
echo "Started jackd as $(cat /tmp/pids/jack.pid)"

jack_wait --wait

export RUST_LOG=warn
./boucle_organelle &
echo $! > /tmp/pids/boucle_organelle.pid
echo "Started boucle_organelle as $(cat /tmp/pids/boucle_organelle.pid)"

trap 'echo "Received SIGINT"; $FW_SCRIPTS_DIR/killpatch.sh; jack_wait --quit' SIGINT
trap 'echo "Received EXIT/ERR/TERM"; $FW_SCRIPTS_DIR/killpatch.sh; jack_wait --quit' EXIT ERR SIGTERM

while ! jack_lsp boucle | grep --silent boucle:boucle_in; do
    echo "Waiting for patch audio ports..."
    jack_lsp boucle
    sleep 1
done
# Sleep after ports appear so they are active: otherwise you see errors:
#
#   cannot connect ports owned by inactive clients; "boucle" is not active
sleep 1

echo "Connecting ports"
jack_connect boucle:boucle_in system:capture_1
jack_connect boucle:boucle_in system:capture_2
jack_connect boucle:boucle_out system:playback_1
jack_connect boucle:boucle_out system:playback_2

fg %1
