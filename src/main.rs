use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct BatLights {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
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

fn main() {
    let cmd = BatLights::parse();
    match cmd.command {
        Commands::Power { state } => println!("Power set to: {:?}", state),
        Commands::Color { r, g, b } => println!("Setting colors to #{r:02X}{g:02X}{b:02X}"),
        Commands::Pattern { index } => println!("Setting pattern {index}"),
        Commands::Mic { sensitivity } => println!("Mic sensitivity {sensitivity}"),
    }
}
