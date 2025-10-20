use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::sensors::bot_position::BotPositionDetector;
use execution_data::{GyroData, ImuFusedData, SensorsData};

pub fn compute_imu_data(
    bot_query: Query<(&GlobalTransform, &Velocity), With<BotPositionDetector>>,
    mut sensors_data: ResMut<SensorsData>,
) {
    let (transform, velocity) = bot_query.single().unwrap();

    let body_rot = Vec3::from(transform.rotation().to_euler(EulerRot::XYZ));
    sensors_data.gyro = GyroData::from(velocity.angvel * body_rot);

    sensors_data.imu_fused = ImuFusedData::from(body_rot);
}
