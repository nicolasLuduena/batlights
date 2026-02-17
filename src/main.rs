use clap::{Parser, Subcommand, ValueEnum};

use crate::controller::Controller;

mod bluetooth;
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

const MAC_ADDR: &str = "AC:C2:01:C9:38:5D";
const CHARACTERISTIC_UUID: &str = "0000ffe1-0000-1000-8000-00805f9b34fb";

#[tokio::main]
async fn main() -> Result<(), String> {
    let cmd = BatLights::parse();
    let bluetooth = crate::bluetooth::BluetoothConnection::new(
        MAC_ADDR.to_string(),
        CHARACTERISTIC_UUID.to_string(),
    )
    .await?;

    let payload = match cmd.command {
        Commands::Power { state } => Controller::power(state == PowerState::On),
        Commands::Color { r, g, b } => Controller::color(controller::Color { r, g, b }),
        Commands::Pattern { index } => Controller::pattern(index),
        Commands::Mic { sensitivity } => Controller::mic(sensitivity),
    };

    bluetooth.write(payload).await?;

    bluetooth.bye().await?;

    Ok(())
}
