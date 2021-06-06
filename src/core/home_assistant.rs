use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::{Config, HomeAssistantConfig, SensorType};
use crate::core::mqtt::MqttCommand;
use crate::core::worker::Worker;
use crate::modules::{SensorsModule, SensorClass};

pub struct HomeAssistantWorker {
    mqtt_sender: UnboundedSender<MqttCommand>,
}

impl Worker for HomeAssistantWorker {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let hass_config = config.hass.clone();
        let topic = format!("desktop2mqtt/{}", hass_config.entity_id);
        let device = Device::new(
            format!("desktop2mqtt_{}", hass_config.entity_id),
            hass_config.name.clone(),
        );
        let modules_config = config.modules.clone();
        async move {
            if let Some(idle) = modules_config.idle {
                let expire_after = idle.poll_rate * 2;
                self.announce_occupancy(&hass_config, topic.clone(), device.clone(), expire_after.as_secs())?;
            }
            if modules_config.backlight.is_some() {
                self.announce_backlight(&hass_config, topic.clone(), device.clone())?;
            }
            if modules_config.sensors.types.len() > 0 {
                self.announce_sensors(&hass_config, topic.clone(), device.clone(), &modules_config.sensors.types)?;
            }

            Ok(())
        }
        .boxed()
    }
}

impl HomeAssistantWorker {
    pub fn new(mqtt_sender: UnboundedSender<MqttCommand>) -> Self {
        HomeAssistantWorker { mqtt_sender }
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
        let msg = ConfigMessage::binary_sensor(
            format!("{} Occupancy", &config.name),
            format!("{}_occupancy_desktop2mqtt", config.entity_id),
            device,
            topic,
            BinarySensorConfig {
                device_class: "occupancy".to_string().into(),
                value_template: "{{ value_json.occupancy }}".to_string(),
                expire_after: Some(expire_after),
                ..Default::default()
            },
        );

        self.mqtt_sender
            .send(MqttCommand::new_json(config_topic, &msg)?)?;

        Ok(())
    }

    fn announce_sensors(
        &self,
        config: &HomeAssistantConfig,
        topic: String,
        device: Device,
        enabled_sensors: &[SensorType]
    ) -> anyhow::Result<()> {
        for sensor in SensorsModule::get_sensors(enabled_sensors)? {
            let config_topic = format!("homeassistant/sensor/{}/{}/config", config.entity_id, sensor.id);
            let msg = ConfigMessage::sensor(
                format!("{} {}", &config.name, sensor.name),
                format!("{}_{}_desktop2mqtt", config.entity_id, sensor.id),
                device.clone(),
                topic.clone(),
                SensorConfig {
                    device_class: sensor.class.to_hass_class(),
                    value_template: format!("{{{{ value_json.sensors.{} }}}}", sensor.id),
                    unit_of_measurement: sensor.class.to_unit(),
                    icon: sensor.icon,
                    ..Default::default()
                }
            );

            self.mqtt_sender.send(MqttCommand::new_json(config_topic, &msg)?)?;
        }
        Ok(())
    }
}

trait ToHassClass {
    fn to_hass_class(&self) -> Option<String>;

    fn to_unit(&self) -> Option<String>;
}

impl ToHassClass for SensorClass {
    fn to_hass_class(&self) -> Option<String> {
        match self {
            SensorClass::Generic => None,
            SensorClass::Battery => Some("battery".to_string()),
            SensorClass::Temperature => Some("temperature".to_string()),
        }
    }

    fn to_unit(&self) -> Option<String> {
        match self {
            SensorClass::Generic => Some("%".into()),
            SensorClass::Battery => Some("%".into()),
            SensorClass::Temperature => Some("°C".into()),
        }
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
    pub binary_sensor: Option<BinarySensorConfig>,
    #[serde(flatten)]
    pub sensor: Option<SensorConfig>,
    #[serde(flatten)]
    pub light: Option<LightConfig>,
}

impl ConfigMessage {
    fn binary_sensor(
        name: String,
        id: String,
        device: Device,
        topic: String,
        config: BinarySensorConfig,
    ) -> Self {
        ConfigMessage {
            availability_topic: format!("{}/availability", topic),
            name,
            unique_id: id,
            device,
            state_topic: topic.clone(),
            json_attributes_topic: topic,
            binary_sensor: Some(config),
            sensor: None,
            light: None,
        }
    }

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
            binary_sensor: None,
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
            binary_sensor: None,
            sensor: None,
            light: Some(config),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BinarySensorConfig {
    pub device_class: Option<String>,
    pub value_template: String,
    pub payload_off: bool,
    pub payload_on: bool,
    /// Defines the number of seconds after the sensor’s state expires, if it’s not updated.
    pub expire_after: Option<u64>,
}

impl Default for BinarySensorConfig {
    fn default() -> Self {
        BinarySensorConfig {
            device_class: None,
            value_template: String::new(),
            payload_on: true,
            payload_off: false,
            expire_after: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    pub value_template: String,
    /// Defines the number of seconds after the sensor’s state expires, if it’s not updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_of_measurement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>
}

impl Default for SensorConfig {
    fn default() -> Self {
        SensorConfig {
            device_class: None,
            value_template: String::new(),
            expire_after: None,
            unit_of_measurement: None,
            icon: None,
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
