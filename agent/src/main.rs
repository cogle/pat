use agent_lib::{HTU32D, TemperatureUnits};
use std::{thread, time};
fn main() {
    let mut agent = HTU32D::new(TemperatureUnits::Fahrenheit);
    let sleep_time = time::Duration::from_secs(5);

    loop {
        let temp = agent.read_temperature().unwrap();
        let humidity = agent.read_humidity().unwrap();
        println!("Temperature: {}", temp);
        println!("Humidity: {}", humidity);
        thread::sleep(sleep_time);
    }
}
