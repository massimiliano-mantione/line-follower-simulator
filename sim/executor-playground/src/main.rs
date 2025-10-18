use bindings::devices::TimeUs;

pub mod bindings;
pub mod bot_executor;
pub mod bot_wasm_host;
pub mod mock_stepper;

const TOTAL_SIMULATION_TIME_US: TimeUs = 60_000_000;

fn main() -> wasmtime::Result<()> {
    // Load the component from disk
    let wasm_bytes = std::fs::read("../bot/target/wasm32-wasip2/release/line_follower_robot.wasm")?;

    // Get configuration
    let cfg = bot_executor::get_robot_configuration(&wasm_bytes)?;
    println!("Robot configuration: {:#?}", &cfg);

    // Run robot logic
    bot_executor::run_robot_simulation(&wasm_bytes, cfg, TOTAL_SIMULATION_TIME_US, None, true)?;

    Ok(())
}
