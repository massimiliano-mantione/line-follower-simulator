use app_builder::VisualizerData;
use bevy::{math::Vec2, text::cosmic_text::Angle};
use clap::{self, ArgEnum, Parser, Subcommand, ValueEnum};
use executor::{
    wasm_bindings::exports::robot::{Color, Configuration},
    wasmtime,
};
use runner::{get_bot_config_from_file, run_bot_from_file};
use track::{SegmentTransform, Track, TrackSegment};
use utils::Side;

use crate::app_builder::create_app;

mod app_builder;
mod bot;
mod data;
mod runner;
mod server;
mod track;
mod ui;
mod utils;
mod visualizer;

#[derive(Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum TrackId {
    /// A basic straight track
    Line,
    /// A test track with 90deg angles
    Angle,
    /// A test track with smooth turns
    Turn,
    /// A simple track
    #[default]
    Simple,
    /// A full racing track
    Race,
}

impl std::str::FromStr for TrackId {
    type Err = wasmtime::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <TrackId as ArgEnum>::from_str(s, true).map_err(|err| wasmtime::Error::msg(err.to_string()))
    }
}

#[derive(Parser)]
#[clap(name = "sim")]
#[clap(version = "1.0")]
#[clap(about = "Line Follower Simulator", long_about = None)]
struct Args {
    /// Simulation step period in us
    #[clap(long, short, default_value = "500")]
    period: u32,
    /// Track used in the simulation
    /// (one of line, angle, turn, simple, race)
    #[clap(long, short, default_value = "simple")]
    track: TrackId,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the simulator on a single robot
    Run {
        /// Path to the robot configuration file
        #[clap(long, short)]
        input: String,
        /// Directory where output data is saved (defaults to current directory)
        #[clap(long, short, default_value = ".")]
        output: String,
        /// Save robot logs
        #[clap(long, short)]
        logs: bool,
        /// Racing start time in us
        #[clap(long, short, default_value = "1000000")]
        start_time: u32,
        /// CLI only (run headless, without graphical visualizer)
        #[clap(long, short)]
        cli: bool,
    },
    /// Test a robot configuration
    Test {
        /// Path to the robot configuration file
        #[clap(long, short)]
        input: Option<String>,
    },
    /// Run the simulator accepting robots from HTTP requests
    Serve {
        /// Address the server will bind to
        #[clap(long, short, default_value = "0.0.0.0")]
        address: String,
        /// HTTP server port
        #[clap(long, short, default_value = "9999")]
        port: u16,
        /// Racing start time in us
        #[clap(long, short, default_value = "1000000")]
        start_time: u32,
    },
}

fn main() -> executor::wasmtime::Result<()> {
    let args = Args::parse();

    let track = Track::new(
        Vec2::new(5.0, 6.5),
        SegmentTransform::new(Vec2::new(0.5, -2.3), Angle::from_degrees(0.0)),
        vec![
            TrackSegment::start(),
            TrackSegment::straight(2.0),
            TrackSegment::ninety_deg_turn(0.5, Side::Right),
            TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
            TrackSegment::ninety_deg_turn(1.0, Side::Left),
            TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
            TrackSegment::end(),
        ],
    );

    let period = args.period;

    match args.cmd {
        Command::Run {
            input,
            output,
            logs,
            start_time,
            cli,
        } => {
            println!(
                "running robot \"{}\" output at path \"{}\" (write logs: {})...",
                input, output, logs
            );

            let bot_execution_data = run_bot_from_file(
                input,
                Some(output.clone()),
                logs,
                period,
                start_time,
                track.clone(),
            )?;
            println!(
                "data has {} frames",
                bot_execution_data.data.body_data.steps.len()
            );

            if !cli {
                create_app(
                    app_builder::AppType::Visualizer(app_builder::VisualizerData::Runner {
                        bot: bot_execution_data,
                        output,
                        logs,
                        period,
                        start_time,
                    }),
                    track,
                    period,
                )
                .run();
            }
        }
        Command::Test { input } => {
            let cfg = match input {
                Some(input) => match get_bot_config_from_file(input) {
                    Ok(config) => {
                        println!("test robot \"{}\"...", &config.name);
                        Some(config)
                    }
                    Err(err) => {
                        eprintln!("Error loading bot config: {}", err);
                        eprintln!("Using default config.");
                        None
                    }
                },
                None => None,
            };

            let bot_config = cfg.unwrap_or(Configuration {
                name: "NONAME".into(),
                color_main: Color { r: 0, g: 0, b: 255 },
                color_secondary: Color { r: 255, g: 0, b: 0 },
                width_axle: 100.0,
                length_front: 200.0,
                length_back: 20.0,
                clearing_back: 20.0,
                wheel_diameter: 20.0,
                gear_ratio_num: 1,
                gear_ratio_den: 20,
                front_sensors_spacing: 10.0,
                front_sensors_height: 4.0,
            });

            create_app(app_builder::AppType::Test(bot_config), track, period).run();
        }
        Command::Serve {
            address,
            port,
            start_time,
        } => {
            println!("Starting server...");
            create_app(
                app_builder::AppType::Visualizer(VisualizerData::Server {
                    address,
                    port,
                    period,
                    start_time,
                }),
                track,
                period,
            )
            .run();
        }
    }

    Ok(())
}
