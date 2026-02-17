use clap::{Parser, Subcommand, ValueEnum};

use crate::controller::Controller;

mod bluetooth;
mod controller;
mod tui;

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

#[derive(Subcommand, Debug, PartialEq, PartialOrd)]
pub enum Commands {
    Power { state: PowerState },
    Color { r: u8, g: u8, b: u8 },
    Pattern { index: u8 },
    Mic { sensitivity: u8 },
    Tui,
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

    if let Commands::Tui = cmd.command {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Spawn a task to handle bluetooth communication
        let bt_handle = tokio::spawn(async move {
            while let Some(payload) = rx.recv().await {
                if let Err(e) = bluetooth.write(payload).await {
                    eprintln!("BT Write Error: {}", e);
                }
            }
            if let Err(e) = bluetooth.bye().await {
                eprintln!("BT Disconnect Error: {}", e);
            }
        });

        // Run the TUI
        if let Err(e) = crate::tui::run(tx).await {
            eprintln!("TUI Error: {}", e);
        }

        // Wait for the bluetooth task to finish (it finishes when tx is dropped)
        let _ = bt_handle.await;
    } else {
        let payload = match cmd.command {
            Commands::Power { state } => Controller::power(state == PowerState::On),
            Commands::Color { r, g, b } => Controller::color(controller::Color { r, g, b }),
            Commands::Pattern { index } => Controller::pattern(index),
            Commands::Mic { sensitivity } => Controller::mic(sensitivity),
            _ => unreachable!("Tui handled above"),
        };
        bluetooth.write(payload).await?;
        bluetooth.bye().await?;
    }

    Ok(())
}
