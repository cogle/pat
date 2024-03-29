use anyhow::Result;
use chrono::{DateTime, Utc};
use embedded_hal::blocking::i2c;
use serde::Serialize;

use crate::error;
use std::fmt;

static DEVICE_ADDRESS: u8 = 0x40;

static SOFT_RESET_COMMAND: usize = 0xFE;
static READ_TEMPERATURE_COMMAND: usize = 0xE3;
static READ_HUMIDITY_COMMAND: usize = 0xE5;

static CHECKSUM_DIVISOR: u16 = 0x131;

fn to_celcuis(temperature: f32) -> f32 {
    (temperature - 32.0) * (5.0 / 9.0)
}

fn to_fahrenhiet(temperature: f32) -> f32 {
    32.0 + (9.0 / 5.0) * temperature
}

#[derive(Clone, Copy, Serialize)]
pub enum TemperatureUnits {
    Celsuis,
    Fahrenheit,
}

#[derive(Serialize)]
pub struct Temperature {
    value: f32,
    unit: TemperatureUnits,
}

impl Temperature {
    pub fn get_temperature(self: &Self) -> (f32, TemperatureUnits) {
        (self.value, self.unit)
    }

    pub fn convert_to_celcuis(self: &mut Self) {
        match self.unit {
            TemperatureUnits::Fahrenheit => {
                self.value = to_celcuis(self.value);
                self.unit = TemperatureUnits::Celsuis;
            }
            _ => {}
        }
    }

