#!/usr/bin/env bash

export RPC_URL=ws://127.0.0.1:9944
export PRIVATE_KEY=$RELAYER_PRIVATE_KEY

node init.js &

./node-template --dev --ws-external --base-path /tmp/substrate
