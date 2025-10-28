use bevy::{
    ecs::{component::Component, resource::Resource},
    math::Vec3,
    transform::components::Transform,
};

#[derive(Clone, Component)]
pub struct BodyExecutionData {
    pub period: u32,
    pub steps: Vec<Transform>,
}

impl BodyExecutionData {
    pub fn empty(period: u32) -> Self {
        Self {
            period,
            steps: Vec::new(),
        }
    }

    pub fn at_time_secs(&self, time_secs: f32) -> Transform {
        if self.steps.is_empty() {
            Transform::default()
        } else {
            let index = ((time_secs * 1_000_000.0 / (self.period as f32))
                .floor()
                .max(0.0) as usize)
                .min(self.steps.len() - 1);
            self.steps[index]
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WheelDataSide {
    Left,
    Right,
}

impl std::fmt::Display for WheelDataSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "L"),
            Self::Right => write!(f, "R"),
        }
    }
}

impl WheelDataSide {
    pub fn axis_rotation(&self) -> Vec3 {
        match self {
            Self::Left => Vec3::NEG_X,
            Self::Right => Vec3::NEG_X,
        }
    }

    pub fn axis_direction(&self) -> Vec3 {
        match self {
            Self::Left => Vec3::NEG_X,
            Self::Right => Vec3::X,
        }
    }
}

#[derive(Clone, Component)]
pub struct WheelExecutionData {
    pub period: u32,
    pub side: WheelDataSide,
    pub steps: Vec<f32>,
}

impl WheelExecutionData {
    pub fn empty(period: u32, side: WheelDataSide) -> Self {
        Self {
            period,
            side,
            steps: Vec::new(),
        }
    }

    pub fn axis_rotation(&self) -> Vec3 {
        self.side.axis_rotation()
    }

    pub fn axis_direction(&self) -> Vec3 {
        self.side.axis_direction()
    }

