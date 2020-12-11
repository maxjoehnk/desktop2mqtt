mod config;
mod modules;
mod options;

use crate::config::get_config;
use crate::modules::*;
use crate::options::CliOptions;
use log::LevelFilter;
use structopt::StructOpt;
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = CliOptions::from_args();
    setup_logging(options.verbose);

    let config = get_config(&options)?;

    let (mqtt_sender, mqtt_receiver) = mpsc::unbounded_channel();
    let (mqtt_event_sender, mqtt_event_receiver) = broadcast::channel(10);
    let (state_sender, state_receiver) = mpsc::unbounded_channel();

    let mut mqtt_module = MqttModule::new(mqtt_receiver, mqtt_event_sender);
    let mut hass_discovery_module = HomeAssistantModule::new(mqtt_sender.clone());
    let mut state_module = StateModule::new(mqtt_sender.clone(), state_receiver);
    let mut idle_module = IdleModule::new(state_sender.clone());
    let mut backlight_module =
        get_backlight_module(state_sender, mqtt_event_receiver, config.backlight);

    log::info!("Starting desktop2mqtt...");

    tokio::try_join!(
        mqtt_module.run(&config),
        hass_discovery_module.run(&config),
        state_module.run(&config),
        idle_module.run(&config),
        backlight_module.run(&config)
    )?;

    Ok(())
}

fn setup_logging(verbose: u8) {
    let log_level = match verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter(None, log_level)
        .init();
}
