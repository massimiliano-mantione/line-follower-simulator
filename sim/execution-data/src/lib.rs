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
        Some(self.cmp(other))
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
    pub fn empty(force_initially_started: bool) -> Self {
        Self {
            start_time_us: if force_initially_started {
                Some(0)
            } else {
                None
            },
            out_time_us: None,
            end_time_us: None,
        }
    }

    pub fn is_active_now(&self) -> bool {
        self.start_time_us.is_some() && self.out_time_us.is_none() && self.end_time_us.is_none()
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
                    time_secs: (end_secs - start_secs).max(0.0),
                };
            }
        }

        if let Some(out_us) = self.out_time_us {
            if time_us > out_us {
                let out_secs = out_us as f32 / 1_000_000.0;
                return BotStatus::OutAt {
                    time_secs: (out_secs - start_secs).max(0.0),
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
            let racing_us = if ended_us > start_us {
                ended_us - start_us
            } else {
                0
            };
            return BotFinalStatus::EndedAt {
                time_secs: racing_us as f32 / 1_000_000.0,
            };
        }

        if let Some(out_us) = self.out_time_us {
            let racing_us = if out_us > start_us {
                out_us - start_us
            } else {
                0
            };
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
    pub fn empty(period: u32, force_initially_started: bool) -> Self {
        Self {
            body_data: BodyExecutionData::empty(period),
            left_wheel_data: WheelExecutionData::empty(period, WheelDataSide::Left),
            right_wheel_data: WheelExecutionData::empty(period, WheelDataSide::Right),
            activity_data: ActivityData::empty(force_initially_started),
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

// EXERCISE 12.3: the data types for the two new sensors.
//
// A gyroscope reports angular *velocity* in rad/s; a fused IMU reports absolute
// *angles* in radians. Both have roll, pitch and yaw.
//
// Give each one a `From<Vec3>` conversion, and note that the axis mapping is a
// *choice* tied to the Z-up convention in this codebase: roll comes from y, pitch
// from x, yaw from z. That is exactly the sort of thing that has to be written down
// somewhere, or it gets rediscovered painfully.
//
// Then add a field per sensor to `SensorsData` below.

/// Bot logical positions
#[derive(Debug, Clone, Copy, Default)]
pub enum BotPosition {
    #[default]
    OnTrack,
    Out,
    End,
}

/// Bot physical positions
#[derive(Debug, Clone, Copy, Default)]
pub struct BotPhysicalPosition {
    pub pos: Vec3,
    pub rot: Vec3,
}

impl std::fmt::Display for BotPhysicalPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pos [{} {} {}] rot [{} {} {}]",
            (self.pos.x * 1000.0) as i32,
            (self.pos.y * 1000.0) as i32,
            (self.pos.z * 1000.0) as i32,
            self.rot.x.to_degrees() as i32,
            self.rot.y.to_degrees() as i32,
            self.rot.z.to_degrees() as i32,
        )
    }
}

/// Wrapper for all sensors data.
#[derive(Clone, Copy, Resource, Default)]
pub struct SensorsData {
    pub motor_angles: MotorAngles,
    // EXERCISE 12.3 (continued): one field per new sensor.
    pub line_sensors: [f32; 16],
    pub bot_position: BotPosition,
    pub bot_physical_position: BotPhysicalPosition,
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
    // EXERCISE 12.6: one accessor per new sensor.
    //
    // This is the architectural seam. Adding a method here means every implementor
    // must be updated - `MockStepper` and `RunnerStepper`. That sounds like a cost
    // and is actually the seam doing its job: it is what lets the whole clock and
    // futures machinery be tested with no Bevy at all.

    /// Get absolute bot position
    fn get_absolute_bot_position(&self) -> BotPhysicalPosition;

    /// Set motor drivers duty cycles.
    fn set_motor_drivers_duty_cycles(&mut self, duty_cycles: MotorDriversDutyCycles);

    /// Get the collected execution data.
    fn get_data(&mut self) -> ExecutionData;

    fn is_active(&self) -> bool;
}
