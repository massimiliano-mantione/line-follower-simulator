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
    pub configuration: Option<Configuration>,
}

impl BotConfigurationResource {
    pub fn cfg(&self) -> Configuration {
        self.configuration
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Configuration {
                name: "NONAME".into(),
                color_main: Color { r: 0, g: 255, b: 0 },
                color_secondary: Color {
                    r: 255,
                    g: 0,
                    b: 255,
                },
                width_axle: 100.0,
                length_front: 100.0,
                length_back: 20.0,
                clearing_back: 10.0,
                wheel_diameter: 20.0,
                gear_ratio_num: 1,
                gear_ratio_den: 1,
                front_sensors_spacing: 10.0,
                front_sensors_height: 4.0,
            })
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
        app.insert_resource(BotConfigurationResource {
            configuration: self.configuration.clone(),
        });
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
