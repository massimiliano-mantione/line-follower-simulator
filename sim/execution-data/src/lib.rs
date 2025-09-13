use rapier3d::math;

#[derive(Clone, Copy)]
pub struct ExecutionStep {
    pub time_s: f32,
    pub body_rotation: math::Rotation<f32>,
    pub body_translation: math::Vector<f32>,
    pub left_wheel_rotation: math::Rotation<f32>,
    pub left_wheel_translation: math::Vector<f32>,
    pub right_wheel_rotation: math::Rotation<f32>,
    pub right_wheel_translation: math::Vector<f32>,
}

pub struct ExecutionData {
    pub steps: Vec<ExecutionStep>,
}

/// Motor angles in radians.
#[derive(Clone, Copy)]
pub struct MotorAngles {
    pub left: f32,
    pub right: f32,
}

/// Accelerometer data in m/s².
#[derive(Clone, Copy)]
pub struct AccelData {
    pub front: f32,
    pub side: f32,
    pub vertical: f32,
}

/// Gyroscope data in rad/s.
#[derive(Clone, Copy)]
pub struct GyroData {
    pub roll_angular_speed: f32,
    pub pitch_angular_speed: f32,
    pub yaw_angular_speed: f32,
}

/// Fused IMU data in radians.
#[derive(Clone, Copy)]
pub struct ImuFusedData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

pub trait SimulationStepper {
    /// Time per step in microseconds.
    const STEP_US: u32;

    /// Perform a single simulation step.
    fn step(&mut self);

    /// Get the current simulation time in microseconds.
    fn get_time_us(&self) -> u32;

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

    /// Get the collected execution data.
    fn get_data(&self) -> ExecutionData;
}
