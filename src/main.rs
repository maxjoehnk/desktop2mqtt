mod config;
mod modules;
mod options;

use crate::config::{get_config, Config};
use crate::modules::*;
use crate::options::CliOptions;
use log::LevelFilter;
use structopt::StructOpt;
use tokio::sync::{broadcast, mpsc};
use mqtt_async_client::client::{Publish, Client};

fn main() -> anyhow::Result<()> {
    let options = CliOptions::from_args();
    setup_logging(options.verbose);

    let config = get_config(&options)?;

    log::info!("Starting desktop2mqtt...");

    let mut runtime = tokio::runtime::Runtime::new()?;

    let mut client = Client::builder().set_host(config.mqtt.url.clone()).build()?;

    runtime.block_on(run(&mut client, config.clone()))?;
    log::info!("Stopping desktop2mqtt...");

    runtime.block_on(go_offline(&client, &config))?;
    runtime.shutdown_background();

    Ok(())
}

async fn go_offline(client: &Client, config: &Config) -> anyhow::Result<()> {
    let msg = MqttMessage {
        topic: format!("desktop2mqtt/{}/availability", config.hass.entity_id),
        payload: "offline".to_string()
    };
    let mut publish = Publish::from(msg);
    publish.set_retain(true);
    client.publish(&publish).await?;
    Ok(())
}

async fn run(client: &mut Client, config: Config) -> anyhow::Result<()> {
    client.connect().await?;
    tokio::select! {
        result = run_loop(client, config) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {}
    };

    Ok(())
}

async fn run_loop(client: &mut Client, config: Config) -> anyhow::Result<()> {
    let (mqtt_sender, mqtt_receiver) = mpsc::unbounded_channel();
    let (mqtt_event_sender, mqtt_event_receiver) = broadcast::channel(10);
    let (state_sender, state_receiver) = mpsc::unbounded_channel();

    let mut mqtt_module = MqttModule::new(client, mqtt_receiver, mqtt_event_sender);
    let mut hass_discovery_module = HomeAssistantModule::new(mqtt_sender.clone());
    let mut state_module = StateModule::new(mqtt_sender.clone(), state_receiver);
    let mut idle_module = IdleModule::new(state_sender.clone());
    let mut backlight_module =
        get_backlight_module(state_sender, mqtt_event_receiver, config.backlight);

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
