use agent_lib::{TemperatureUnits, HTU32D};
use std::{thread, time};
fn main() {
    let mut agent = HTU32D::new(TemperatureUnits::Fahrenheit);
    let sleep_time = time::Duration::from_secs(5);

    loop {
        let sensor_data = agent.read_sensors().unwrap();
        println!("{}", sensor_data);
        thread::sleep(sleep_time);
    }
}
