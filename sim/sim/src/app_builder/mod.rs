use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::bot::BotPlugin;
use crate::track::TrackPlugin;
use crate::ui::GuiSetup;
use crate::utils::EntityFeatures;

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

impl AppType {
    fn entity_features(&self) -> EntityFeatures {
        match self {
            AppType::Simulator(_) => EntityFeatures::Physics,
            AppType::Test(_) => EntityFeatures::PhysicsAndVisualization,
            AppType::Visualizer => EntityFeatures::Visualization,
        }
    }

    pub fn has_physics(&self) -> bool {
        self.entity_features().has_physics()
    }

    pub fn has_visualization(&self) -> bool {
        self.entity_features().has_visualization()
    }

    pub fn into_configuration(self) -> Option<Configuration> {
        match self {
            AppType::Simulator(config) => Some(config),
            AppType::Test(config) => Some(config),
            AppType::Visualizer => None,
        }
    }
}

pub struct HeadlessSetupPlugin;

impl Plugin for HeadlessSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin::default(),
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>();
    }
}

pub struct WindowSetupPlugin;

impl Plugin for WindowSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            .insert_resource(Time::from_hz(120.0));
    }
}

pub struct RapierPhysicsSetupPlugin;

impl Plugin for RapierPhysicsSetupPlugin {
    fn build(&self, app: &mut App) {
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
        ));
    }
}

pub fn create_app(app_type: AppType, step_period_us: u32) -> App {
    let mut app = App::new();

    let step_hz = 1_000_000.0 / (step_period_us as f64);
    app.insert_resource(Time::<Fixed>::from_hz(step_hz));

    if app_type.has_visualization() {
        app.add_plugins(WindowSetupPlugin);
    } else {
        app.add_plugins(HeadlessSetupPlugin);
    }

    if app_type.has_physics() {
        app.add_plugins(RapierPhysicsSetupPlugin);
    }

    app.add_plugins((
        BotPlugin::new(app_type.entity_features()),
        TrackPlugin::new(app_type.entity_features()),
    ));

    app.add_plugins(GuiSetup::new(app_type.entity_features()));

    app_type
        .into_configuration()
        .map(|config| app.insert_resource(BotConfigWrapper::new(config)));

    app
}
