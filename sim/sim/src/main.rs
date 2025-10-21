use clap::{self, Parser, Subcommand};
use executor::test_run;
use runner::simulator_runner;

mod app_builder;
mod bot;
mod data;
mod motors;
mod runner;
mod sensors;
mod track;
mod ui;
mod utils;

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

            let (data, _cfg) = simulator_runner(input, output, logs)?;
            println!("data has {} frames", data.steps.len());
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

            test_run(input, output, logs)?;
        }
        Command::Serve => {
            println!("Starting server...");
        }
    }

    Ok(())
}
