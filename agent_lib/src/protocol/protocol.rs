use crate::agents::HTU21DFSensorData;
use serde::Serialize;

#[derive(Serialize)]
pub enum Payload {
    #[serde(rename = "htu21dsData")]
    HTU21DSData(HTU21DFSensorData),
}
