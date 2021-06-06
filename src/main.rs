mod config;
mod modules;
mod options;
mod core;

use crate::config::{get_config, Config};
use crate::core::*;
use crate::modules::*;
use crate::options::CliOptions;
use log::LevelFilter;
use mqtt_async_client::client::{Client, Publish};
use structopt::StructOpt;
use tokio::sync::{broadcast, mpsc};

fn main() -> anyhow::Result<()> {
    let options = CliOptions::from_args();
    setup_logging(options.verbose);

    let config = get_config(&options)?;

    log::info!("Starting desktop2mqtt...");
    log::trace!("Config: {:?}", config);

    let runtime = tokio::runtime::Runtime::new()?;

    let mut client = Client::builder()
        .set_host(config.mqtt.url.clone())
        .build()?;

    runtime.block_on(run(&mut client, config.clone()))?;
    log::info!("Stopping desktop2mqtt...");

    runtime.block_on(go_offline(&client, &config))?;
    runtime.shutdown_background();

    Ok(())
}

async fn go_offline(client: &Client, config: &Config) -> anyhow::Result<()> {
    let msg = MqttMessage {
        topic: format!("desktop2mqtt/{}/availability", config.hass.entity_id),
        payload: "offline".to_string(),
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
    let (mqtt_event_sender, _) = broadcast::channel(10);
    let (state_sender, state_receiver) = mpsc::unbounded_channel();

    let mut mqtt_worker = MqttWorker::new(client, mqtt_receiver, mqtt_event_sender.clone());
    let mut hass_discovery_worker = HomeAssistantWorker::new(mqtt_sender.clone());
    let mut state = State::new(mqtt_sender.clone(), state_receiver);
    let mut idle_module = IdleModule::new(state_sender.clone());
    let mut backlight_module = if let Some(backlight) = config.modules.backlight {
        get_backlight_module(state_sender.clone(), mqtt_event_sender.subscribe(), backlight)
    }else {
        Box::new(EmptyWorker) as Box<dyn LocalWorker>
    };
    let mut notifications_module = NotificationsModule::new(mqtt_event_sender.subscribe(), mqtt_sender);
    let mut sensors_module = SensorsModule::new(state_sender.clone());

    tokio::try_join!(
        mqtt_worker.run(&config),
        hass_discovery_worker.run(&config),
        state.run(&config),
        idle_module.run(&config),
        backlight_module.run(&config),
        notifications_module.run(&config),
        sensors_module.run(&config),
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
