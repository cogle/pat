use agent_lib::{TemperatureUnits, HTU21DF, Payload};
use anyhow::Result;
use lib::Connection;
use std::{thread, time};

#[async_std::main]
async fn main() -> Result<()> {
    let mut connection = Connection::new().await;
    let sleep_time = time::Duration::from_secs(5);

    let mut agent = HTU21DF::new(TemperatureUnits::Fahrenheit);

    let topic = String::from("test/temperature");
    connection.create_publisher(topic.clone()).await;

    loop {
        let sensor_data = agent.read_sensors().unwrap();
        println!("{}", sensor_data);
        connection.publish(&topic, Payload::HTU21DSData(sensor_data)).await?;
        thread::sleep(sleep_time);
    }
}
