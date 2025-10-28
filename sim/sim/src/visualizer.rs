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
use executor::wasm_bindings::exports::robot::Configuration;

use crate::{
    bot::vis::{BotAssets, spawn_bot_body, spawn_bot_wheel},
    track::{Track, setup_track},
    ui_runner::RunnerGuiState,
    utils::EntityFeatures,
};

#[derive(Component)]
pub struct BotVisualization {
    pub config: Configuration,
    pub bot_number: usize,
    pub bot_activity: ActivityData,
    pub bot_final_status: BotFinalStatus,
}

const VIS_LAYER_Z_STEP: f32 = 0.7;

impl BotVisualization {
    pub fn build_transform(&self) -> Transform {
        Transform::from_xyz(0.0, 0.0, self.bot_number as f32 * VIS_LAYER_Z_STEP)
    }
}

pub fn spawn_bot_visualization(
    commands: &mut Commands,
    track: &Track,
    data: ExecutionData,
    configuration: Configuration,
    bot_number: usize,
    bot_assets: &BotAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let root_component = BotVisualization {
        config: configuration.clone(),
        bot_number,
        bot_activity: data.activity_data,
        bot_final_status: data.activity_data.final_status(),
    };
    let root_transform = root_component.build_transform();
    let track_root = commands.spawn((root_component, root_transform)).id();

    setup_track(
        commands,
        track_root,
        EntityFeatures::Visualization,
        track,
        false,
        meshes,
        materials,
    );

    let bot = spawn_bot_body(
        commands,
        track_root,
        &configuration,
        bot_assets,
        materials,
        Some(data.body_data),
    );
    spawn_bot_wheel(
        commands,
        bot,
        &configuration,
        bot_assets,
        materials,
        crate::utils::Side::Left,
        Some(data.left_wheel_data),
    );
    spawn_bot_wheel(
        commands,
        bot,
        &configuration,
        bot_assets,
        materials,
        crate::utils::Side::Right,
        Some(data.right_wheel_data),
    );
}

pub fn sync_bot_layers(layers: Query<(&BotVisualization, &mut Transform)>) {
    for (vis, mut transform) in layers {
        *transform = vis.build_transform();
    }
}

pub fn sync_bot_body(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&BodyExecutionData, &mut Transform)>,
) {
    for (data, mut transform) in data {
        *transform = data.at_time_secs(gui_state.play_time_sec());
    }
}

pub fn sync_bot_wheel(
    gui_state: Res<RunnerGuiState>,
    data: Query<(&WheelExecutionData, &mut Transform)>,
) {
    for (data, mut transform) in data {
        let angle = data.at_time_secs(gui_state.play_time_sec());
        transform.rotation = Quat::from_axis_angle(data.axis_rotation(), angle);
    }
}
