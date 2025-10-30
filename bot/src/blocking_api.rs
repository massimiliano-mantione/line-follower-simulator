use crate::{
    wasm_bindings::{
        devices::{
            DeviceOperation, device_operation_blocking, device_operation_immediate,
            set_motors_power,
        },
        diagnostics::{CsvColumn, write_file, write_line},
    },
    wasm_bindings_ext::DeviceValueExt,
};

/// Get the current values of all line sensors.
pub fn get_line_sensors() -> [u8; 16] {
    let l = device_operation_immediate(DeviceOperation::ReadLineLeft);
    let r = device_operation_immediate(DeviceOperation::ReadLineRight);
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
pub fn get_motor_angles() -> (u16, u16) {
    let values = device_operation_immediate(DeviceOperation::ReadMotorAngles);
    let left = values.get_u16(0);
    let right = values.get_u16(1);
    (left, right)
}

/// Get the current values of the gyro (returns pitch, roll, and yaw speed values in deg/s).
pub fn read_gyro() -> (i16, i16, i16) {
    let values = device_operation_immediate(DeviceOperation::ReadGyro);
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Get the current absolute euler angles (returns pitch, roll, and yaw values in deg).
pub fn get_imu_fused_data() -> (i16, i16, i16) {
    let values = device_operation_immediate(DeviceOperation::ReadImuFusedData);
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Get the current time in microseconds.
pub fn get_time_us() -> u32 {
    device_operation_immediate(DeviceOperation::GetTime).get_u32(0)
}

/// Sleep for the given time in microseconds.
pub fn sleep_for(time_us: u32) {
    device_operation_blocking(DeviceOperation::SleepFor(time_us));
}

/// Sleep until the given time in microseconds.
pub fn sleep_until(time_us: u32) {
    device_operation_blocking(DeviceOperation::SleepUntil(time_us));
}

/// Check if the remote is enabled.
pub fn remote_enabled() -> bool {
    device_operation_immediate(DeviceOperation::GetEnabled).get_u8(0) != 0
}

/// Wait for the remote to be enabled.
pub fn wait_remote_enabled() {
    device_operation_blocking(DeviceOperation::WaitEnabled);
}

/// Wait for the remote to be disabled.
pub fn wait_remote_disabled() {
    device_operation_blocking(DeviceOperation::WaitDisabled);
}

/// Log a message to the console.
pub fn console_log(text: &str) {
    write_line(text);
}

/// Write data into a binary file
pub fn write_plain_file(name: &str, data: &[u8]) {
    write_file(name, data, None);
}

/// Write data into a CSV file with the given specification
pub fn write_csv_file(name: &str, data: &[u8], spec: &[CsvColumn]) {
    write_file(name, data, Some(spec));
}

/// Set motors PWM duty cycle (from -1000 to 1000).
pub fn set_motors_pwm(left: i16, right: i16) {
    set_motors_power(left, right);
}

pub mod csv {
    use crate::wasm_bindings::diagnostics::{CsvColumn, NamedValue, ValueKind};
    pub const C_I8: ValueKind = ValueKind::Int8;
    pub const C_I16: ValueKind = ValueKind::Int16;
    pub const C_I32: ValueKind = ValueKind::Int32;
    pub const C_U8: ValueKind = ValueKind::Uint8;
    pub const C_U16: ValueKind = ValueKind::Uint16;
    pub const C_U32: ValueKind = ValueKind::Uint32;
    pub const PAD_8: ValueKind = ValueKind::Pad8;
    pub const PAD_16: ValueKind = ValueKind::Pad16;

    pub fn nv(name: &str, value: i32) -> NamedValue {
        NamedValue {
            name: name.to_string(),
            value,
        }
    }

    pub fn named<const S: usize>(values: [NamedValue; S]) -> ValueKind {
        ValueKind::Named(values.into_iter().collect())
    }

    pub fn col(name: &str, kind: ValueKind) -> CsvColumn {
        CsvColumn {
            name: name.to_string(),
            kind,
        }
    }
}
