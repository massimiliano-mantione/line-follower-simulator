#!/usr/bin/env bash

# wit-bindgen rust ../wit/world.wit --out-dir .
# mv ./line_follower_robot.rs src/wasm_bindings.rs
cargo build --target wasm32-wasip2 --release

if [ $? -eq 0 ]; then 
    cp target/wasm32-wasip2/release/line_follower_robot.wasm ../sim
    echo "Build complete. The WASM file has been copied to the sim directory."
fi
