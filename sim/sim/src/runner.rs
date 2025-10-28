use bevy::app::{App, AppExit};
use execution_data::{
    BodyExecutionData, ExecutionData, MotorDriversDutyCycles, SensorsData, WheelExecutionData,
};
use executor::{wasm_bindings::exports::robot::Configuration, wasm_executor, wasmtime};

use crate::{
    app_builder::{self, create_app},
    track::Track,
};

pub struct AppWrapper {
    app: App,
    sensors_data: SensorsData,
}

impl AppWrapper {
    pub fn new(app: App) -> Self {
        Self {
            app,
            sensors_data: SensorsData::default(),
        }
    }

    pub fn sensors_data(&self) -> &SensorsData {
        &self.sensors_data
    }

    pub fn set_motors(&mut self, dc: MotorDriversDutyCycles) {
        let mut res = self
            .app
            .world_mut()
            .get_resource_mut::<MotorDriversDutyCycles>()
            .unwrap();
        *res = dc;
    }

    pub fn step(&mut self, period_us: u32, next_time_us: u32, start_time_us: u32) {
        self.app.update();
        self.sensors_data = *self.app.world().get_resource::<SensorsData>().unwrap();

        // Get mutable ref to execution data
        let mut execution_data = self
            .app
            .world_mut()
            .get_resource_mut::<ExecutionData>()
            .unwrap();

        // Update activity data
        let activity_data = &mut execution_data.activity_data;
        if next_time_us >= start_time_us {
            activity_data.start_time_us = Some(start_time_us);
        }

        if next_time_us > period_us {
            if activity_data.end_time_us.is_none() {
                if self.sensors_data.is_over_track_end {
                    activity_data.end_time_us = Some(next_time_us);
                }
            }
            if activity_data.out_time_us.is_none() {
                if self.sensors_data.is_out_of_track {
                    activity_data.out_time_us = Some(next_time_us);
                }
            }
        }
    }

    pub fn get_execution_data(&mut self) -> ExecutionData {
        let mut res = self
            .app
            .world_mut()
            .get_resource_mut::<ExecutionData>()
            .unwrap();

        ExecutionData {
            body_data: BodyExecutionData {
                period: res.body_data.period,
                steps: res.body_data.steps.drain(..).collect(),
            },
            left_wheel_data: WheelExecutionData {
                period: res.left_wheel_data.period,
                side: res.left_wheel_data.side,
                steps: res.left_wheel_data.steps.drain(..).collect(),
            },
            right_wheel_data: WheelExecutionData {
                period: res.right_wheel_data.period,
                side: res.right_wheel_data.side,
                steps: res.right_wheel_data.steps.drain(..).collect(),
            },
            activity_data: res.activity_data,
        }
    }

    pub fn is_active(&self) -> bool {
        let execution_data = self.app.world().get_resource::<ExecutionData>().unwrap();
        execution_data.activity_data.is_active_now()
    }
}

pub struct RunnerStepper {
    step_period_us: u32,
    start_time_us: u32,
    app_wrapper: AppWrapper,
    current_step: usize,
    current_time_us: u32,
}

impl RunnerStepper {
    pub fn new(app_wrapper: AppWrapper, step_period_us: u32, start_time_us: u32) -> Self {
        Self {
            step_period_us,
            start_time_us,
            app_wrapper,
            current_step: 0,
            current_time_us: 0,
        }
    }
}

impl execution_data::SimulationStepper for RunnerStepper {
    fn step_us(&self) -> u32 {
        self.step_period_us
    }

    fn step(&mut self) {
        self.app_wrapper.step(
            self.step_period_us,
            self.current_time_us + self.step_period_us,
            self.start_time_us,
        );
        self.current_step += 1;
        self.current_time_us += self.step_period_us;
    }

    fn get_time_us(&self) -> u32 {
        self.current_time_us
    }

    fn get_step_count(&self) -> usize {
        self.current_step
    }

    fn get_line_sensors_left(&self) -> [f32; 8] {
        self.app_wrapper.sensors_data().line_sensors[0..8]
            .try_into()
            .expect("lenght should be 8")
    }

    fn get_line_sensors_right(&self) -> [f32; 8] {
        self.app_wrapper.sensors_data().line_sensors[8..]
            .try_into()
            .expect("lenght should be 8")
    }

    fn get_motor_angles(&self) -> execution_data::MotorAngles {
        self.app_wrapper.sensors_data().motor_angles
    }

    fn get_gyro(&self) -> execution_data::GyroData {
        self.app_wrapper.sensors_data().gyro
    }

    fn get_imu_fused_data(&self) -> execution_data::ImuFusedData {
        self.app_wrapper.sensors_data().imu_fused
    }

    fn set_motor_drivers_duty_cycles(
        &mut self,
        duty_cycles: execution_data::MotorDriversDutyCycles,
    ) {
        self.app_wrapper.set_motors(duty_cycles);
    }

    fn get_data(&mut self) -> execution_data::ExecutionData {
        self.app_wrapper.get_execution_data()
    }

    fn is_active(&self) -> bool {
        self.app_wrapper.is_active()
    }
}

#[derive(Clone)]
pub struct BotExecutionData {
    pub config: Configuration,
    pub data: ExecutionData,
}

pub fn get_bot_config_from_file(input: String) -> wasmtime::Result<Configuration> {
    // Load the component from disk
    let wasm_bytes = std::fs::read(&input)?;
    wasm_executor::get_robot_configuration(&wasm_bytes)
}

pub fn run_bot_from_file(
    input: String,
    output: Option<String>,
    logs: bool,
    step_period_us: u32,
    start_time_us: u32,
    track: Track,
) -> wasmtime::Result<BotExecutionData> {
    // Load the component from disk
    let wasm_bytes = std::fs::read(&input)?;
    run_bot_from_code(
        wasm_bytes,
        output,
        logs,
        step_period_us,
        start_time_us,
        track,
    )
}

pub fn run_bot_from_code(
    wasm_bytes: Vec<u8>,
    output: Option<String>,
    logs: bool,
    step_period_us: u32,
    start_time_us: u32,
    track: Track,
) -> wasmtime::Result<BotExecutionData> {
    // Get configuration
    let config = wasm_executor::get_robot_configuration(&wasm_bytes)?;
    println!("Robot configuration: {:#?}", &config);

    let (result_sender, result_receiver) = std::sync::mpsc::channel();

    create_app(
        app_builder::AppType::Simulator(config.clone()),
        track,
        step_period_us,
    )
    .set_runner(move |app| {
        let app_wrapper = AppWrapper::new(app);
        let stepper = RunnerStepper::new(app_wrapper, step_period_us, start_time_us);

        // Run robot logic
        let sim_result = wasm_executor::run_robot_simulation(
            &wasm_bytes,
            stepper,
            executor::TOTAL_SIMULATION_TIME_US,
            output.map(|output| output.into()),
            logs,
        );

        // Prepare bevy app result
        let app_result = if sim_result.is_ok() {
            AppExit::Success
        } else {
            AppExit::error()
        };

        // Send result back
        result_sender.send(sim_result).ok();
        app_result
    })
    .run();

    match result_receiver.recv() {
        Ok(result) => result.map(move |data| BotExecutionData { config, data }),
        Err(_) => Err(wasmtime::Error::msg("Failed to receive result")),
    }
}
