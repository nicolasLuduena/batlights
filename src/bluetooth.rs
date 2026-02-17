use std::time::Duration;

use btleplug::api::{Characteristic, WriteType};
use btleplug::{
    api::{Central, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use tokio::time;
use uuid::Uuid;

pub struct BluetoothConnection {
    pub peripheral: Peripheral,
    pub characteristic: Characteristic,
}

impl BluetoothConnection {
    pub async fn new(mac: String, data_uuid: String) -> Result<BluetoothConnection, String> {
        let manager = Manager::new().await.expect("Error booting up bluetooth");
        let adapters = manager
            .adapters()
            .await
            .expect("Error loading bluetooth adapters");
        let adapter = adapters
            .into_iter()
            .next()
            .expect("No bluetooth adapter found");

        println!("Scanning");
        let _ = adapter.start_scan(ScanFilter::default()).await;
        time::sleep(Duration::from_millis(100)).await;

        let peripheral = adapter
            .peripherals()
            .await
            .expect("Could not load peripherals")
            .into_iter()
            .find(|p| p.address().to_string() == mac)
            .unwrap_or_else(|| panic!("Could not find peripheral with MAC address {mac}"));

        peripheral
            .connect()
            .await
            .expect("Could not connect to peripheral");
        println!("Connected! Discovering services...");
        peripheral
            .discover_services()
            .await
            .expect("Could not discover services");

        peripheral
            .discover_services()
            .await
            .expect("Error discovering services");

        let cmd_char = peripheral
            .characteristics()
            .into_iter()
            .find(|x| x.uuid == Uuid::parse_str(data_uuid.as_str()).unwrap())
            .expect("Port for writing color information not found");

        Ok(BluetoothConnection {
            peripheral,
            characteristic: cmd_char,
        })
    }

    pub async fn write(&self, payload: [u8; 9]) -> Result<(), String> {
        let _ = self
            .peripheral
            .write(&self.characteristic, &payload, WriteType::WithoutResponse)
            .await;
        Ok(())
    }

    pub async fn bye(&self) -> Result<(), String> {
        self.peripheral
            .disconnect()
            .await
            .expect("Error disconnecting");
        Ok(())
    }
}
