use bevy::prelude::*;

use crate::bot::sensors::bot_position::BotPositionDetector;
use execution_data::{ExecutionData, MotorAngles};

fn store_data(
    bot_query: Query<&Transform, With<BotPositionDetector>>,
    motor_angles: Res<MotorAngles>,
    mut exec_data: ResMut<ExecutionData>,
) {
    let body_transform = *bot_query.single().unwrap();
    exec_data.body_data.steps.push(body_transform);
    exec_data.left_wheel_data.steps.push(motor_angles.left);
    exec_data.right_wheel_data.steps.push(motor_angles.right);
}

pub struct StoreExecDataPlugin;

impl Plugin for StoreExecDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExecutionData::default()).add_systems(
            RunFixedMainLoop,
            (store_data)
                .chain()
                .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
        );
    }
}
