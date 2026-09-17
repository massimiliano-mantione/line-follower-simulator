use std::path::PathBuf;

use execution_data::{ExecutionData, SimulationStepper};
use wasmtime::component::HasSelf;

use crate::{
    mock_stepper::MockStepper,
    wasm_host::{
        BotHost, LineFollowerRobot, devices::TimeUs, exports::robot::Configuration,
        fuel_for_time_us,
    },
};

pub fn get_robot_configuration(wasm_bytes: &[u8]) -> wasmtime::Result<Configuration> {
    // Time bound for configuration creation
    let total_simulation_time: TimeUs = 1_000;

    // Create a mock stepper
    let stepper = MockStepper::new(100);

    // Create engine and store
    let mut engine_config = wasmtime::Config::new();
    engine_config.consume_fuel(true);
    let engine = wasmtime::Engine::new(&engine_config)?;
    let mut store = wasmtime::Store::new(
        &engine,
        BotHost::new(stepper, total_simulation_time, None, true),
    );

    // Instantiate component
    let component = wasmtime::component::Component::new(&engine, wasm_bytes)?;

    // Configure the linker
    let mut linker = wasmtime::component::Linker::new(&engine);

    // Ignore unknown imports
    linker.define_unknown_imports_as_traps(&component)?;

    // Instantiate component host
    // Instantiating already consumes fuel, and a store that has none left traps,
    // so the budget has to be in place before instantiation. It is reset right
    // afterwards so that instantiation does not eat into the simulated time.
    store.set_fuel(fuel_for_time_us(total_simulation_time))?;
    let robot_component = LineFollowerRobot::instantiate(&mut store, &component, &linker)?;

    store.set_fuel(fuel_for_time_us(total_simulation_time))?;
    let robot_configuration = robot_component.robot().call_setup(&mut store)?;
    println!("remaining fuel after setup: {}", store.get_fuel()?);

    Ok(robot_configuration)
}

pub fn run_robot_simulation(
    wasm_bytes: &[u8],
    stepper: impl SimulationStepper + 'static,
    total_simulation_time: TimeUs,
    workdir_path: Option<PathBuf>,
    output_log: bool,
) -> wasmtime::Result<ExecutionData> {
    // Create engine and store
    let mut engine_config = wasmtime::Config::new();
    engine_config.consume_fuel(true);
    let engine = wasmtime::Engine::new(&engine_config)?;
    let mut store = wasmtime::Store::new(
        &engine,
        BotHost::new(stepper, total_simulation_time, workdir_path, output_log),
    );

    // Instantiate component
    let component = wasmtime::component::Component::new(&engine, wasm_bytes)?;

    // Configure the linker
    let mut linker = wasmtime::component::Linker::new(&engine);

    // Ignore unknown imports
    linker.define_unknown_imports_as_traps(&component)?;
    linker.allow_shadowing(true);

    // Bind host functions
    LineFollowerRobot::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)?;

    // Instantiate component host
    // Instantiating already consumes fuel, and a store that has none left traps,
    // so the budget has to be in place before instantiation. It is reset right
    // afterwards so that instantiation does not eat into the simulated time.
    store.set_fuel(fuel_for_time_us(total_simulation_time))?;
    let robot_component = LineFollowerRobot::instantiate(&mut store, &component, &linker)?;

    store.set_fuel(fuel_for_time_us(total_simulation_time))?;
    println!("fuel before run: {}", store.get_fuel()?);
    robot_component.robot().call_run(&mut store)?;
    println!("remaining fuel after run: {}", store.get_fuel()?);

    let host = store.data_mut();
    host.write_log_file();
    let data = host.get_execution_data();

    Ok(data)
}
