use crate::config::{Config, HomeAssistantConfig};
use crate::modules::Module;
use crate::mqtt::MqttMessage;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

pub struct HomeAssistantModule {
    mqtt_sender: UnboundedSender<MqttMessage>,
}

impl Module for HomeAssistantModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let hass_config = config.hass.clone();
        let topic = format!("desktop2mqtt/{}", hass_config.entity_id);
        let device = Device::new(
            format!("desktop2mqtt_{}", hass_config.entity_id),
            hass_config.name.clone(),
        );
        async move {
            self.announce_occupancy(&hass_config, topic, device)?;

            Ok(())
        }
        .boxed()
    }
}

impl HomeAssistantModule {
    pub fn new(mqtt_sender: UnboundedSender<MqttMessage>) -> Self {
        HomeAssistantModule { mqtt_sender }
    }

    fn announce_occupancy(
        &self,
        config: &HomeAssistantConfig,
        topic: String,
        device: Device,
    ) -> anyhow::Result<()> {
        let config_topic = format!(
            "homeassistant/binary_sensor/{}/occupancy/config",
            config.entity_id
        );
        let msg = ConfigMessage {
            name: format!("{} Occupancy", &config.name),
            unique_id: format!("{}_occupancy_desktop2mqtt", config.entity_id),
            state_topic: topic.clone(),
            device_class: "occupancy".to_string(),
            payload_off: false,
            payload_on: true,
            json_attributes_topic: topic,
            value_template: "{{ value_json.occupancy }}".to_string(),
            device,
        };

        self.mqtt_sender
            .send(MqttMessage::new_json(config_topic, &msg)?)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigMessage {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub device_class: String,
    pub device: Device,
    pub payload_off: bool,
    pub payload_on: bool,
    pub value_template: String,
    pub json_attributes_topic: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Device {
    pub identifiers: String,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub sw_version: String,
}

impl Device {
    pub fn new(id: String, name: String) -> Self {
        Device {
            identifiers: id,
            name,
            manufacturer: "Max Jöhnk".to_string(),
            model: env!("CARGO_PKG_NAME").to_string(),
            sw_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