    pub fn convert_to_fahrenhiet(self: &mut Self) {
        match self.unit {
            TemperatureUnits::Celsuis => {
                self.value = to_fahrenhiet(self.value);
                self.unit = TemperatureUnits::Fahrenheit;
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            TemperatureUnits::Celsuis => write!(f, "{:.2}°C", self.value),
            TemperatureUnits::Fahrenheit => write!(f, "{:.2}°F", self.value),
        }
    }
}

#[derive(Serialize)]
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

pub struct HTU21DF<I2C> {
    i2c_comm: I2C,
    temperature_unit: TemperatureUnits,
}

#[derive(Serialize)]
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

impl<I2C, E> HTU21DF<I2C>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
    E: std::fmt::Debug + std::error::Error + std::marker::Send + std::marker::Sync + 'static,
{
    pub fn new(mut i2c_comm: I2C, temperature_unit: TemperatureUnits) -> Self {
        i2c_comm
            .write(DEVICE_ADDRESS, &[SOFT_RESET_COMMAND as u8])
            .unwrap();

        Self {
            i2c_comm,
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
        self.i2c_comm
            .write(DEVICE_ADDRESS, &[READ_TEMPERATURE_COMMAND as u8])?;

        let mut data = [0u8; 3];
        self.read(&mut data)?;

        self.calculate_temperature(&data)
    }

    pub fn read_humidity(self: &mut Self) -> Result<Humidity> {
        self.i2c_comm
            .write(DEVICE_ADDRESS, &[READ_HUMIDITY_COMMAND as u8])?;

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

    fn calculate_temperature(self: &Self, data: &[u8]) -> Result<Temperature> {
        let (signal_value, checksum_value) = self.parse_data_buffer(&data);
        Self::validate(&data, checksum_value)?;

        // Returned Temperature is in Celsius
        let temperature = Self::temperature_formula(signal_value as f32);

        let converted_value = match self.temperature_unit {
            TemperatureUnits::Celsuis => temperature,
            TemperatureUnits::Fahrenheit => to_fahrenhiet(temperature),
        };

        Ok(Temperature {
            value: converted_value,
            unit: self.temperature_unit,
        })
    }

    fn parse_data_buffer(self: &Self, signal_data: &[u8]) -> (u16, u8) {
        let signal_value = ((signal_data[0] as u16) << 8) | (signal_data[1] as u16);
        let checksum_value = signal_data[2];

        (signal_value, checksum_value)
    }

    fn read(self: &mut Self, data_buffer: &mut [u8]) -> Result<()> {
        self.i2c_comm.read(DEVICE_ADDRESS, data_buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal::blocking::i2c::{Read, Write};

    const READ_VEC_LEN: usize = 3;

    #[derive(Default)]
    struct UnitTestMockI2C {
        mock_read_value: Vec<u8>,
    }

    impl UnitTestMockI2C {
        pub fn set_read_value(&mut self, read_value: Vec<u8>) {
            self.mock_read_value = read_value;
        }
    }

    #[derive(Debug)]
    struct MockError;

    impl std::error::Error for MockError {}

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "")
        }
    }

    impl Write for UnitTestMockI2C {
        type Error = MockError;
        fn write(&mut self, _addr: u8, _output: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl Read for UnitTestMockI2C {
        type Error = MockError;
        fn read(
            &mut self,
            _address: u8,
            buffer: &mut [u8],
        ) -> std::result::Result<(), Self::Error> {
            match self.mock_read_value.len() {
                READ_VEC_LEN => {
                    buffer.copy_from_slice(&self.mock_read_value);
                    Ok(())
                }
                _ => Err(MockError),
            }
        }
    }

    use super::*;
    static PRECISION_EPSILON: f32 = 0.1;

    fn close_enough(a: f32, b: f32) -> bool {
        (a - b).abs() <= PRECISION_EPSILON
    }

    #[test]
    fn crc_checksum_test_1() {
        let checksum = HTU21DF::<UnitTestMockI2C>::checksum_calc(&[0xDC]);
        assert_eq!(checksum, 0x79);
    }

    #[test]
    fn crc_checksum_test_2() {
        let checksum = HTU21DF::<UnitTestMockI2C>::checksum_calc(&[0x68, 0x3A]);
        assert_eq!(checksum, 0x7C);
    }

    #[test]
    fn crc_checksum_test_3() {
        let checksum = HTU21DF::<UnitTestMockI2C>::checksum_calc(&[0x4E, 0x85]);
        assert_eq!(checksum, 0x6B);
    }

    #[test]
    fn checksum_validate_test_1() {
        let result = HTU21DF::<UnitTestMockI2C>::validate(&[0x68, 0x3A], 0x7C);
        assert!(result.is_ok());
    }

    #[test]
    fn checksum_validate_test_2() {
        let result = HTU21DF::<UnitTestMockI2C>::validate(&[0x4E, 0x85], 0x6B);
        assert!(result.is_ok());
    }

    #[test]
    fn checksum_validate_negative_test_1() {
        let result = HTU21DF::<UnitTestMockI2C>::validate(&[0x4E, 0x85], 0x7C);
        assert!(result.is_err());
    }

    #[test]
    fn checksum_validate_negative_test_2() {
        let result = HTU21DF::<UnitTestMockI2C>::validate(&[0x68, 0x3A], 0x6B);
        assert!(result.is_err());
    }

    #[test]
    fn signal_to_temperature_test_1() {
        let input = 0x683A;
        let expected = 24.7;

        let temp = HTU21DF::<UnitTestMockI2C>::temperature_formula(input as f32);
        assert!(
            close_enough(temp, expected),
            "Expected {} but got {}",
            expected,
            temp
        );
    }

    #[test]
    fn signal_to_humdity_test_1() {
        let input = 0x4E85;
        let expected = 32.3;

        let relative_humidity = HTU21DF::<UnitTestMockI2C>::relative_humidity_formula(input as f32);
        assert!(
            close_enough(relative_humidity, expected),
            "Expected {} but got {}",
            expected,
            relative_humidity
        );
    }

    #[test]
    fn signal_to_humdity_test_2() {
        let input = 0x7C80;
        let expected = 54.8;
        let relative_humidity = HTU21DF::<UnitTestMockI2C>::relative_humidity_formula(input as f32);
        assert!(
            close_enough(relative_humidity, expected),
            "Expected {} but got {}",
            expected,
            relative_humidity
        );
    }

    #[test]
    fn read_temperature_test_1() {
        let expected = 24.7;

        let mut unit_test_mock_i2c = UnitTestMockI2C::default();
        unit_test_mock_i2c.set_read_value(vec![0x68, 0x3A, 0x7C]);

        let mut test_device = HTU21DF::new(unit_test_mock_i2c, TemperatureUnits::Celsuis);
        let temperature_read = test_device.read_temperature();

        assert!(temperature_read.is_ok());

        let (temp, _) = temperature_read.unwrap().get_temperature();

        assert!(
            close_enough(temp, expected),
            "Expected {} but got {}",
            expected,
            temp
        );
    }

    #[test]
    fn read_humidity_test_1() {
        let expected = 32.3;

        let mut unit_test_mock_i2c = UnitTestMockI2C::default();
        unit_test_mock_i2c.set_read_value(vec![0x4E, 0x85, 0x6B]);

        let mut test_device = HTU21DF::new(unit_test_mock_i2c, TemperatureUnits::Celsuis);
        let humidity_read = test_device.read_humidity();

        assert!(humidity_read.is_ok());

        let humidity = match humidity_read.unwrap() {
            Humidity::Relative(value) => value,
        };

        assert!(
            close_enough(humidity, expected),
            "Expected {} but got {}",
            expected,
            humidity
        );
    }
}
