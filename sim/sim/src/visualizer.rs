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
}

#[allow(unused_variables, unused_mut)]
pub fn sync_bot_layers(mut layers: Query<(&mut BotVisualization, &mut Transform)>) {
}

#[allow(unused_variables)]
pub fn sync_bot_body(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&BodyExecutionData, &mut Transform)>,
) {
}

#[allow(unused_variables)]
pub fn sync_bot_wheel(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&WheelExecutionData, &mut Transform)>,
) {
}
