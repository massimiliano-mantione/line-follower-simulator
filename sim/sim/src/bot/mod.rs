use bevy::prelude::*;

pub mod model;
pub mod motors;
pub mod sensors;
pub mod vis;

use model::setup_bot_model;
use motors::Wheel;
use vis::setup_bot_visualizer;

use crate::{
    bot::{motors::MotorsModelPlugin, sensors::SensorsModelPlugin},
    data::StoreExecDataPlugin,
    utils::{EntityFeatures, Side},
};

pub struct BotPlugin {
    features: EntityFeatures,
}

impl BotPlugin {
    pub fn new(features: EntityFeatures) -> Self {
        Self { features }
    }
}

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_bot_entities);
        if self.features.has_physics() {
            app.add_systems(Startup, setup_bot_model.after(setup_bot_entities));
            app.add_plugins((MotorsModelPlugin, SensorsModelPlugin, StoreExecDataPlugin));
        }
        if self.features.has_visualization() {
            app.add_systems(Startup, setup_bot_visualizer.after(setup_bot_entities));
        }
    }
}

#[derive(Component)]
pub struct BotBodyMarker;

pub fn setup_bot_entities(mut commands: Commands) {
    commands.spawn(BotBodyMarker);
    for side in [Side::Left, Side::Right] {
        commands.spawn(Wheel::new(Vec3::NEG_X * side.sign(), side));
    }
}
