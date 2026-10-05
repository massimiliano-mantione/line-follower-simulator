// Several of these are used only by the EXERCISE 6.x bodies once implemented.
#[allow(unused_imports)]
use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        system::{Commands, Query, Res},
    },
    math::Quat,
    pbr::StandardMaterial,
    render::mesh::Mesh,
    transform::components::Transform,
};
use execution_data::{
    ActivityData, BodyExecutionData, BotFinalStatus, ExecutionData, WheelExecutionData,
};
use executor::wasm_host::exports::robot::Configuration;

#[allow(unused_imports)]
use crate::{
    bot::vis::{BotAssets, spawn_bot_body, spawn_bot_wheel},
    track::{Track, setup_track},
    ui_runner::RunnerGuiState,
    utils::EntityFeatures,
};

#[derive(Component)]
pub struct BotVisualization {
    pub config: Configuration,
    pub bot_activity: ActivityData,
    pub bot_final_status: BotFinalStatus,
}

// Used by EXERCISE 6.7 once implemented.
#[allow(dead_code)]
const VIS_LAYER_Z_STEP: f32 = 0.7;

#[allow(dead_code)]
impl BotVisualization {
    #[allow(unused_variables)]
    pub fn build_transform(&self, layer: usize) -> Transform {
        // EXERCISE 6.7: place this robot's whole world on its own stacked layer.
        //
        // Each robot gets a private copy of the track, one VIS_LAYER_Z_STEP above
        // the next, so several races can be compared side by side. See 6.9 for who
        // decides which layer.
        Transform::default()
    }
}

#[allow(unused_variables)]
pub fn spawn_bot_visualization(
    commands: &mut Commands,
    track: &Track,
    data: ExecutionData,
    configuration: Configuration,
    bot_assets: &BotAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // EXERCISE 6.8: build the visualization for one finished run.
    //
    // This is the punchline of the lesson: the *same* entity tree is used twice,
    // with two disjoint component sets. During simulation the bot entities carried
    // `Collider`, `RigidBody`, `ImpulseJoint`, `Velocity`. Here the very same spawn
    // functions build the same tree, but what gets attached is the recorded data.
    //
    // Build, in order:
    //
    //  1. A root entity carrying a `BotVisualization` (holding the config, the
    //     activity data and its final status) and the transform from 6.7.
    //  2. This robot's own copy of the track under that root, with
    //     `EntityFeatures::Visualization` and `is_bottom = false` - which is what
    //     gives the stacked copies a translucent floor while the bottom one stays
    //     opaque. See `setup_track`.
    //  3. The bot body under the root, and a wheel on each side under the body,
    //     passing the recorded `BodyExecutionData` / `WheelExecutionData` so the
    //     sync systems below have something to read.
    //
    // `spawn_bot_body` and `spawn_bot_wheel` are shared with the simulator and need
    // no changes - look at what their `data` parameters do.
    //
    // Until this is written, loading a robot appears to do nothing.
}

#[allow(unused_variables, unused_mut)]
pub fn sync_bot_layers(mut layers: Query<(&mut BotVisualization, &mut Transform)>) {
    // EXERCISE 6.9: re-sort the stack every frame, so the leader floats to the top.
    //
    // Rank the robots by `bot_final_status` (6.5 gave you the ordering) and assign
    // each one its layer transform (6.7). Careful with the direction: the *best*
    // result must end up on the *highest* layer, or the winner sinks to the bottom
    // of the stack and hides under everyone else's translucent floor.
}

#[allow(unused_variables)]
pub fn sync_bot_body(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&BodyExecutionData, &mut Transform)>,
) {
    // EXERCISE 6.10: pose every recorded body at the current playback time.
    //
    // `gui_state.play_time_sec()` is the single f32 the whole visualization is a
    // function of. The body wants its entire recorded transform.
}

#[allow(unused_variables)]
pub fn sync_bot_wheel(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&WheelExecutionData, &mut Transform)>,
) {
    // EXERCISE 6.11: rotate every recorded wheel to its angle at that same instant.
    //
    // Unlike the body, a wheel only wants its *rotation* changed - it is already
    // positioned relative to the body. Replace the whole transform and the wheels
    // teleport to the origin.
    //
    // See: WheelExecutionData::axis_rotation, Quat::from_axis_angle
}
