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
    // EXERCISE 1.1: instantiate the robot component and call its `setup` export.
    //
    // Build an engine with fuel consumption enabled, put a `BotHost` in a store,
    // instantiate the component and call `setup()` to get the robot's own
    // description of its body.
    //
    // Two things make this different from `run_robot_simulation` below:
    //  - the time budget is tiny (1 ms): reading a configuration should be nearly
    //    free, and a robot that spends a whole race computing it is broken;
    //  - there is no simulator yet. Building the Bevy world needs the robot's
    //    dimensions, which is what we are about to ask for, so `setup` runs
    //    against a `MockStepper` instead. See the `SimulationStepper` seam.
    //
    // Careful: instantiating a component *executes* WASM, so it consumes fuel, and
    // a store with no fuel traps immediately.
    //
    // See: wasmtime::Config::consume_fuel, Store::set_fuel,
    //      component::Component::new, component::Linker,
    //      Linker::define_unknown_imports_as_traps

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

    // A fresh budget, then ask the robot to describe itself
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
    // EXERCISE 1.2: run the robot's `run` export against the real simulator.
    //
    // Same shape as `get_robot_configuration`, plus two differences:
    //  - the host functions must actually be wired in, so the robot can read
    //    sensors and drive motors (`add_to_linker`);
    //  - the budget is the whole race, and `BotHost` derives the simulated clock
    //    from how much of it the robot has spent.
    //
    // The sandbox matters here: this binary may come from a competitor over HTTP.
    // It gets no WASI, no filesystem and no network - only what `wit/world.wit`
    // declares. Anything else it imports must trap.
    //
    // Afterwards, flush the robot's log and return the recorded execution data.
    //
    // See: LineFollowerRobot::add_to_linker, component::HasSelf,
    //      BotHost::{write_log_file, get_execution_data}

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

    // A fresh budget for the whole race, then run it
    store.set_fuel(fuel_for_time_us(total_simulation_time))?;
    println!("fuel before run: {}", store.get_fuel()?);
    robot_component.robot().call_run(&mut store)?;
    println!("remaining fuel after run: {}", store.get_fuel()?);

    // Flush the robot's log and hand back the recording
    let host = store.data_mut();
    host.write_log_file();
    let data = host.get_execution_data();

    Ok(data)
}
