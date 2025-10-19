use std::f32::consts::PI;

pub struct MockStepper {
    current_step: usize,
    current_time_us: u32,
}

impl MockStepper {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            current_time_us: 0,
        }
    }

    pub fn time_s(&self) -> f32 {
        self.current_time_us as f32 / 1_000_000.0
    }
}

impl execution_data::SimulationStepper for MockStepper {
    const STEP_US: u32 = 100;

    fn step(&mut self) {
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
        [0.0; 8]
    }

    fn get_line_sensors_right(&self) -> [f32; 8] {
        [0.0; 8]
    }

    fn get_motor_angles(&self) -> execution_data::MotorAngles {
        execution_data::MotorAngles {
            left: self.time_s() * PI * 2.0,
            right: self.time_s() * PI * 2.0,
        }
    }

    fn get_accel(&self) -> execution_data::AccelData {
        execution_data::AccelData {
            front: 0.0,
            side: 0.0,
            vertical: -9.81,
        }
    }

    fn get_gyro(&self) -> execution_data::GyroData {
        execution_data::GyroData {
            roll_angular_speed: 0.0,
            pitch_angular_speed: 0.0,
            yaw_angular_speed: 0.0,
        }
    }

    fn get_imu_fused_data(&self) -> execution_data::ImuFusedData {
        execution_data::ImuFusedData {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    fn set_motor_drivers_duty_cycles(
        &mut self,
        _duty_cycles: execution_data::MotorDriversDutyCycles,
    ) {
        // Do nothing
    }

    fn get_data(&self) -> execution_data::ExecutionData {
        execution_data::ExecutionData { steps: Vec::new() }
    }

    fn is_active(&self) -> bool {
        true
    }
}
