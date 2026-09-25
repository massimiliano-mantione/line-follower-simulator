use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::bot_position::BotPositionDetector;
use execution_data::SensorsData;

// `sensors_data` needs to be `mut` once EXERCISE 12.8 writes to it.
#[allow(unused_variables)]
pub fn compute_imu_data(
    bot_query: Query<(&Transform, &Velocity), With<BotPositionDetector>>,
    sensors_data: ResMut<SensorsData>,
) {
}
