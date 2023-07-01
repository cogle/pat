use serde::{Deserialize, Serialize};


enum DeviceType {
    Hub,
    RemoteNode
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    device_name: String,
}

