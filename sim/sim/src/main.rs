use app_builder::VisualizerData;
use clap::{self, Parser, Subcommand};
use executor::wasm_bindings::exports::robot::{Color, Configuration};
use runner::run_bot_from_file;

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

#[derive(Parser)]
#[clap(name = "sim")]
#[clap(version = "1.0")]
#[clap(about = "Line Follower Simulator", long_about = None)]
struct Args {
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
        /// Simulation step period in us
        #[clap(long, short, default_value = "500")]
        period: u32,
        /// CLI only (run headless, without graphical visualizer)
        #[clap(long, short)]
        cli: bool,
    },
    /// Test a robot configuration
    Test {
        /// Path to the robot configuration file
        #[clap(long, short)]
        input: String,
        /// Directory where output data is saved (defaults to current directory)
        #[clap(long, short, default_value = ".")]
        output: String,
        /// Save robot logs
        #[clap(long, short)]
        logs: bool,
        /// Simulation step period in us
        #[clap(long, short, default_value = "500")]
        period: u32,
    },
    /// Run the simulator accepting robots from HTTP requests
    Serve {
        /// Address the server will bind to
        #[clap(long, short, default_value = "0.0.0.0")]
        address: String,
        /// HTTP server port
        #[clap(long, short, default_value = "9999")]
        port: u16,
        /// Simulation step period in us
        #[clap(long, short, default_value = "500")]
        period: u32,
    },
}

fn main() -> executor::wasmtime::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Command::Run {
            input,
            output,
            logs,
            period,
            cli,
        } => {
            println!(
                "running robot \"{}\" output at path \"{}\" (write logs: {})...",
                input, output, logs
            );

            let bot_execution_data = run_bot_from_file(input, Some(output.clone()), logs, period)?;
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
                    }),
                    period,
                )
                .run();
            }
        }
        Command::Test {
            input,
            output,
            logs,
            period,
        } => {
            println!(
                "test robot \"{}\" output at path \"{}\" (write logs: {})...",
                input, output, logs
            );

            let bot_config = Configuration {
                name: "bot test".into(),
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
            };

            create_app(app_builder::AppType::Test(bot_config), period).run();
        }
        Command::Serve {
            address,
            port,
            period,
        } => {
            println!("Starting server...");
            create_app(
                app_builder::AppType::Visualizer(VisualizerData::Server {
                    address,
                    port,
                    period,
                }),
                period,
            )
            .run();
        }
    }

    Ok(())
}
