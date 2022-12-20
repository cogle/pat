use chrono::{DateTime, Utc};
use rppal::i2c::I2c;
use anyhow::Result;

use crate::error;
use std::fmt;

static DEVICE_ADDRESS: u16 = 0x40;

static SOFT_RESET_REGISTER_ADDRESS: usize = 0xFE;
static TEMPERATURE_REGISTER_ADDRESS: usize = 0xE3;
static HUMIDITY_REGISTER_ADDRESS: usize = 0xE5;

static CHECKSUM_DIVISOR: u16 = 0x131;

fn to_celcuis(temperature: f32) -> f32 {
    (temperature - 32.0) * (5.0 / 9.0)
}

fn to_fahrenhiet(temperature: f32) -> f32 {
    32.0 + (9.0 / 5.0) * temperature
}

#[derive(Clone, Copy)]
pub enum TemperatureUnits {
    Celsuis,
    Fahrenheit,
}

pub struct Temperature {
    temperature: f32,
    unit: TemperatureUnits,
}

impl Temperature {
    pub fn convert_to_celcuis(self: &mut Self) {
        match self.unit {
            TemperatureUnits::Fahrenheit => {
                self.temperature = to_celcuis(self.temperature);
                self.unit = TemperatureUnits::Celsuis;
            }
            _ => {}
        }
    }

    pub fn convert_to_fahrenhiet(self: &mut Self) {
        match self.unit {
            TemperatureUnits::Celsuis => {
                self.temperature = to_fahrenhiet(self.temperature);
                self.unit = TemperatureUnits::Fahrenheit;
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            TemperatureUnits::Celsuis => write!(f, "{:.2}°C", self.temperature),
            TemperatureUnits::Fahrenheit => write!(f, "{:.2}°F", self.temperature),
        }
    }
}

pub enum Humidity {
    Relative(f32),
}

impl std::fmt::Display for Humidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Humidity::Relative(humidity) => write!(f, "{:.2}%", humidity),
        }
    }
}
pub struct HTU21DF {
    comm_channel: I2c,
    temperature_unit: TemperatureUnits,
}

pub struct HTU21DFSensorData {
    temperature: Temperature,
    humidity: Humidity,
    timestamp: DateTime<Utc>,
}

impl std::fmt::Display for HTU21DFSensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] Temperature: {}\tHumidity: {}",
            self.timestamp, self.temperature, self.humidity
        )
    }
}

impl HTU21DF {
    pub fn new(temperature_unit: TemperatureUnits) -> Self {
        let mut comm_channel = I2c::new().unwrap();
        comm_channel.set_slave_address(DEVICE_ADDRESS).unwrap();
        
        comm_channel.write(&[SOFT_RESET_REGISTER_ADDRESS as u8]).unwrap();

        Self {
            comm_channel,
            temperature_unit,
        }
    }

    pub fn checksum_calc(data: &[u8]) -> u8 {
        let mut checksum: u8 = 0x00;
        for byte in data {
            checksum ^= byte;
            for _bit in 0..8 {
                if checksum & 0x80 != 0 {
                    checksum <<= 1;
                    checksum ^= CHECKSUM_DIVISOR as u8;
                } else {
                    checksum <<= 1;
                }
            }
        }
        checksum
    }

    fn temperature_formula(signal_value: f32) -> f32 {
        -46.85 + (175.72) * ((signal_value as f32) / (u32::pow(2, 16) as f32))
    }

    fn relative_humidity_formula(signal_value: f32) -> f32 {
        -6.0 + (125.0) * ((signal_value as f32) / (u32::pow(2, 16) as f32))
    }

    fn validate(data: &[u8], expected: u8) -> Result<(), crate::error::Error> {
        let checksum = Self::checksum_calc(&data[0..2]);

        if checksum == expected {
            Ok(())
        } else {
            Err(error::Error::ChecksumError {
                expected: checksum as u32,
                actual: expected as u32,
                data: data[0..2].to_vec(),
            })
        }
    }

    pub fn read_sensors(self: &mut Self) -> Result<HTU21DFSensorData> {
        let timestamp = Utc::now();
        let temperature = self.read_temperature()?;
        let humidity = self.read_humidity()?;

        Ok(HTU21DFSensorData {
            temperature,
            humidity,
            timestamp,
        })
    }

