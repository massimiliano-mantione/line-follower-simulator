use wasm_bindings::devices::TimeUs;
pub use wasmtime;
pub mod mock_stepper;
pub mod wasm_bindings;
pub mod wasm_executor;
pub mod wasm_host;

pub const TOTAL_SIMULATION_TIME_US: TimeUs = 60_000_000;

pub fn test_run(input: String, output: String, logs: bool) -> wasmtime::Result<()> {
    // Load the component from disk
    let wasm_bytes = std::fs::read(&input)?;

    // Get configuration
    let cfg = wasm_executor::get_robot_configuration(&wasm_bytes)?;
    println!("Robot configuration: {:#?}", &cfg);

    // Run robot logic
    wasm_executor::run_robot_simulation(
        &wasm_bytes,
        cfg,
        TOTAL_SIMULATION_TIME_US,
        Some(output.into()),
        logs,
    )?;

    Ok(())
}
