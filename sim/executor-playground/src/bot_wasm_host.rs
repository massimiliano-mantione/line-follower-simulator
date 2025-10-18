use std::{
    collections::{BTreeMap, BTreeSet},
    f32::consts::PI,
    path::PathBuf,
    u16,
};

use execution_data::{
    AccelData, GyroData, ImuFusedData, MotorAngles, MotorDriversDutyCycles, SimulationStepper,
};

use crate::bindings::{
    self,
    devices::{
        DeviceOperation, DeviceValue, FutureHandle, MotorPower, PollError, PollOperationStatus,
        TimeUs,
    },
    diagnostics::CsvColumn,
};

pub trait DeviceValueExt {
    fn get_u8(&self, index: usize) -> u8;
    fn get_u16(&self, index: usize) -> u16;
    fn get_i16(&self, index: usize) -> i16;
    fn get_u32(&self, index: usize) -> u32;
    fn get_bool(&self, index: usize) -> bool;

    fn set_u8(self, value: u8, index: usize) -> Self;
    fn set_u16(self, value: u16, index: usize) -> Self;
    fn set_i16(self, value: i16, index: usize) -> Self;
    fn set_u32(self, value: u32, index: usize) -> Self;
    fn set_bool(self, value: bool, index: usize) -> Self;
}

pub const DEVICE_VALUE_ZERO: DeviceValue = DeviceValue {
    v0: 0,
    v1: 0,
    v2: 0,
    v3: 0,
    v4: 0,
    v5: 0,
    v6: 0,
    v7: 0,
};

fn build_u16_from_u8s(v0: u8, v1: u8) -> u16 {
    ((v1 as u16) << 8) | (v0 as u16)
}
fn build_i16_from_u8s(v0: u8, v1: u8) -> i16 {
    build_u16_from_u8s(v0, v1) as i16
}
fn build_u32_from_u8s(v0: u8, v1: u8, v2: u8, v3: u8) -> u32 {
    ((v3 as u32) << 24) | ((v2 as u32) << 16) | ((v1 as u32) << 8) | (v0 as u32)
}

fn decompose_u16_to_u8s(value: u16) -> (u8, u8) {
    let v0 = (value & 0x00FF) as u8;
    let v1 = ((value & 0xFF00) >> 8) as u8;
    (v0, v1)
}
fn decompose_i16_to_u8s(value: i16) -> (u8, u8) {
    decompose_u16_to_u8s(value as u16)
}
fn decompose_u32_to_u8s(value: u32) -> (u8, u8, u8, u8) {
    let v0 = (value & 0x000000FF) as u8;
    let v1 = ((value & 0x0000FF00) >> 8) as u8;
    let v2 = ((value & 0x00FF0000) >> 16) as u8;
    let v3 = ((value & 0xFF000000) >> 24) as u8;
    (v0, v1, v2, v3)
}

impl DeviceValueExt for DeviceValue {
    fn get_u8(&self, index: usize) -> u8 {
        match index {
            0 => self.v0,
            1 => self.v1,
            2 => self.v2,
            3 => self.v3,
            4 => self.v4,
            5 => self.v5,
            6 => self.v6,
            7 => self.v7,
            _ => 0,
        }
    }

