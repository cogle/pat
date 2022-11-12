use serde_derive::Deserialize;

// An Agent config is a configuration that maps to a remote IoT device that is collecting
// data and sending it back to the Hub.
#[derive(Deserialize)]
struct MQTTConfig {
}

