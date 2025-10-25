use bevy::prelude::*;

use execution_data::SensorsData;

pub mod bot_position;
pub mod imu;
pub mod line_sensors;
pub mod motor_angles;

use bot_position::compute_bot_position;
use imu::compute_imu_data;
use line_sensors::compute_sensor_readings;
use motor_angles::compute_motor_angles_position;

#[allow(unused)]
fn print_sensors_data(sensors_data: Res<SensorsData>) {
    println!("line sensors: {:?}", sensors_data.line_sensors);
    println!("bot position: {:?}", sensors_data.bot_position);
    println!(
        "motor angles: l {} r {}",
        sensors_data.motor_angles.left, sensors_data.motor_angles.right
    );
    println!(
        "gyro: r {:.4} p {:.4} y {:.4}",
        sensors_data.gyro.roll_angular_speed,
        sensors_data.gyro.pitch_angular_speed,
        sensors_data.gyro.yaw_angular_speed
    );
    println!(
        "imu: r {:.4} p {:.4} y {:.4}",
        sensors_data.imu_fused.roll, sensors_data.imu_fused.pitch, sensors_data.imu_fused.yaw
    );
}

pub struct SensorsModelPlugin;

impl Plugin for SensorsModelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SensorsData::default()).add_systems(
            RunFixedMainLoop,
            (
                compute_sensor_readings,
                compute_bot_position,
                compute_motor_angles_position,
                compute_imu_data,
                // print_sensors_data,
            )
                .chain()
                .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
        );
    }
}
