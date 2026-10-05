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
    // EXERCISE 12.8: compute both sensors from the physics state. Six lines.
    //
    // Rapier already knows the body's angular velocity, and its orientation is in
    // the transform - as a quaternion, which you will want as Euler angles.
    //
    //  - the gyro is angular velocity expressed in the body's own frame;
    //  - the fused IMU is just the body's orientation.
    //
    // Then register this system in `SensorsModelPlugin`'s chain in `mod.rs` - mind
    // the position, because `store_data` in `data.rs` is ordered `.after` it so that
    // the recording sees fresh values.
    //
    // See: Transform::rotation, Quat::to_euler, EulerRot::XYZ, Velocity::angular
}
