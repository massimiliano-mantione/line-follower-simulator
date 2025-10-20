use bevy::{ecs::resource::Resource, transform::components::Transform};

#[derive(Clone, Copy)]
pub struct ExecutionStep {
    pub time_us: u32,
    pub body_transform: Transform,
    pub left_wheel_transform: Transform,
    pub right_wheel_transform: Transform,
}

pub struct ExecutionData {
    pub steps: Vec<ExecutionStep>,
}

/// Motor drivers duty cycles.
#[derive(Clone, Copy, Default)]
pub struct MotorDriversDutyCycles {
    pub left: i16,
    pub right: i16,
}

/// Motor angles in radians.
#[derive(Clone, Copy, Default)]
pub struct MotorAngles {
    pub left: f32,
    pub right: f32,
}

/// Accelerometer data in m/s².
#[derive(Clone, Copy, Default)]
pub struct AccelData {
    pub front: f32,
    pub side: f32,
    pub vertical: f32,
}

/// Gyroscope data in rad/s.
#[derive(Clone, Copy, Default)]
pub struct GyroData {
    pub roll_angular_speed: f32,
    pub pitch_angular_speed: f32,
    pub yaw_angular_speed: f32,
}

/// Fused IMU data in radians.
#[derive(Clone, Copy, Default)]
pub struct ImuFusedData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

/// Wrapper for all sensors data.
#[derive(Clone, Copy, Resource, Default)]
pub struct SensorsData {
    pub motor_angles: MotorAngles,
    pub accel: AccelData,
    pub gyro: GyroData,
    pub imu_fused: ImuFusedData,
}

pub trait SimulationStepper {
    /// Time per step in microseconds.
    const STEP_US: u32;

    /// Get time per step in microseconds.
    fn get_step_us(&self) -> u32 {
        Self::STEP_US
    }

    /// Perform a single simulation step.
    fn step(&mut self);

    /// Get the current simulation time in microseconds.
    fn get_time_us(&self) -> u32;

    /// Get the time that the simulation will reach at the next step.
    fn get_time_us_at_next_step(&self) -> u32 {
        self.get_time_us() + Self::STEP_US
    }

    fn get_time_us_at_next_step_after(&self, time_us: u32) -> u32 {
        let stray_time = time_us % Self::STEP_US;
        if stray_time == 0 {
            time_us
        } else {
            time_us + Self::STEP_US - stray_time
        }
    }

    /// Perform steps to reach the required time
    fn step_until_time_us(&mut self, target_time_us: u32) {
        while self.get_time_us_at_next_step() <= target_time_us {
            self.step();
        }
    }

    /// Get the time that the simulation will reach after the given number of steps.
    fn get_time_after_steps_us(&self, steps: usize) -> u32 {
        self.get_time_us() + (steps as u32 * Self::STEP_US)
    }

    /// Get the simulated steps count.
    fn get_step_count(&self) -> usize;

    /// Get the current state of the left line sensors.
    fn get_line_sensors_left(&self) -> [f32; 8];
    /// Get the current state of the right line sensors.
    fn get_line_sensors_right(&self) -> [f32; 8];
    /// Get the current motor angles.
    fn get_motor_angles(&self) -> MotorAngles;
    /// Get the current accelerometer data.
    fn get_accel(&self) -> AccelData;
    /// Get the current gyroscope data.
    fn get_gyro(&self) -> GyroData;
    /// Get the current IMU fused data.
    fn get_imu_fused_data(&self) -> ImuFusedData;

    /// Set motor drivers duty cycles.
    fn set_motor_drivers_duty_cycles(&mut self, duty_cycles: MotorDriversDutyCycles);

    /// Get the collected execution data.
    fn get_data(&self) -> ExecutionData;

    fn is_active(&self) -> bool;
}
