pub mod async_event_loop;
#[allow(warnings)]
pub mod line_follower_robot;
pub mod value_ext;

use async_event_loop::{FutureHandleExt, pin_boxed};
use line_follower_robot::devices::{
    DeviceOperation, PollError, device_operation_async, device_operation_blocking, poll_loop,
};
use line_follower_robot::diagnostics::write_line;
use line_follower_robot::exports::robot::{Color, Configuration, Guest};
use value_ext::DeviceValueExt;

struct Component;

impl Guest for Component {
    fn setup() -> Configuration {
        Configuration {
            name: "Liner".to_string(),
            color_main: Color { r: 255, g: 0, b: 0 },
            color_secondary: Color { r: 0, g: 255, b: 0 },
            width_axle: 200.0,
            length_front: 300.0,
            length_back: 20.0,
            clearing_back: 3.0,
            wheel_diameter: 15.0,
            gear_ratio_num: 1,
            gear_ratio_den: 20,
            front_sensors_spacing: 4.0,
            front_sensors_height: 4.0,
        }
    }

    fn run() -> () {
        //simple_blocking_run();
        async_event_loop::run(pin_boxed(simple_async_run()));
    }
}

pub async fn simple_async_loop() -> Result<(), PollError> {
    for i in 1..1000 {
        let time = device_operation_blocking(DeviceOperation::GetTime);
        let gyro = device_operation_async(DeviceOperation::ReadGyro)
            .into_future()
            .await?;
        write_line(&format!(
            "log: {} time {} gyro {} {} {} {}",
            i,
            time.get_u32(0),
            gyro.get_i16(0),
            gyro.get_i16(1),
            gyro.get_i16(2),
            gyro.get_i16(3)
        ));
        device_operation_async(DeviceOperation::SleepFor(1_000_000))
            .into_future()
            .await?;
    }
    Ok(())
}

pub async fn simple_async_run() {
    if let Err(err) = simple_async_loop().await {
        println!("Error: {}", err);
    }
}

pub fn simple_blocking_run() {
    for i in 1..1000 {
        poll_loop(true);
        let time = device_operation_blocking(DeviceOperation::GetTime);
        let gyro = device_operation_blocking(DeviceOperation::ReadGyro);
        write_line(&format!(
            "log: {} time {} gyro {} {} {} {}",
            i,
            time.get_u32(0),
            gyro.get_i16(0),
            gyro.get_i16(1),
            gyro.get_i16(2),
            gyro.get_i16(3)
        ));
        device_operation_blocking(DeviceOperation::SleepFor(1_000_000));
        poll_loop(false);
    }
}

line_follower_robot::export!(Component with_types_in line_follower_robot);
