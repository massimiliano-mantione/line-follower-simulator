use bevy::prelude::*;

pub mod model;
pub mod motors;
pub mod sensors;
pub mod vis;

use executor::wasm_bindings::exports::robot::{Color, Configuration};
use model::setup_bot_model;
use motors::Wheel;
use vis::{setup_bot_assets, setup_test_bot_visualizer};

use crate::{
    bot::{motors::MotorsModelPlugin, sensors::SensorsModelPlugin},
    data::StoreExecDataPlugin,
    utils::{EntityFeatures, Side},
};

pub struct BotPlugin {
    features: EntityFeatures,
    configuration: Option<Configuration>,
}

#[derive(Resource)]
pub struct BotConfigurationResource {
    pub configuration: Configuration,
}

impl BotConfigurationResource {
    pub fn cfg(&self) -> Configuration {
        self.configuration.clone()
    }
}

impl BotPlugin {
    pub fn new(features: EntityFeatures, configuration: Option<Configuration>) -> Self {
        Self {
            features,
            configuration,
        }
    }
}

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        if self.features.has_visualization() {
            app.add_systems(Startup, setup_bot_assets);
        }
        if self.features.has_physics() {
            if self.features.has_visualization() {
                app.add_systems(Startup, setup_bot_entities.after(setup_bot_assets));
            } else {
                app.add_systems(Startup, setup_bot_entities);
            }
            app.add_systems(Startup, setup_bot_model.after(setup_bot_entities));
            app.add_plugins((MotorsModelPlugin, SensorsModelPlugin, StoreExecDataPlugin));
            if self.features.has_visualization() {
                let configuration = self
                    .configuration
                    .clone()
                    .expect("Test app must have a bot configuration");
                app.insert_resource(BotConfigurationResource { configuration });
                app.add_systems(Startup, setup_test_bot_visualizer.after(setup_bot_entities));
            }
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
