use clap::{Parser, Subcommand, ValueEnum};

use crate::controller::Controller;

mod controller;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct BatLights {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, PartialOrd)]
pub enum PowerState {
    On,
    Off,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Power { state: PowerState },
    Color { r: u8, g: u8, b: u8 },
    Pattern { index: u8 },
    Mic { sensitivity: u8 },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cmd = BatLights::parse();
    let mut controller = crate::controller::MockController::new();
    match cmd.command {
        Commands::Power { state } => controller.set_power(state == PowerState::On).await,
        Commands::Color { r, g, b } => controller.set_color(controller::Color { r, g, b }).await,
        Commands::Pattern { index } => controller.set_pattern(index).await,
        Commands::Mic { sensitivity } => controller.set_mic(sensitivity).await,
    }
}
