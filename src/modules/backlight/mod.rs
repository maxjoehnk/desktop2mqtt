use crate::config::{BacklightProvider, Config};
use futures_util::future::{BoxFuture, LocalBoxFuture};
use futures_util::FutureExt;
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;
use crate::core::state::{StateChange, PowerState};
use crate::core::mqtt::MqttMessage;
use crate::core::LocalWorker;

mod raspberry_pi;
mod stub;

struct BacklightModule<T: Backlight> {
    backlight: T,
    sender: UnboundedSender<StateChange>,
    receiver: broadcast::Receiver<MqttMessage>,
}

impl<T: Backlight> BacklightModule<T> {
    fn new(
        backlight: T,
        sender: UnboundedSender<StateChange>,
        receiver: broadcast::Receiver<MqttMessage>,
    ) -> Self {
        BacklightModule {
            backlight,
            sender,
            receiver,
        }
    }
}

impl<T: Backlight> LocalWorker for BacklightModule<T> {
    fn run(&mut self, config: &Config) -> LocalBoxFuture<anyhow::Result<()>> {
        let topic = format!("desktop2mqtt/{}/set", config.hass.entity_id);
        async move {
            let mut power = self.backlight.get_power().await?;
            let mut brightness = self.backlight.get_brightness().await?;

            self.sender
                .send(StateChange::Backlight { brightness, power })?;

            while let Ok(msg) = self.receiver.recv().await {
                if msg.topic != topic {
                    continue;
                }
                let state: BacklightUpdate = msg.deserialize()?;
                if let Some(next) = state.power {
                    power = next.into();
                    self.backlight.set_power(power).await?;
                }
                if let Some(next) = state.brightness {
                    brightness = next;
                    self.backlight.set_brightness(brightness).await?;
                }

                self.sender
                    .send(StateChange::Backlight { brightness, power })?;
            }

            Ok(())
        }
        .boxed_local()
    }
}

pub trait Backlight: Send {
    fn set_brightness(&mut self, value: u32) -> BoxFuture<anyhow::Result<()>>;
    fn get_brightness(&self) -> BoxFuture<anyhow::Result<u32>>;
    fn set_power(&mut self, value: bool) -> BoxFuture<anyhow::Result<()>>;
    fn get_power(&self) -> BoxFuture<anyhow::Result<bool>>;
}

pub fn get_backlight_module(
    sender: UnboundedSender<StateChange>,
    receiver: broadcast::Receiver<MqttMessage>,
    config: BacklightProvider,
) -> Box<dyn LocalWorker> {
    match config {
        BacklightProvider::RaspberryPi => to_module(
            self::raspberry_pi::RaspberryPiBacklight::new(),
            sender,
            receiver,
        ),
        BacklightProvider::Stub => to_module(self::stub::StubBacklight::new(), sender, receiver),
    }
}

fn to_module<TBacklight: Backlight + 'static>(
    backlight: TBacklight,
    sender: UnboundedSender<StateChange>,
    receiver: broadcast::Receiver<MqttMessage>,
) -> Box<dyn LocalWorker> {
    let module = BacklightModule::new(backlight, sender, receiver);

    Box::new(module)
}

#[derive(Debug, Clone, Deserialize)]
pub struct BacklightUpdate {
    #[serde(rename = "state")]
    pub power: Option<PowerState>,
    pub brightness: Option<u32>,
}
