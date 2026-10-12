#!/usr/bin/env bash

./build.sh

if [ $? -eq 0 ]; then 
    echo "Build successful. Running the simulator with the new WASM file..."

    pushd ../sim
    cargo run --release -p sim -- run -i line_follower_robot.wasm
    popd
fi
