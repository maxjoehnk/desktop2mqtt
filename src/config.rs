use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub hass: HomeAssistantConfig,
    pub idle_timeout: u64,
    #[serde(default)]
    pub backlight: BacklightProvider,
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

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BacklightProvider {
    None,
    RaspberryPi,
    Stub,
}

impl Default for BacklightProvider {
    fn default() -> Self {
        BacklightProvider::None
    }
}