    pub fn at_time_secs(&self, time_secs: f32) -> f32 {
        if self.steps.is_empty() {
            0.0
        } else {
            let index = ((time_secs * 1_000_000.0 / self.period as f32)
                .floor()
                .max(0.0) as usize)
                .min(self.steps.len() - 1);
            self.steps[index]
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActivityData {
    pub start_time_us: Option<u32>,
    pub out_time_us: Option<u32>,
    pub end_time_us: Option<u32>,
}

#[derive(Clone, Copy)]
pub enum BotStatus {
    Waiting { time_secs: f32 },
    Racing { time_secs: f32 },
    EndedAt { time_secs: f32 },
    OutAt { time_secs: f32 },
}

impl BotStatus {
    pub fn display_time_secs(&self) -> f32 {
        match self {
            BotStatus::Waiting { time_secs } => *time_secs,
            BotStatus::Racing { time_secs } => *time_secs,
            BotStatus::EndedAt { time_secs } => *time_secs,
            BotStatus::OutAt { time_secs } => *time_secs,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BotFinalStatus {
    NotStarted,
    NotEnded,
    EndedAt { time_secs: f32 },
    OutAt { time_secs: f32 },
}

impl BotFinalStatus {
    pub fn end_time(&self) -> Option<f32> {
        match self {
            BotFinalStatus::NotStarted => None,
            BotFinalStatus::NotEnded => None,
            BotFinalStatus::EndedAt { time_secs } => Some(*time_secs),
            BotFinalStatus::OutAt { time_secs } => Some(*time_secs),
        }
    }

    fn kind_rank(&self) -> usize {
        match self {
            BotFinalStatus::NotStarted => 3,
            BotFinalStatus::NotEnded => 2,
            BotFinalStatus::EndedAt { .. } => 0,
            BotFinalStatus::OutAt { .. } => 1,
        }
    }

    fn kind_value(&self) -> f32 {
        match self {
            BotFinalStatus::NotStarted => 0.0,
            BotFinalStatus::NotEnded => 0.0,
            BotFinalStatus::EndedAt { time_secs } => *time_secs,
            BotFinalStatus::OutAt { time_secs } => *time_secs,
        }
    }
}

impl std::cmp::Eq for BotFinalStatus {}

impl std::cmp::PartialOrd for BotFinalStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.kind_rank().cmp(&other.kind_rank()) {
            std::cmp::Ordering::Equal => self.kind_value().partial_cmp(&other.kind_value()),
            ord => Some(ord),
        }
    }
}

impl std::cmp::Ord for BotFinalStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind_rank().cmp(&other.kind_rank()) {
            std::cmp::Ordering::Equal => {
                let self_value = self.kind_value();
                let other_value = other.kind_value();
                self_value.total_cmp(&other_value)
            }
            ord => ord,
        }
    }
}

impl ActivityData {
    pub fn empty() -> Self {
        Self {
            start_time_us: None,
            out_time_us: None,
            end_time_us: None,
        }
    }

    pub fn is_active_now(&self) -> bool {
        self.start_time_us.is_none() && self.out_time_us.is_none() && self.end_time_us.is_none()
    }

    pub fn status_at_time(&self, time_secs: f32) -> BotStatus {
        let time_us: u32 = (time_secs * 1_000_000.0) as u32;

        let start_secs = match self.start_time_us {
            Some(start) => {
                if time_us < start {
                    return BotStatus::Waiting { time_secs };
                } else {
                    start as f32 / 1_000_000.0
                }
            }
            None => {
                return BotStatus::Waiting { time_secs };
            }
        };

        if let Some(end_us) = self.end_time_us {
            if time_us > end_us {
                let end_secs = end_us as f32 / 1_000_000.0;
                return BotStatus::EndedAt {
                    time_secs: end_secs - start_secs,
                };
            }
        }

        if let Some(out_us) = self.out_time_us {
            if time_us > out_us {
                let out_secs = out_us as f32 / 1_000_000.0;
                return BotStatus::OutAt {
                    time_secs: out_secs - start_secs,
                };
            }
        }

        BotStatus::Racing {
            time_secs: time_secs - start_secs,
        }
    }

    pub fn final_status(&self) -> BotFinalStatus {
        let start_us = match self.start_time_us {
            Some(start_us) => start_us,
            None => return BotFinalStatus::NotStarted,
        };

        if let Some(ended_us) = self.end_time_us {
            let racing_us = ended_us - start_us;
            return BotFinalStatus::EndedAt {
                time_secs: racing_us as f32 / 1_000_000.0,
            };
        }

        if let Some(out_us) = self.out_time_us {
            let racing_us = out_us - start_us;
            return BotFinalStatus::OutAt {
                time_secs: racing_us as f32 / 1_000_000.0,
            };
        }

        BotFinalStatus::NotEnded
    }
}

#[derive(Clone, Resource)]
pub struct ExecutionData {
    pub body_data: BodyExecutionData,
    pub left_wheel_data: WheelExecutionData,
    pub right_wheel_data: WheelExecutionData,
    pub activity_data: ActivityData,
}

impl ExecutionData {
    pub fn empty(period: u32) -> Self {
        Self {
            body_data: BodyExecutionData::empty(period),
            left_wheel_data: WheelExecutionData::empty(period, WheelDataSide::Left),
            right_wheel_data: WheelExecutionData::empty(period, WheelDataSide::Right),
            activity_data: ActivityData::empty(),
        }
    }
}

pub const PWM_MAX: i16 = 1000;
pub const PWM_MIN: i16 = -1000;

/// Motor drivers duty cycles.
#[derive(Clone, Copy, Resource, Default)]
pub struct MotorDriversDutyCycles {
    pub left: i16,
    pub right: i16,
}

/// Motor angles in radians.
#[derive(Clone, Copy, Resource, Default)]
pub struct MotorAngles {
    pub left: f32,
    pub right: f32,
}

/// Gyroscope data in rad/s.
#[derive(Clone, Copy, Default)]
pub struct GyroData {
    pub roll_angular_speed: f32,
    pub pitch_angular_speed: f32,
    pub yaw_angular_speed: f32,
}

impl From<Vec3> for GyroData {
    fn from(value: Vec3) -> Self {
        Self {
            roll_angular_speed: value.y,
            pitch_angular_speed: value.x,
            yaw_angular_speed: value.z,
        }
    }
}

/// Fused IMU data in radians.
#[derive(Clone, Copy, Default)]
pub struct ImuFusedData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl From<Vec3> for ImuFusedData {
    fn from(value: Vec3) -> Self {
        Self {
            roll: value.y,
            pitch: value.x,
            yaw: value.z,
        }
    }
}

/// Bot logical positions
#[derive(Debug, Clone, Copy, Default)]
pub enum BotPosition {
    #[default]
    OnTrack,
    Out,
    End,
}

/// Wrapper for all sensors data.
#[derive(Clone, Copy, Resource, Default)]
pub struct SensorsData {
    pub motor_angles: MotorAngles,
    pub gyro: GyroData,
    pub imu_fused: ImuFusedData,
    pub line_sensors: [f32; 16],
    pub bot_position: BotPosition,
    pub is_out_of_track: bool,
    pub is_over_track_end: bool,
}

pub trait SimulationStepper {
    /// Get time per step in microseconds.
    fn step_us(&self) -> u32;

    /// Perform a single simulation step.
    fn step(&mut self);

    /// Get the current simulation time in microseconds.
    fn get_time_us(&self) -> u32;

    /// Get the time that the simulation will reach at the next step.
    fn get_time_us_at_next_step(&self) -> u32 {
        self.get_time_us() + self.step_us()
    }

    fn get_time_us_at_next_step_after(&self, time_us: u32) -> u32 {
        let stray_time = time_us % self.step_us();
        if stray_time == 0 {
            time_us
        } else {
            time_us + self.step_us() - stray_time
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
        self.get_time_us() + (steps as u32 * self.step_us())
    }

    /// Get the simulated steps count.
    fn get_step_count(&self) -> usize;

    /// Get the current state of the left line sensors.
    fn get_line_sensors_left(&self) -> [f32; 8];
    /// Get the current state of the right line sensors.
    fn get_line_sensors_right(&self) -> [f32; 8];
    /// Get the current motor angles.
    fn get_motor_angles(&self) -> MotorAngles;
    /// Get the current gyroscope data.
    fn get_gyro(&self) -> GyroData;
    /// Get the current IMU fused data.
    fn get_imu_fused_data(&self) -> ImuFusedData;

    /// Set motor drivers duty cycles.
    fn set_motor_drivers_duty_cycles(&mut self, duty_cycles: MotorDriversDutyCycles);

    /// Get the collected execution data.
    fn get_data(&mut self) -> ExecutionData;

    fn is_active(&self) -> bool;
}
