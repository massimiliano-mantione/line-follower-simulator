use bevy::prelude::*;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::bot::add_bot_setup;
use crate::data::add_data;
use crate::motors::add_motors;
use crate::sensors::add_sensors;
use crate::track::add_track;
use crate::ui::add_ui_setup;

#[derive(Resource)]
pub struct BotConfigWrapper {
    pub config: Configuration,
}

impl BotConfigWrapper {
    fn new(config: Configuration) -> Self {
        Self { config }
    }
}

pub enum AppType {
    Simulator(Configuration),
    Test(Configuration),
    Visualizer,
}

pub fn create_app(app_type: AppType) -> App {
    let mut app = App::new();

    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
            RapierContextInitialization::InitializeDefaultRapierContext {
                rapier_configuration: {
                    let mut config = RapierConfiguration::new(0.001);
                    config.gravity = Vec3::NEG_Z * 9.81;
                    config
                },
                integration_parameters: IntegrationParameters::default(),
            },
        ),
    ))
    .insert_resource(Time::<Fixed>::from_hz(10000.0));

    match app_type {
        AppType::Simulator(configuration) => {
            app.add_plugins(MinimalPlugins)
                .insert_resource(BotConfigWrapper::new(configuration));

            add_track(&mut app);
            add_bot_setup(&mut app);
            add_motors(&mut app);
            add_sensors(&mut app);

            add_data(&mut app);
        }
        AppType::Test(configuration) => {
            app.add_plugins((
                DefaultPlugins,
                DefaultEditorCamPlugins,
                RapierDebugRenderPlugin::default(),
            ))
            .insert_resource(BotConfigWrapper::new(configuration))
            .insert_resource(Time::from_hz(120.0));

            add_track(&mut app);
            add_bot_setup(&mut app);
            add_motors(&mut app);
            add_sensors(&mut app);

            add_ui_setup(&mut app);
        }
        AppType::Visualizer => {}
    };

    app
}
