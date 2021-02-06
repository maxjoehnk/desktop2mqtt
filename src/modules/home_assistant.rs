use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::{BacklightProvider, Config, HomeAssistantConfig};
use crate::modules::Module;
use crate::mqtt::MqttCommand;

pub struct HomeAssistantModule {
    mqtt_sender: UnboundedSender<MqttCommand>,
}

impl Module for HomeAssistantModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let hass_config = config.hass.clone();
        let topic = format!("desktop2mqtt/{}", hass_config.entity_id);
        let device = Device::new(
            format!("desktop2mqtt_{}", hass_config.entity_id),
            hass_config.name.clone(),
        );
        let backlight = config.backlight;
        let expire_after = config.idle_poll_rate;
        async move {
            self.announce_occupancy(&hass_config, topic.clone(), device.clone(), expire_after)?;
            if backlight != BacklightProvider::None {
                self.announce_backlight(&hass_config, topic, device)?;
            }

            Ok(())
        }
        .boxed()
    }
}

impl HomeAssistantModule {
    pub fn new(mqtt_sender: UnboundedSender<MqttCommand>) -> Self {
        HomeAssistantModule { mqtt_sender }
    }

    fn announce_backlight(
        &self,
        config: &HomeAssistantConfig,
        topic: String,
        device: Device,
    ) -> anyhow::Result<()> {
        let config_topic = format!("homeassistant/light/{}/backlight/config", config.entity_id);
        let command_topic = format!("{}/set", topic);
        let msg = ConfigMessage::light(
            format!("{} Backlight", &config.name),
            format!("{}_backlight_desktop2mqtt", config.entity_id),
            device,
            topic,
            LightConfig {
                command_topic: command_topic.clone(),
                brightness: true,
                schema: "json".to_string(),
            },
        );

        self.mqtt_sender
            .send(MqttCommand::subscribe(command_topic))?;
        self.mqtt_sender
            .send(MqttCommand::new_json(config_topic, &msg)?)?;

        Ok(())
    }

    fn announce_occupancy(
        &self,
        config: &HomeAssistantConfig,
        topic: String,
        device: Device,
        expire_after: u64,
    ) -> anyhow::Result<()> {
        let config_topic = format!(
            "homeassistant/binary_sensor/{}/occupancy/config",
            config.entity_id
        );
        let msg = ConfigMessage::sensor(
            format!("{} Occupancy", &config.name),
            format!("{}_occupancy_desktop2mqtt", config.entity_id),
            device,
            topic,
            SensorConfig {
                device_class: "occupancy".to_string(),
                value_template: "{{ value_json.occupancy }}".to_string(),
                expire_after: Some(expire_after),
                ..Default::default()
            },
        );

        self.mqtt_sender
            .send(MqttCommand::new_json(config_topic, &msg)?)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigMessage {
    pub availability_topic: String,
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub device: Device,
    pub json_attributes_topic: String,
    #[serde(flatten)]
    pub sensor: Option<SensorConfig>,
    #[serde(flatten)]
    pub light: Option<LightConfig>,
}

impl ConfigMessage {
    fn sensor(
        name: String,
        id: String,
        device: Device,
        topic: String,
        config: SensorConfig,
    ) -> Self {
        ConfigMessage {
            availability_topic: format!("{}/availability", topic),
            name,
            unique_id: id,
            device,
            state_topic: topic.clone(),
            json_attributes_topic: topic,
            sensor: Some(config),
            light: None,
        }
    }

    fn light(name: String, id: String, device: Device, topic: String, config: LightConfig) -> Self {
        ConfigMessage {
            availability_topic: format!("{}/availability", topic),
            name,
            unique_id: id,
            device,
            state_topic: topic.clone(),
            json_attributes_topic: topic,
            sensor: None,
            light: Some(config),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorConfig {
    pub device_class: String,
    pub value_template: String,
    pub payload_off: bool,
    pub payload_on: bool,
    /// Defines the number of seconds after the sensor’s state expires, if it’s not updated.
    pub expire_after: Option<u64>,
}

impl Default for SensorConfig {
    fn default() -> Self {
        SensorConfig {
            device_class: String::new(),
            value_template: String::new(),
            payload_on: true,
            payload_off: false,
            expire_after: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LightConfig {
    pub command_topic: String,
    pub brightness: bool,
    pub schema: String,
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