    fn get_u16(&self, index: usize) -> u16 {
        match index {
            0 => build_u16_from_u8s(self.v0, self.v1),
            1 => build_u16_from_u8s(self.v2, self.v3),
            2 => build_u16_from_u8s(self.v4, self.v5),
            3 => build_u16_from_u8s(self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_i16(&self, index: usize) -> i16 {
        match index {
            0 => build_i16_from_u8s(self.v0, self.v1),
            1 => build_i16_from_u8s(self.v2, self.v3),
            2 => build_i16_from_u8s(self.v4, self.v5),
            3 => build_i16_from_u8s(self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_u32(&self, index: usize) -> u32 {
        match index {
            0 => build_u32_from_u8s(self.v0, self.v1, self.v2, self.v3),
            1 => build_u32_from_u8s(self.v4, self.v5, self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_bool(&self, index: usize) -> bool {
        match index {
            0 => {
                if self.v0 == 0 {
                    false
                } else {
                    true
                }
            }
            1 => {
                if self.v1 == 0 {
                    false
                } else {
                    true
                }
            }
            2 => {
                if self.v2 == 0 {
                    false
                } else {
                    true
                }
            }
            3 => {
                if self.v3 == 0 {
                    false
                } else {
                    true
                }
            }
            4 => {
                if self.v4 == 0 {
                    false
                } else {
                    true
                }
            }
            5 => {
                if self.v5 == 0 {
                    false
                } else {
                    true
                }
            }
            6 => {
                if self.v6 == 0 {
                    false
                } else {
                    true
                }
            }
            7 => {
                if self.v7 == 0 {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn set_u8(mut self, value: u8, index: usize) -> Self {
        match index {
            0 => self.v0 = value,
            1 => self.v1 = value,
            2 => self.v2 = value,
            3 => self.v3 = value,
            4 => self.v4 = value,
            5 => self.v5 = value,
            6 => self.v6 = value,
            7 => self.v7 = value,
            _ => (),
        };
        self
    }

    fn set_u16(mut self, value: u16, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
            }
            1 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v2 = v0;
                self.v3 = v1;
            }
            2 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
            }
            3 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v6 = v0;
                self.v7 = v1;
            }
            _ => (),
        };
        self
    }

    fn set_i16(mut self, value: i16, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
            }
            1 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v2 = v0;
                self.v3 = v1;
            }
            2 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
            }
            3 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v6 = v0;
                self.v7 = v1;
            }
            _ => (),
        };
        self
    }

    fn set_u32(mut self, value: u32, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1, v2, v3) = decompose_u32_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
                self.v2 = v2;
                self.v3 = v3;
            }
            1 => {
                let (v0, v1, v2, v3) = decompose_u32_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
                self.v6 = v2;
                self.v7 = v3;
            }
            _ => (),
        };
        self
    }

