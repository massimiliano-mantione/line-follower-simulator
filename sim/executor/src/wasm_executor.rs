#![allow(unused_imports, unused_variables)]

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
    todo!("build the engine, instantiate the component, call setup()")
}

pub fn run_robot_simulation(
    wasm_bytes: &[u8],
    stepper: impl SimulationStepper + 'static,
    total_simulation_time: TimeUs,
    workdir_path: Option<PathBuf>,
    output_log: bool,
) -> wasmtime::Result<ExecutionData> {
    todo!("wire the host functions in, then call run()")
}
