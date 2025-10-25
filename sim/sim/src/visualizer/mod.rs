use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};
use execution_data::ExecutionData;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::ui::RunnerGuiState;

#[derive(Component)]
pub struct BotVisualization {
    data: ExecutionData,
    config: Configuration,
}

pub fn spawn_bot_visualization(mut commands: Commands) {
    todo!()
}

pub fn sync_visualizer_time(
    vis_query: Query<(Entity, &BotVisualization)>,
    runner_gui_state: Res<RunnerGuiState>,
) {
    todo!()
}
