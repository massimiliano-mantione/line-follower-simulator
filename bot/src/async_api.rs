use futures_lite::future::zip;

use crate::{
    wasm_bindings::devices::{DeviceOperation, device_operation_async, device_operation_immediate},
    wasm_bindings_ext::DeviceValueExt,
};

pub use crate::async_framework::*;
pub use crate::blocking_api::{
    console_log, csv, get_time_us, set_motors_pwm, write_csv_file, write_plain_file,
};

/// Get the current values of all line sensors.
pub async fn get_line_sensors() -> [u8; 16] {
    let lf = device_operation_async(DeviceOperation::ReadLineLeft).into_future();
    let rf = device_operation_async(DeviceOperation::ReadLineRight).into_future();
    let (l, r) = zip(lf, rf).await;
    let mut result = [0; 16];
    (0..8)
        .into_iter()
        .map(|i| l.get_u8(i))
        .chain((0..8).into_iter().map(|i| r.get_u8(i)))
        .enumerate()
        .for_each(|(i, v)| result[i] = v);
    result
}

/// Get the current values of motor angles (returns left and right angles with 16 bits of precision).
pub async fn get_motor_angles() -> (u16, u16) {
    let values = device_operation_async(DeviceOperation::ReadMotorAngles)
        .into_future()
        .await;
    let left = values.get_u16(0);
    let right = values.get_u16(1);
    (left, right)
}

/// Get the current values of the gyro (returns pitch, roll, and yaw speed values in deg/s).
pub async fn read_gyro() -> (i16, i16, i16) {
    let values = device_operation_async(DeviceOperation::ReadGyro)
        .into_future()
        .await;
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Get the current absolute euler angles (returns pitch, roll, and yaw values in deg).
pub async fn get_imu_fused_data() -> (i16, i16, i16) {
    let values = device_operation_async(DeviceOperation::ReadImuFusedData)
        .into_future()
        .await;
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Sleep for the given time in microseconds.
pub async fn sleep_for(time_us: u32) {
    device_operation_async(DeviceOperation::SleepFor(time_us))
        .into_future()
        .await;
}

/// Sleep until the given time in microseconds.
pub async fn sleep_until(time_us: u32) {
    device_operation_async(DeviceOperation::SleepUntil(time_us))
        .into_future()
        .await;
}

/// Check if the remote is enabled.
pub fn remote_enabled() -> bool {
    device_operation_immediate(DeviceOperation::GetEnabled).get_u8(0) != 0
}

/// Wait for the remote to be enabled.
pub async fn wait_remote_enabled() {
    device_operation_async(DeviceOperation::WaitEnabled)
        .into_future()
        .await;
}

/// Wait for the remote to be disabled.
pub async fn wait_remote_disabled() {
    device_operation_async(DeviceOperation::WaitDisabled)
        .into_future()
        .await;
}
