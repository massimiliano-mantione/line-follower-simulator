use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    bot::motors::Wheel,
    utils::{SetBySide, Side},
};
use execution_data::{MotorAngles, SensorsData};

impl SetBySide<f32> for MotorAngles {
    fn set_by_side(&mut self, side: Side, value: f32) {
        match side {
            Side::Left => self.left = value,
            Side::Right => self.right = value,
        };
    }
}

pub fn compute_motor_angles_position(
    wheels_query: Query<(&Wheel, &Transform)>,
    mut sensors_data: ResMut<SensorsData>,
    mut motor_angles: ResMut<MotorAngles>,
) {
    for (wheel, transform) in &wheels_query {
        // Rotation in radians [0, 2pi]
        let rot = PI
            - Vec3::from(transform.rotation.to_euler(EulerRot::XYZ))
                .dot(transform.rotation * wheel.axle.abs());

        motor_angles.set_by_side(wheel.side, rot);
    }
    sensors_data.motor_angles = *motor_angles;
}
