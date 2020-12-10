use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub hass: HomeAssistantConfig,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAssistantConfig {
    pub entity_id: String,
    pub name: String,
}
