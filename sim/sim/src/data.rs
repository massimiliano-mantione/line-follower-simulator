use bevy::prelude::*;

use crate::{
    app_builder::BotUpdate,
    bot::sensors::{bot_position::BotPositionDetector, imu::compute_imu_data},
};
use execution_data::{ExecutionData, MotorAngles};

#[allow(unused_variables, unused_mut)]
fn store_data(
    bot_query: Query<&Transform, With<BotPositionDetector>>,
    motor_angles: Res<MotorAngles>,
    mut exec_data: ResMut<ExecutionData>,
) {
}

pub struct StoreExecDataPlugin {
    step_period_us: u32,
    force_initially_started: bool,
}

impl StoreExecDataPlugin {
    pub fn new(step_period_us: u32, force_initially_started: bool) -> Self {
        Self {
            step_period_us,
            force_initially_started,
        }
    }
}

impl Plugin for StoreExecDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExecutionData::empty(
            self.step_period_us,
            self.force_initially_started,
        ))
        .add_systems(BotUpdate, store_data.after(compute_imu_data));
    }
}
