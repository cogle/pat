use serde::Serialize;
use crate::agents::HTU21DFSensorData;

#[derive(Serialize)]
pub enum Payload {
    #[serde(rename = "htu21dsData")]
    HTU21DSData(HTU21DFSensorData),
}