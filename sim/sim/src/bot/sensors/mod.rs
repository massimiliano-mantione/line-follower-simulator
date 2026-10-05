use bevy::prelude::*;

use execution_data::{MotorAngles, SensorsData};

pub mod bot_position;
pub mod imu;
pub mod line_sensors;
pub mod motor_angles;

use bot_position::compute_bot_position;
use imu::compute_imu_data;
use line_sensors::compute_sensor_readings;
use motor_angles::compute_motor_angles_position;

use crate::app_builder::BotUpdate;

#[allow(unused)]
fn print_sensors_data(sensors_data: Res<SensorsData>) {
    println!("line sensors: {:?}", sensors_data.line_sensors);
    println!("bot position: {:?}", sensors_data.bot_position);
    println!(
        "motor angles: l {} r {}",
        sensors_data.motor_angles.left, sensors_data.motor_angles.right
    );
    // EXERCISE 12.8 (continued): print the new sensors here too.
}

pub struct SensorsModelPlugin;

impl Plugin for SensorsModelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SensorsData::default())
            .insert_resource(MotorAngles::default())
            .add_systems(
                BotUpdate,
                (
                    compute_sensor_readings,
                    compute_bot_position,
                    compute_motor_angles_position,
                    compute_imu_data,
                    // print_sensors_data,
                )
                    .chain(),
            );
    }
}
