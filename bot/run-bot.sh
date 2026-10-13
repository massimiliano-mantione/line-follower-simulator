#!/usr/bin/env bash

./build.sh

if [ $? -eq 0 ]; then 
    echo "Build successful. Running the simulator with the new WASM file..."

    if [ -z "$1" ]; then
        track=""
    else
        track="-t $1"
    fi

    pushd ../sim
    cargo run --release -p sim -- $track run -i line_follower_robot.wasm
    popd
fi
