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
    // EXERCISE 6.6: record one sample per simulation tick. Three lines.
    //
    // The body's transform and both wheel angles, appended to the vectors in
    // `exec_data`. That is the entire recording - no sensor values, no PWM, no
    // forces. It is the minimum needed to redraw the run, and everything else a
    // robot author wants to inspect goes through telemetry that the *robot* writes
    // itself, because the robot knows what is worth recording and pays for it in
    // simulated time.
    //
    // Note where this system runs: `BotUpdate`, ordered after `compute_imu_data`, so
    // the sample it takes is the state *after* this tick's sensors and transforms
    // have been computed.
    //
    // Until this is written, every run reports "data has 0 frames".
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
