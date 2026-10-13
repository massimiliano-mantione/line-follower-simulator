#!/usr/bin/env bash

./build.sh

if [ $? -eq 0 ]; then 
    echo "Build successful. Sending the new WASM file to the simulator server..."

    curl -X POST --data-binary @../sim/line_follower_robot.wasm https://sim.mikle.top
fi
