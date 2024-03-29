use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::core::mqtt::MqttCommand;
use crate::config::Config;
use crate::core::Worker;
use std::collections::HashMap;

pub struct State {
    sender: UnboundedSender<MqttCommand>,
    receiver: UnboundedReceiver<StateChange>,
}

impl State {
    pub fn new(
        sender: UnboundedSender<MqttCommand>,
        receiver: UnboundedReceiver<StateChange>,
    ) -> Self {
        State { sender, receiver }
    }
}

impl Worker for State {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let mut state = DesktopState::default();
        let topic = format!("desktop2mqtt/{}", config.hass.entity_id);
        async move {
            self.sender
                .send(MqttCommand::new_json(topic.clone(), &state)?)?;
            while let Some(value) = self.receiver.recv().await {
                log::debug!("Received state change {:?}", &value);
                match value {
                    StateChange::Idle(idle) => {
                        state.occupancy = Some(!idle);
                    }
                    StateChange::Backlight { brightness, power } => {
                        state.backlight_brightness = brightness;
                        state.backlight_power = power.into();
                    }
                    StateChange::Sensor { name, value } => {
                        state.sensors.insert(name, value);
                    }
                }
                self.sender
                    .send(MqttCommand::new_json(topic.clone(), &state)?)?;
            }

            Ok(())
        }
        .boxed()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateChange {
    Idle(bool),
    Backlight { power: bool, brightness: u32 },
    Sensor { name: String, value: f32 },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DesktopState {
    pub occupancy: Option<bool>,
    #[serde(rename = "state")]
    pub backlight_power: PowerState,
    #[serde(rename = "brightness")]
    pub backlight_brightness: u32,
    pub sensors: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacklightState {
    #[serde(rename = "state")]
    pub power: PowerState,
    pub brightness: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerState {
    #[serde(rename = "ON")]
    On,
    #[serde(rename = "OFF")]
    Off,
}

impl Default for PowerState {
    fn default() -> Self {
        PowerState::On
    }
}

impl From<bool> for PowerState {
    fn from(power: bool) -> Self {
        if power {
            PowerState::On
        } else {
            PowerState::Off
        }
    }
}

impl From<PowerState> for bool {
    fn from(power: PowerState) -> Self {
        match power {
            PowerState::On => true,
            PowerState::Off => false,
        }
    }
}