    fn set_bool(mut self, value: bool, index: usize) -> Self {
        match index {
            0 => self.v0 = if value { 1 } else { 0 },
            1 => self.v1 = if value { 1 } else { 0 },
            2 => self.v2 = if value { 1 } else { 0 },
            3 => self.v3 = if value { 1 } else { 0 },
            4 => self.v4 = if value { 1 } else { 0 },
            5 => self.v5 = if value { 1 } else { 0 },
            6 => self.v6 = if value { 1 } else { 0 },
            7 => self.v7 = if value { 1 } else { 0 },
            _ => (),
        };
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FutureOperation {
    ReadLineLeft,
    ReadLineRight,
    ReadMotorAngles,
    ReadAccel,
    ReadGyro,
    ReadImuFusedData,
    GetTime,
    Sleep,
    GetEnabled,
    WaitEnabled,
    WaitDisabled,
}

impl From<DeviceOperation> for FutureOperation {
    fn from(value: DeviceOperation) -> Self {
        match value {
            DeviceOperation::ReadLineLeft => FutureOperation::ReadLineLeft,
            DeviceOperation::ReadLineRight => FutureOperation::ReadLineRight,
            DeviceOperation::ReadMotorAngles => FutureOperation::ReadMotorAngles,
            DeviceOperation::ReadAccel => FutureOperation::ReadAccel,
            DeviceOperation::ReadGyro => FutureOperation::ReadGyro,
            DeviceOperation::ReadImuFusedData => FutureOperation::ReadImuFusedData,
            DeviceOperation::GetTime => FutureOperation::GetTime,
            DeviceOperation::SleepFor(_) => FutureOperation::Sleep,
            DeviceOperation::SleepUntil(_) => FutureOperation::Sleep,
            DeviceOperation::GetEnabled => FutureOperation::GetEnabled,
            DeviceOperation::WaitEnabled => FutureOperation::WaitEnabled,
            DeviceOperation::WaitDisabled => FutureOperation::WaitDisabled,
        }
    }
}

impl FutureOperation {
    pub fn compute_value(
        &self,
        stepper: &impl SimulationStepper,
        current_time: TimeUs,
    ) -> DeviceValueRaw {
        match self {
            FutureOperation::ReadLineLeft => {
                DeviceValueRaw::from_sensor_values(stepper.get_line_sensors_left())
            }
            FutureOperation::ReadLineRight => {
                DeviceValueRaw::from_sensor_values(stepper.get_line_sensors_right())
            }
            FutureOperation::ReadMotorAngles => {
                DeviceValueRaw::from_motor_angles(stepper.get_motor_angles())
            }
            FutureOperation::ReadAccel => DeviceValueRaw::from_accel_data(stepper.get_accel()),
            FutureOperation::ReadGyro => DeviceValueRaw::from_gyro_data(stepper.get_gyro()),
            FutureOperation::ReadImuFusedData => {
                DeviceValueRaw::from_imu_fused_data(stepper.get_imu_fused_data())
            }
            FutureOperation::GetTime => DeviceValueRaw::zero().with_u32(0, current_time),
            FutureOperation::Sleep => DeviceValueRaw::zero(),
            FutureOperation::GetEnabled => {
                DeviceValueRaw::zero().with_u8(0, if stepper.is_active() { 1 } else { 0 })
            }
            FutureOperation::WaitEnabled => DeviceValueRaw::zero(),
            FutureOperation::WaitDisabled => DeviceValueRaw::zero(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeviceValueRaw {
    pub v0: u8,
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
    pub v4: u8,
    pub v5: u8,
    pub v6: u8,
    pub v7: u8,
}

impl Into<DeviceValue> for DeviceValueRaw {
    fn into(self) -> DeviceValue {
        DeviceValue {
            v0: self.v0,
            v1: self.v1,
            v2: self.v2,
            v3: self.v3,
            v4: self.v4,
            v5: self.v5,
            v6: self.v6,
            v7: self.v7,
        }
    }
}

pub const DEVICE_VALUE_RAW_ZERO: DeviceValueRaw = DeviceValueRaw {
    v0: 0,
    v1: 0,
    v2: 0,
    v3: 0,
    v4: 0,
    v5: 0,
    v6: 0,
    v7: 0,
};

impl DeviceValueRaw {
    pub const fn zero() -> Self {
        DEVICE_VALUE_RAW_ZERO
    }

    pub fn with_u8(self, index: usize, value: u8) -> Self {
        match index {
            0 => Self { v0: value, ..self },
            1 => Self { v1: value, ..self },
            2 => Self { v2: value, ..self },
            3 => Self { v3: value, ..self },
            4 => Self { v4: value, ..self },
            5 => Self { v5: value, ..self },
            6 => Self { v6: value, ..self },
            7 => Self { v7: value, ..self },
            _ => self,
        }
    }

    pub fn with_u16(self, index: usize, value: u16) -> Self {
        match index {
            0 => Self {
                v0: value as u8,
                v1: (value >> 8) as u8,
                ..self
            },
            1 => Self {
                v2: value as u8,
                v3: (value >> 8) as u8,
                ..self
            },
            2 => Self {
                v4: value as u8,
                v5: (value >> 8) as u8,
                ..self
            },
            3 => Self {
                v6: value as u8,
                v7: (value >> 8) as u8,
                ..self
            },
            _ => self,
        }
    }

    pub fn with_i16(self, index: usize, value: i16) -> Self {
        self.with_u16(index, value as u16)
    }

    pub fn with_u32(self, index: usize, value: u32) -> Self {
        match index {
            0 => Self {
                v0: value as u8,
                v1: (value >> 8) as u8,
                v2: (value >> 16) as u8,
                v3: (value >> 24) as u8,
                ..self
            },
            1 => Self {
                v4: value as u8,
                v5: (value >> 8) as u8,
                v6: (value >> 16) as u8,
                v7: (value >> 24) as u8,
                ..self
            },
            _ => self,
        }
    }

    pub fn from_sensor_values(sensor_values: [f32; 8]) -> Self {
        sensor_values
            .iter()
            .enumerate()
            .fold(Self::zero(), |v, (i, s)| v.with_u8(i, (s * 255.0) as u8))
    }

    pub fn from_motor_angles(angles: MotorAngles) -> Self {
        Self::zero()
            .with_u16(0, (angles.left * (u16::MAX as f32) / (PI * 2.0)) as u16)
            .with_u16(1, (angles.right * (u16::MAX as f32) / (PI * 2.0)) as u16)
    }

    pub fn from_accel_data(accel_data: AccelData) -> Self {
        Self::zero()
            .with_u16(0, accel_data.front as u16)
            .with_u16(1, accel_data.side as u16)
            .with_u16(2, accel_data.vertical as u16)
    }

    pub fn from_gyro_data(gyro_data: GyroData) -> Self {
        Self::zero()
            .with_i16(0, gyro_data.roll_angular_speed as i16)
            .with_i16(1, gyro_data.pitch_angular_speed as i16)
            .with_i16(2, gyro_data.yaw_angular_speed as i16)
    }

    pub fn from_imu_fused_data(imu_data: ImuFusedData) -> Self {
        Self::zero()
            .with_i16(0, imu_data.roll.to_radians() as i16)
            .with_i16(1, imu_data.pitch.to_radians() as i16)
            .with_i16(2, imu_data.yaw.to_radians() as i16)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FutureReadyCondition {
    ReadyAt(TimeUs),
    IsActive,
    IsInactive,
}

impl FutureReadyCondition {
    pub fn is_time_based(&self) -> bool {
        match self {
            FutureReadyCondition::ReadyAt(_) => true,
            FutureReadyCondition::IsActive => false,
            FutureReadyCondition::IsInactive => false,
        }
    }

    pub fn is_activity(&self) -> bool {
        match self {
            FutureReadyCondition::ReadyAt(_) => false,
            FutureReadyCondition::IsActive => true,
            FutureReadyCondition::IsInactive => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FutureValueStatus {
    Pending,
    Ready(DeviceValueRaw),
    Consumed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FutureValueRequest {
    pub ready_condition: FutureReadyCondition,
    pub id: u32,
    pub operation: FutureOperation,
    pub value: FutureValueStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FutureValueReadyTime {
    pub ready_at: TimeUs,
    pub id: u32,
}

impl PartialOrd for FutureValueReadyTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ready_at.partial_cmp(&other.ready_at) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for FutureValueReadyTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.ready_at.cmp(&other.ready_at) {
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
        }
        self.id.cmp(&other.id)
    }
}

pub trait DeviceOperationExt {
    fn ready_condition(
        &self,
        current_time: TimeUs,
        stepper: &impl SimulationStepper,
    ) -> FutureReadyCondition;
    fn duration(&self, current_time: TimeUs) -> TimeUs;
}

impl DeviceOperationExt for DeviceOperation {
    fn ready_condition(
        &self,
        current_time: TimeUs,
        stepper: &impl SimulationStepper,
    ) -> FutureReadyCondition {
        match *self {
            DeviceOperation::ReadLineLeft
            | DeviceOperation::ReadLineRight
            | DeviceOperation::ReadMotorAngles
            | DeviceOperation::ReadAccel
            | DeviceOperation::ReadGyro
            | DeviceOperation::ReadImuFusedData => {
                let step_time = stepper.get_step_us();
                let stray_time = current_time % step_time;
                let trigger_time = if stray_time == 0 {
                    current_time
                } else {
                    current_time + step_time - stray_time
                };
                FutureReadyCondition::ReadyAt(trigger_time)
            }
            DeviceOperation::SleepFor(duration) => {
                FutureReadyCondition::ReadyAt(current_time + duration)
            }
            DeviceOperation::SleepUntil(deadline) => {
                FutureReadyCondition::ReadyAt(deadline.max(current_time))
            }
            DeviceOperation::GetTime | DeviceOperation::GetEnabled => {
                FutureReadyCondition::ReadyAt(current_time)
            }
            DeviceOperation::WaitEnabled => FutureReadyCondition::IsActive,
            DeviceOperation::WaitDisabled => FutureReadyCondition::IsInactive,
        }
    }

    fn duration(&self, current_time: TimeUs) -> TimeUs {
        match *self {
            DeviceOperation::ReadLineLeft
            | DeviceOperation::ReadLineRight
            | DeviceOperation::ReadMotorAngles
            | DeviceOperation::ReadAccel
            | DeviceOperation::ReadGyro
            | DeviceOperation::ReadImuFusedData
            | DeviceOperation::GetTime
            | DeviceOperation::GetEnabled
            | DeviceOperation::WaitEnabled
            | DeviceOperation::WaitDisabled => 0,
            DeviceOperation::SleepFor(duration) => duration,
            DeviceOperation::SleepUntil(deadline) => {
                if deadline > current_time {
                    deadline - current_time
                } else {
                    0
                }
            }
        }
    }
}

// A CPU clock of 20 MHz means one instruction takes 50ns,
// and one fuel unit symbolizes one instruction.
const FUEL_UNIT_NS: u64 = 50;

pub fn fuel_for_time_us(time_us: TimeUs) -> u64 {
    time_us as u64 * 1000 / FUEL_UNIT_NS
}

pub fn time_us_for_fuel(fuel: u64) -> TimeUs {
    ((fuel * FUEL_UNIT_NS) / 1000) as TimeUs
}

pub struct BotHost<S: SimulationStepper> {
    stepper: S,
    total_simulation_time: TimeUs,
    current_fuel: u64,
    skipped_fuel: u64,

    workdir_path: Option<PathBuf>,
    output_log: bool,
    log_lines: Vec<String>,

    next_future_handle_id: u32,
    futures_by_id: BTreeMap<u32, FutureValueRequest>,
    futures_by_ready_time: BTreeSet<FutureValueReadyTime>,
    futures_by_activity: BTreeSet<u32>,
}

impl<S: SimulationStepper> bindings::devices::Host for BotHost<S> {
    #[doc = " Perform a blocking operation (returns the provided value, blocking for the needed time)"]
    fn device_operation_blocking(
        &mut self,
        current_fuel: u64,
        operation: DeviceOperation,
    ) -> wasmtime::Result<DeviceValue> {
        let start_time = self.setup_current_time(current_fuel)?;
        match operation.ready_condition(start_time, &self.stepper) {
            FutureReadyCondition::ReadyAt(ready_at) => {
                self.step_until_time(ready_at);
            }
            FutureReadyCondition::IsActive => {
                while !self.stepper.is_active() {
                    self.step();
                }
            }
            FutureReadyCondition::IsInactive => {
                while self.stepper.is_active() {
                    self.step();
                }
            }
        }
        let end_time = self.stepper.get_time_us().max(start_time);

        self.set_current_time(end_time)?;
        let op: FutureOperation = operation.into();
        Ok(op.compute_value(&self.stepper, start_time).into())
    }

    #[doc = " Initiate an async operation (immediately returns a handle to the future value)"]
    fn device_operation_async(
        &mut self,
        current_fuel: u64,
        operation: DeviceOperation,
    ) -> wasmtime::Result<FutureHandle> {
        let current_time = self.setup_current_time(current_fuel)?;
        let id = self.next_future_handle_id;
        self.next_future_handle_id += 1;

        let future_value = FutureValueRequest {
            ready_condition: operation.ready_condition(current_time, &self.stepper),
            id,
            operation: operation.into(),
            value: FutureValueStatus::Pending,
        };

        let ready_at = match future_value.ready_condition {
            FutureReadyCondition::ReadyAt(ready_at) => {
                self.futures_by_ready_time
                    .insert(FutureValueReadyTime { ready_at, id });
                ready_at
            }
            FutureReadyCondition::IsActive | FutureReadyCondition::IsInactive => {
                self.futures_by_activity.insert(id);
                current_time
            }
        };
        self.futures_by_id.insert(id, future_value);

        Ok(FutureHandle { id, ready_at })
    }

    #[doc = " Poll the status of an async operation (returns immediately)"]
    fn device_poll(
        &mut self,
        current_fuel: u64,
        handle: FutureHandle,
    ) -> wasmtime::Result<Result<PollOperationStatus, PollError>> {
        let current_time = self.setup_current_time(current_fuel)?;
        self.step_until_time(current_time);
        let r = match self.futures_by_id.get_mut(&handle.id) {
            Some(f) => match f.value {
                FutureValueStatus::Pending => Ok(PollOperationStatus::Pending),
                FutureValueStatus::Ready(device_value_raw) => {
                    f.value = FutureValueStatus::Consumed;
                    Ok(PollOperationStatus::Ready(device_value_raw.into()))
                }
                FutureValueStatus::Consumed => Err(PollError::ConsumedHandle),
            },
            None => Err(PollError::InvalidHandle),
        };
        Ok(r)
    }

    #[doc = " Advance one step in the physical simulation"]
    fn world_step(&mut self, current_fuel: u64) -> wasmtime::Result<()> {
        let current_time = self.setup_current_time(current_fuel)?;
        self.step();
        let end_time = self.stepper.get_time_us().max(current_time);
        self.set_current_time(end_time)?;
        Ok(())
    }

    #[doc = " Instructs the simulation to forget the handle to an async operation"]
    #[doc = " (is equivalent to dropping the future in Rust)"]
    fn forget_handle(&mut self, handle: FutureHandle) -> () {
        self.futures_by_activity.remove(&handle.id);
        self.futures_by_ready_time.remove(&FutureValueReadyTime {
            ready_at: handle.ready_at,
            id: handle.id,
        });
        self.futures_by_id.remove(&handle.id);
    }

    #[doc = " Set the power of both motors"]
    fn set_motors_power(
        &mut self,
        current_fuel: u64,
        left: MotorPower,
        right: MotorPower,
    ) -> wasmtime::Result<()> {
        let current_time = self.setup_current_time(current_fuel)?;
        self.step_until_time(current_time);
        self.stepper
            .set_motor_drivers_duty_cycles(MotorDriversDutyCycles { left, right });
        Ok(())
    }
}

impl<S: SimulationStepper> bindings::diagnostics::Host for BotHost<S> {
    #[doc = " Write a line of text as a log, like writing to a serial line"]
    #[doc = " (each character takes 100 microseconds)"]
    fn write_line(
        &mut self,
        current_fuel: u64,
        text: wasmtime::component::__internal::String,
    ) -> wasmtime::Result<()> {
        let current_time = self.setup_current_time(current_fuel)?;
        let char_count = text.as_bytes().len();
        self.skip_time((char_count * 100) as u32)?;

        if self.output_log || self.workdir_path.is_some() {
            let sec = current_time / 1_000_000;
            let ms = (current_time / 1_000) % 1000;
            let us = current_time % 1000;
            let line = format!("{:02}.{:03}_{:03}: {}", sec, ms, us, text);

            if self.output_log {
                println!("{}", &line);
            }
            if self.workdir_path.is_some() {
                self.log_lines.push(line);
            }
        }

        Ok(())
    }

    #[doc = " Write a buffer into a file, eventually converting it to CSV"]
    #[doc = " (each byte takes 10 microseconds)"]
    fn write_file(
        &mut self,
        current_fuel: u64,
        name: wasmtime::component::__internal::String,
        data: wasmtime::component::__internal::Vec<u8>,
        csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
    ) -> wasmtime::Result<()> {
        self.setup_current_time(current_fuel)?;
        self.skip_time((data.len() * 10) as u32)?;

        if let Some(path) = self.workdir_path.as_ref() {
            let bin_name = format!("{}.bin", name);
            let bin_path = path.join(&bin_name);
            if let Err(err) = std::fs::write(&bin_path, &data) {
                eprintln!("Error writing file {}: {}", bin_path.display(), err);
            }
            if let Some(csv) = csv {
                let csv_name = format!("{}.csv", name);
                let csv_path = path.join(&csv_name);
                let handler = CvsLineHandler::new(&csv);
                let text = handler.build_text(&data);
                if let Err(err) = std::fs::write(&csv_path, &text) {
                    eprintln!("Error writing file {}: {}", csv_path.display(), err);
                }
            }
        }

        Ok(())
    }
}

enum CsvColumnKind {
    Int8,
    Int16,
    Int32,
    Uint8,
    Uint16,
    Uint32,
    NamedUint8(BTreeMap<u8, String>),
    IgnoreU8,
    IgnoreU16,
}

impl CsvColumnKind {
    pub fn size(&self) -> usize {
        match self {
            CsvColumnKind::Int8 => 1,
            CsvColumnKind::Int16 => 2,
            CsvColumnKind::Int32 => 4,
            CsvColumnKind::Uint8 => 1,
            CsvColumnKind::Uint16 => 2,
            CsvColumnKind::Uint32 => 4,
            CsvColumnKind::NamedUint8(_) => 1,
            CsvColumnKind::IgnoreU8 => 1,
            CsvColumnKind::IgnoreU16 => 2,
        }
    }

    pub fn get_value(&self, buf: &[u8], start: usize) -> i64 {
        match self {
            CsvColumnKind::Int8 => buf[start] as i8 as i64,
            CsvColumnKind::Int16 => i16::from_le_bytes([buf[start], buf[start + 1]]) as i64,
            CsvColumnKind::Int32 => {
                i32::from_le_bytes([buf[start], buf[start + 1], buf[start + 2], buf[start + 3]])
                    as i64
            }
            CsvColumnKind::Uint8 => buf[start] as i64,
            CsvColumnKind::Uint16 => u16::from_le_bytes([buf[start], buf[start + 1]]) as i64,
            CsvColumnKind::Uint32 => {
                u32::from_le_bytes([buf[start], buf[start + 1], buf[start + 2], buf[start + 3]])
                    as i64
            }
            CsvColumnKind::NamedUint8(_) => buf[start] as i8 as i64,
            CsvColumnKind::IgnoreU8 => 0,
            CsvColumnKind::IgnoreU16 => 0,
        }
    }

    pub fn get_value_text(&self, buf: &[u8], start: usize) -> String {
        let value = self.get_value(buf, start);
        if let CsvColumnKind::NamedUint8(names) = self {
            match names.get(&(value as u8)) {
                Some(name) => name.clone(),
                None => value.to_string(),
            }
        } else {
            value.to_string()
        }
    }
}

struct CsvColumnHandler {
    start: usize,
    kind: CsvColumnKind,
    name: String,
}

struct CvsLineHandler {
    columns: Vec<CsvColumnHandler>,
    size: usize,
}

impl CvsLineHandler {
    pub fn new(spec: &wasmtime::component::__internal::Vec<CsvColumn>) -> Self {
        let mut columns = Vec::new();
        let mut size = 0;

        for column in spec {
            let handler = CsvColumnHandler {
                start: size,
                kind: match &column.kind {
                    bindings::diagnostics::ValueKind::Int8 => CsvColumnKind::Int8,
                    bindings::diagnostics::ValueKind::Int16 => CsvColumnKind::Int16,
                    bindings::diagnostics::ValueKind::Int32 => CsvColumnKind::Int32,
                    bindings::diagnostics::ValueKind::Uint8 => CsvColumnKind::Uint8,
                    bindings::diagnostics::ValueKind::Uint16 => CsvColumnKind::Uint16,
                    bindings::diagnostics::ValueKind::Uint32 => CsvColumnKind::Uint32,
                    bindings::diagnostics::ValueKind::Named(named_values) => {
                        CsvColumnKind::NamedUint8(named_values.iter().fold(
                            BTreeMap::new(),
                            |mut names, named_value| {
                                names.insert(named_value.value as u8, named_value.name.clone());
                                names
                            },
                        ))
                    }
                    bindings::diagnostics::ValueKind::Pad8 => CsvColumnKind::IgnoreU8,
                    bindings::diagnostics::ValueKind::Pad16 => CsvColumnKind::IgnoreU16,
                },
                name: column.name.clone(),
            };
            size += handler.kind.size();
            columns.push(handler);
        }

        Self { columns, size }
    }

    pub fn build_header(&self) -> String {
        let mut line = String::new();
        let mut needs_separator = false;
        for column in &self.columns {
            if needs_separator {
                line.push_str(&",");
            } else {
                needs_separator = true;
            }
            line.push_str(&column.name);
        }
        line.push_str(&"\n");
        line
    }

    pub fn build_line(&self, buf: &[u8]) -> String {
        let mut line = String::new();
        let mut needs_separator = false;
        for column in &self.columns {
            if needs_separator {
                line.push_str(&",");
            } else {
                needs_separator = true;
            }
            line.push_str(&column.kind.get_value_text(buf, column.start));
        }
        line.push_str(&"\n");
        line
    }

    pub fn build_text(&self, buf: &[u8]) -> String {
        let mut text = String::new();
        text.push_str(&self.build_header());
        for chunk in buf.chunks_exact(self.size) {
            text.push_str(&self.build_line(chunk));
        }
        text
    }
}

impl<S: SimulationStepper> BotHost<S> {
    pub fn new(
        stepper: S,
        total_simulation_time: TimeUs,
        workdir_path: Option<PathBuf>,
        output_log: bool,
    ) -> Self {
        Self {
            total_simulation_time,
            current_fuel: fuel_for_time_us(total_simulation_time),
            skipped_fuel: 0,
            stepper,
            workdir_path,
            output_log,
            log_lines: Vec::new(),
            next_future_handle_id: 1,
            futures_by_id: BTreeMap::new(),
            futures_by_ready_time: BTreeSet::new(),
            futures_by_activity: BTreeSet::new(),
        }
    }

    fn check_fuel(&self) -> wasmtime::Result<()> {
        if self.current_fuel <= self.skipped_fuel {
            return Err(wasmtime::Error::msg("Insufficient fuel"));
        }
        Ok(())
    }

    fn setup_current_time(&mut self, current_fuel: u64) -> wasmtime::Result<TimeUs> {
        self.current_fuel = current_fuel;
        self.current_time()
    }

    fn current_time(&self) -> wasmtime::Result<TimeUs> {
        self.check_fuel()?;
        let remaining_fuel = self.current_fuel - self.skipped_fuel;
        Ok(self.total_simulation_time - time_us_for_fuel(remaining_fuel))
    }

    fn skip_fuel(&mut self, fuel: u64) -> wasmtime::Result<()> {
        self.skipped_fuel += fuel;
        self.check_fuel()?;
        Ok(())
    }

    fn skip_time(&mut self, time: TimeUs) -> wasmtime::Result<()> {
        self.skip_fuel(fuel_for_time_us(time))
    }

    fn set_current_time(&mut self, time: TimeUs) -> wasmtime::Result<()> {
        if time >= self.total_simulation_time {
            return Err(wasmtime::Error::msg(
                "Cannot advance time beyond total simulation time",
            ));
        }
        let remaining_time = self.total_simulation_time - time;
        let remaining_fuel = fuel_for_time_us(remaining_time);

        // remaining_fuel == self.current_fuel - self.skipped_fuel
        // self.skipped_fuel = remaining_fuel - self.current_fuel
        if remaining_fuel >= self.current_fuel {
            return Err(wasmtime::Error::msg("Not enough fuel to advance time"));
        }
        self.skipped_fuel = self.current_fuel - remaining_fuel;
        self.check_fuel()?;
        Ok(())
    }

    pub fn step(&mut self) {
        self.stepper.step();

        if !self.futures_by_activity.is_empty() {
            let is_active = self.stepper.is_active();
            let mut missing = Vec::new();
            let mut completed = BTreeSet::new();

            for id in self.futures_by_activity.iter().copied() {
                match self.futures_by_id.get_mut(&id) {
                    Some(f) => {
                        if f.value == FutureValueStatus::Pending {
                            match f.ready_condition {
                                FutureReadyCondition::IsActive => {
                                    if is_active {
                                        f.value = FutureValueStatus::Ready(DeviceValueRaw::zero());
                                        completed.insert(id);
                                    }
                                }
                                FutureReadyCondition::IsInactive => {
                                    if !is_active {
                                        f.value = FutureValueStatus::Ready(DeviceValueRaw::zero());
                                        completed.insert(id);
                                    }
                                }
                                FutureReadyCondition::ReadyAt(_) => {}
                            }
                        }
                    }
                    None => missing.push(id),
                }
            }
            self.futures_by_activity
                .retain(|id| !completed.contains(id));
        }

        if !self.futures_by_ready_time.is_empty() {
            let now = self.stepper.get_time_us();
            let mut to_remove = Vec::new();

            for rt in self
                .futures_by_ready_time
                .iter()
                .copied()
                .take_while(|rt| rt.ready_at <= now)
            {
                match self.futures_by_id.get_mut(&rt.id) {
                    Some(f) => {
                        if f.value == FutureValueStatus::Pending {
                            match f.ready_condition {
                                FutureReadyCondition::ReadyAt(ready_time) => {
                                    let value =
                                        f.operation.compute_value(&self.stepper, ready_time);
                                    f.value = FutureValueStatus::Ready(value);
                                }
                                FutureReadyCondition::IsActive
                                | FutureReadyCondition::IsInactive => {}
                            }
                            to_remove.push(rt);
                        }
                    }
                    None => to_remove.push(rt),
                }
            }
            for rt in to_remove {
                self.futures_by_ready_time.remove(&rt);
            }
        }
    }

    pub fn step_until_time(&mut self, target_time: TimeUs) {
        while self.stepper.get_time_at_next_step_us() < target_time {
            self.step();
        }
    }

    pub fn write_log_file(&self) {
        if let Some(path) = &self.workdir_path {
            let log_file_path = path.join("log.txt");
            let log_file_text = self.log_lines.join("\n");
            if let Err(err) = std::fs::write(&log_file_path, &log_file_text) {
                eprintln!("Error writing file {}: {}", log_file_path.display(), err);
            }
        }
    }
}