    pub fn read_temperature(self: &mut Self) -> Result<Temperature> {
        self.comm_channel
            .write(&[TEMPERATURE_REGISTER_ADDRESS as u8])?;

        let mut data = [0u8; 3];
        self.read(&mut data)?;

        self.calculate_temperature(&data)
    }

    pub fn read_humidity(self: &mut Self) -> Result<Humidity> {
        self.comm_channel
            .write(&[HUMIDITY_REGISTER_ADDRESS as u8])?;

        let mut data = [0u8; 3];
        self.read(&mut data)?;

        self.calculate_humidity(&data)
    }

    fn calculate_humidity(self: &Self, data: &[u8]) -> Result<Humidity> {
        let (signal_value, checksum_value) = self.parse_data_buffer(&data);
        Self::validate(&data, checksum_value)?;

        let converted_value = Self::relative_humidity_formula(signal_value as f32);

        Ok(Humidity::Relative(converted_value))
    }

    fn calculate_temperature(
        self: &Self,
        data: &[u8],
    ) -> Result<Temperature> {
        let (signal_value, checksum_value) = self.parse_data_buffer(&data);
        Self::validate(&data, checksum_value)?;

        // Returned Temperature is in Celsius
        let temperature = Self::temperature_formula(signal_value as f32);

        let converted_value = match self.temperature_unit {
            TemperatureUnits::Celsuis => temperature,
            TemperatureUnits::Fahrenheit => to_fahrenhiet(temperature),
        };

        Ok(Temperature {
            temperature: converted_value,
            unit: self.temperature_unit,
        })
    }

    fn parse_data_buffer(self: &Self, signal_data: &[u8]) -> (u16, u8) {
        let signal_value = ((signal_data[0] as u16) << 8) | (signal_data[1] as u16);
        let checksum_value = signal_data[2];

        (signal_value, checksum_value)
    }

    fn read(self: &mut Self, data_buffer: &mut [u8]) -> Result<()> {
        self.comm_channel.read(data_buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEMPERATURE_EPSILON: f32 = 0.2;
    static RELATIVE_HUMIDITY_EPSILON: f32 = 0.2;

    #[test]
    fn crc_checksum_test_1() {
        let checksum = HTU21DF::checksum_calc(&[0xDC]);
        assert_eq!(checksum, 0x79);
    }

    #[test]
    fn crc_checksum_test_2() {
        let checksum = HTU21DF::checksum_calc(&[0x68, 0x3A]);
        assert_eq!(checksum, 0x7C);
    }

    #[test]
    fn crc_checksum_test_3() {
        let checksum = HTU21DF::checksum_calc(&[0x4E, 0x85]);
        assert_eq!(checksum, 0x6B);
    }

    #[test]
    fn checksum_validate_test_1() {
        let result = HTU21DF::validate(&[0x68, 0x3A], 0x7C);
        assert!(result.is_ok());
    }

    #[test]
    fn checksum_validate_test_2() {
        let result = HTU21DF::validate(&[0x4E, 0x85], 0x6B);
        assert!(result.is_ok());
    }

    #[test]
    fn checksum_validate_negative_test_1() {
        let result = HTU21DF::validate(&[0x4E, 0x85], 0x7C);
        assert!(result.is_err());
    }

    #[test]
    fn checksum_validate_negative_test_2() {
        let result = HTU21DF::validate(&[0x68, 0x3A], 0x6B);
        assert!(result.is_err());
    }

    #[test]
    fn signal_to_temperature_test_1() {
        let input = 0x683A;
        let expected = 24.7;

        let temp = HTU21DF::temperature_formula(input as f32);
        assert!(
            (temp - expected).abs() <= TEMPERATURE_EPSILON,
            "Expected {} but got {}",
            expected,
            temp
        );
    }

    #[test]
    fn signal_to_humdity_test_1() {
        let input = 0x4E85;
        let expected = 32.2;

        let relative_humidity = HTU21DF::relative_humidity_formula(input as f32);
        assert!(
            (relative_humidity - expected).abs() <= RELATIVE_HUMIDITY_EPSILON,
            "Expected {} but got {}",
            expected,
            relative_humidity
        );
    }

    #[test]
    fn signal_to_humdity_test_2() {
        let input = 0x7C80;
        let expected = 54.8;
        let relative_humidity = HTU21DF::relative_humidity_formula(input as f32);
        assert!(
            (relative_humidity - expected).abs() <= RELATIVE_HUMIDITY_EPSILON,
            "Expected {} but got {}",
            expected,
            relative_humidity
        );
    }
}
