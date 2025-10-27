use crate::bot::BotPlugin;
use crate::runner::BotExecutionData;
use crate::track::{Track, TrackPlugin};
use crate::ui::GuiSetupPlugin;
use crate::utils::{EntityFeatures, NormalRandom};
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;
use executor::wasm_bindings::exports::robot::Configuration;

#[derive(Resource)]
pub struct BotConfigWrapper {
    pub config: Configuration,
}

impl BotConfigWrapper {
    pub fn new(config: Configuration) -> Self {
        Self { config }
    }
}

#[derive(Clone)]
pub enum VisualizerData {
    Server {
        address: String,
        port: u16,
        period: u32,
    },
    Runner {
        bot: BotExecutionData,
        output: String,
        logs: bool,
        period: u32,
    },
}

impl VisualizerData {
    pub fn output(&self) -> Option<String> {
        match self {
            VisualizerData::Server { .. } => None,
            VisualizerData::Runner { output, .. } => Some(output.clone()),
        }
    }

    pub fn logs(&self) -> bool {
        match self {
            VisualizerData::Server { .. } => false,
            VisualizerData::Runner { logs, .. } => *logs,
        }
    }

    pub fn period(&self) -> u32 {
        match self {
            VisualizerData::Server { period, .. } => *period,
            VisualizerData::Runner { period, .. } => *period,
        }
    }

    pub fn first_bot(&self) -> Option<BotExecutionData> {
        match self {
            VisualizerData::Server { .. } => None,
            VisualizerData::Runner { bot, .. } => Some(bot.clone()),
        }
    }
}

pub enum AppType {
    Simulator(Configuration),
    Test(Configuration),
    Visualizer(VisualizerData),
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::Simulator(_) => write!(f, "Simulator"),
            AppType::Test(_) => write!(f, "Test"),
            AppType::Visualizer(_) => write!(f, "Visualizer"),
        }
    }
}

impl AppType {
    fn entity_features(&self) -> EntityFeatures {
        match self {
            AppType::Simulator(_) => EntityFeatures::Physics,
            AppType::Test(_) => EntityFeatures::PhysicsAndVisualization,
            AppType::Visualizer(_) => EntityFeatures::Visualization,
        }
    }

    pub fn has_physics(&self) -> bool {
        self.entity_features().has_physics()
    }

    pub fn has_visualization(&self) -> bool {
        self.entity_features().has_visualization()
    }

    pub fn configuration(&self) -> Option<Configuration> {
        match self {
            AppType::Simulator(config) => Some(config.clone()),
            AppType::Test(config) => Some(config.clone()),
            AppType::Visualizer(_) => None,
        }
    }

    pub fn into_app_data(&self) -> (Option<Configuration>, Option<VisualizerData>) {
        match self {
            AppType::Simulator(config) => (Some(config.clone()), None),
            AppType::Test(config) => (Some(config.clone()), None),
            AppType::Visualizer(data) => (None, Some(data.clone())),
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

pub fn create_app(app_type: AppType, track: Track, step_period_us: u32) -> App {
    let mut app = App::new();

    let step_hz = 1_000_000.0 / (step_period_us as f64);
    app.insert_resource(Time::<Fixed>::from_hz(step_hz));
    app.insert_resource(NormalRandom::new());
    app.insert_resource(track);

    if app_type.has_visualization() {
        app.add_plugins(WindowSetupPlugin);
    } else {
        app.add_plugins(HeadlessSetupPlugin);
    }

    if app_type.has_physics() {
        app.add_plugins(RapierPhysicsSetupPlugin);
    }

    app.add_plugins((
        BotPlugin::new(app_type.entity_features(), app_type.configuration()),
        TrackPlugin::new(app_type.entity_features()),
    ));

    app.add_plugins(GuiSetupPlugin::new(app_type));

    app
}
