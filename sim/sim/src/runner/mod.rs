use bevy::app::App;
use execution_data::{ExecutionData, MotorDriversDutyCycles, SensorsData};
use executor::wasmtime;

struct AppWrapper {
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

    pub fn step(&mut self) {
        self.app.update();
        self.sensors_data = *self.app.world().get_resource::<SensorsData>().unwrap();
    }

    pub fn get_execution_data(&mut self) -> ExecutionData {
        let mut res = self
            .app
            .world_mut()
            .get_resource_mut::<ExecutionData>()
            .unwrap();

        ExecutionData {
            steps: res.steps.drain(..).collect(),
        }
    }
}

pub struct RunnerStepper {
    app_wrapper: AppWrapper,
    current_step: usize,
    current_time_us: u32,
}

impl RunnerStepper {
    pub fn new(app_wrapper: AppWrapper) -> Self {
        Self {
            app_wrapper,
            current_step: 0,
            current_time_us: 0,
        }
    }

    pub fn time_s(&self) -> f32 {
        self.current_time_us as f32 / 1_000_000.0
    }
}

impl execution_data::SimulationStepper for RunnerStepper {
    const STEP_US: u32 = 100;

    fn step(&mut self) {
        self.app_wrapper.step();
        self.current_step += 1;
        self.current_time_us += Self::STEP_US;
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

    fn get_accel(&self) -> execution_data::AccelData {
        execution_data::AccelData {
            front: 0.0,
            side: 0.0,
            vertical: -9.81,
        }
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
        true
    }
}

pub fn simulator_runner(input: String, output: String, logs: bool) -> wasmtime::Result<()> {
    // let stepper = RunnerStepper::new(AppWrapper::new(app));

    Ok(())
}
