#!/usr/bin/env bash

cargo build --target wasm32-wasip2 --release

if [ $? -eq 0 ]; then 
    cp target/wasm32-wasip2/release/line_follower_robot.wasm ../sim

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
