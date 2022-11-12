use serde::Deserialize;

// An Agent config is a configuration that maps to a remote IoT device that is collecting
// data and sending it back to the Hub.
#[derive(Deserialize)]
pub struct AgentConfig {
    pub device_name: String,
}

impl AgentConfig {
    pub fn new() -> Self {
        toml::from_str("").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_agent_config() {
        let config: AgentConfig = toml::from_str(
            r#"
        device_name = 'test_device'
        "#,
        )
        .unwrap();

        assert_eq!(config.device_name, "test_device");
    }
}
// TODO: ADD FORMAT FILE
