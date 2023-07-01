use serde::Deserialize;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HTU21DFConfig {
    polling_freq: u32,
    zenoh_topic_channel: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Agents {
    HTU21DF(HTU21DFConfig)
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    id: String,
    zenoh_hub_endpoint: String,
    agents: Option<Vec<Agents>>,
}


#[cfg(test)]
mod tests {
    use crate::HTU21DF;

    use super::*;

    #[test]
    fn create_agent_config() {
        let test_id = "test";
        let test_endpoint = "/endpoint";

        let test_config_json = r#"
        {
            "id": "test",
            "zenohHubEndpoint": "/endpoint"
        }"#;

        let agent_config: AgentConfig = serde_json::from_str(test_config_json).unwrap();

        assert!(agent_config.id == test_id);
        assert!(agent_config.zenoh_hub_endpoint == test_endpoint);
        assert!(agent_config.agents.is_none())
    }

    #[test]
    fn create_agent_config_htu21df() {
        let test_id = "test";
        let test_endpoint = "/endpoint";
        let test_polling_freq = 10;
        let test_channel = "/test";

        let test_config_json = r#"
        {
            "id": "test",
            "zenohHubEndpoint": "/endpoint",
            "agents": [
                {
                    "type": "HTU21DF",
                    "pollingFreq" : 10,
                    "zenohTopicChannel" : "/test"
                }
            ]
        }"#;

        let agent_config: AgentConfig = serde_json::from_str(test_config_json).unwrap();

        assert!(agent_config.id == test_id);
        assert!(agent_config.zenoh_hub_endpoint == test_endpoint);
        assert!(agent_config.agents.is_some());

        let agents = agent_config.agents.unwrap();

        assert!(agents.len() == 1);

        let htu21df_agent = &agents[0];

        let device_config = match htu21df_agent {
            Agents::HTU21DF(conf) => Some(conf),
            _ => None
        }; 

        assert!(device_config.is_some());

        let device_config = device_config.unwrap();

        assert!(device_config.polling_freq == test_polling_freq);
        assert!(device_config.zenoh_topic_channel == test_channel);
    }
}
