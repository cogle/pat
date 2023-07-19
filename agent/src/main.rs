use anyhow::Result;
use lib::Connection;
use rppal::i2c::I2c;
use sensor_lib::{Payload, TemperatureUnits, HTU21DF};
use std::{thread, time};

#[async_std::main]
async fn main() -> Result<()> {
    let mut connection = Connection::new().await;
    let sleep_time = time::Duration::from_secs(5);

    let i2c_comm = I2c::new().unwrap();

    let mut agent = HTU21DF::new(i2c_comm, TemperatureUnits::Fahrenheit);

    let topic = String::from("test/temperature");
    connection.create_publisher(topic.clone()).await;

    loop {
        let sensor_data = agent.read_sensors().unwrap();
        println!("{}", sensor_data);
        connection
            .publish(&topic, Payload::HTU21DSData(sensor_data))
            .await?;
        thread::sleep(sleep_time);
    }
}
