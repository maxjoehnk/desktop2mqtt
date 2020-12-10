mod config;
mod modules;

use crate::config::Config;
use crate::modules::*;
use std::fs::File;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config_file = File::open("config.yml")?;
    let config: Config = serde_yaml::from_reader(&config_file)?;

    let (mqtt_sender, mqtt_receiver) = mpsc::unbounded_channel();
    let (state_sender, state_receiver) = mpsc::unbounded_channel();

    let mut mqtt_module = MqttModule::new(mqtt_receiver);
    let mut hass_discovery_module = HomeAssistantModule::new(mqtt_sender.clone());
    let mut state_module = StateModule::new(mqtt_sender.clone(), state_receiver);
    let mut idle_module = IdleModule::new(state_sender);

    tokio::try_join!(
        mqtt_module.run(&config),
        hass_discovery_module.run(&config),
        state_module.run(&config),
        idle_module.run(&config)
    )?;

    Ok(())
}
