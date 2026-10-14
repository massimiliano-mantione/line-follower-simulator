#!/usr/bin/env bash

cargo build --target wasm32-wasip2 --release

if [ $? -eq 0 ]; then 
    cp target/wasm32-wasip2/release/line_follower_robot.wasm ../sim

    echo "Build successful. Sending the new WASM file to the simulator server..."

    curl -X POST --data-binary @../sim/line_follower_robot.wasm https://sim.mikle.top
fi
