use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HTU21DFTemperatureConfig {
    polling_freq: u32
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HTU21DFHumidityConfig {
    polling_freq: u32
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HTU21DFConfig {
    #[serde(flatten)]
    humidity_sensor_config: Option<HTU21DFHumidityConfig>,
    #[serde(flatten)]
    temperature_sensor_config: Option<HTU21DFTemperatureConfig>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Sensor {
    HTU21DF(HTU21DFConfig)
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfig {
    id: String,
    zenoh_hub_endpoint: String,
    sensors: Option<Vec<Sensor>>,
}


#[cfg(test)]
mod tests {
    use crate::HTU21DF;

    use super::*;

    #[test]
    fn create_sensor_config() {
        let test_id = "test";
        let test_endpoint = "endpoint";

        let test_config_json = r#"
        {
            "id": "test",
            "zenohHubEndpoint": "endpoint"
        }"#;

        let sensor_config: SensorConfig = serde_json::from_str(test_config_json).unwrap();

        assert!(sensor_config.id == test_id);
        assert!(sensor_config.zenoh_hub_endpoint == test_endpoint);
        assert!(sensor_config.sensors.is_none())
    }

    #[test]
    fn create_sensor_config_htu21df() {
        let test_id = "test";
        let test_endpoint = "endpoint";
        let test_polling_freq = 10;

        let test_config_json = r#"
        {
            "id": "test",
            "zenohHubEndpoint": "endpoint",
            "sensors": [
                {
                    "type": "HTU21DF",
                    "humiditySensorConfig" : 
                        {
                            "pollingFreq": 10                            
                        }
                }
            ]
        }"#;

        let sensor_config: SensorConfig = serde_json::from_str(test_config_json).unwrap();

        assert!(sensor_config.id == test_id);
        assert!(sensor_config.zenoh_hub_endpoint == test_endpoint);
        assert!(sensor_config.sensors.is_some());

        let sensors = sensor_config.sensors.unwrap();

        assert!(sensors.len() == 1);

        let htu21df_sensor = &sensors[0];

        let device_config = match htu21df_sensor {
            Sensor::HTU21DF(conf) => Some(conf),
            _ => None
        }; 

        assert!(device_config.is_some());

        let device_config = device_config.unwrap();

    }
}
