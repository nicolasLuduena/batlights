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
        let manager = Manager::new().await.map_err(|e| format!("BT Error: {e}"))?;
        let adapters = manager
            .adapters()
            .await
            .map_err(|e| format!("BT Error: {e}"))?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or("BT Error: No bluetooth adapter found")?;

        let _ = adapter.start_scan(ScanFilter::default()).await;
        time::sleep(Duration::from_millis(200)).await;

        let peripheral = adapter
            .peripherals()
            .await
            .map_err(|e| format!("BT Error: {e}"))?
            .into_iter()
            .find(|p| p.address().to_string() == mac)
            .ok_or(format!("Could not find peripheral with MAC address {mac}"))?;

        peripheral
            .connect()
            .await
            .map_err(|e| format!("BT Error: {e}"))?;
        peripheral
            .discover_services()
            .await
            .map_err(|e| format!("BT Error: {e}"))?;

        peripheral
            .discover_services()
            .await
            .map_err(|e| format!("BT Error: {e}"))?;

        let cmd_char = peripheral
            .characteristics()
            .into_iter()
            .find(|x| x.uuid == Uuid::parse_str(data_uuid.as_str()).unwrap())
            .ok_or("Port for writing color information not found")?;

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
            .map_err(|e| format!("BT Error: {e}"))?;
        Ok(())
    }
}
