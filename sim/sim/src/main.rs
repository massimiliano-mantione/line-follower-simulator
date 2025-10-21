use clap::{self, Parser, Subcommand};
use executor::{
    test_run,
    wasm_bindings::exports::robot::{Color, Configuration},
};

mod app_builder;
mod bot;
mod data;
mod motors;
mod runner;
mod sensors;
mod track;
mod ui;
mod utils;

use crate::app_builder::create_app;

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
    },
    /// Run the simulator accepting robots from HTTP requests
    Serve,
}

fn main() -> executor::wasmtime::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Command::Run {
            input,
            output,
            logs,
        } => {
            println!(
                "running robot \"{}\" output at path \"{}\" (write logs: {})...",
                input, output, logs
            );
            test_run(input, output, logs)?;
        }
        Command::Test {
            input,
            output,
            logs,
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
                front_sensors_height: 2.0,
            };
            create_app(app_builder::AppType::Simulator(bot_config))
                .set_runner(|mut app| {
                    loop {
                        println!("In main loop");
                        app.update();
                        if let Some(exit) = app.should_exit() {
                            return exit;
                        }
                    }
                })
                .run();
        }
        Command::Serve => {
            println!("Starting server...");
        }
    }

    Ok(())
}
