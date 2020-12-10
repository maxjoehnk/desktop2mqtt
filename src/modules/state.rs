use crate::config::Config;
use crate::modules::Module;
use crate::mqtt::MqttMessage;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use serde::Serialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct StateModule {
    sender: UnboundedSender<MqttMessage>,
    receiver: UnboundedReceiver<StateChange>,
}

impl StateModule {
    pub fn new(
        sender: UnboundedSender<MqttMessage>,
        receiver: UnboundedReceiver<StateChange>,
    ) -> Self {
        StateModule { sender, receiver }
    }
}

impl Module for StateModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let mut state = DesktopState::default();
        let topic = format!("desktop2mqtt/{}", config.hass.entity_id);
        async move {
            self.sender
                .send(MqttMessage::new_json(topic.clone(), &state)?)?;
            while let Some(value) = self.receiver.recv().await {
                log::debug!("Received state change {:?}", &value);
                match value {
                    StateChange::Idle(idle) => {
                        state.occupancy = Some(!idle);
                    }
                }
                self.sender
                    .send(MqttMessage::new_json(topic.clone(), &state)?)?;
            }

            Ok(())
        }
        .boxed()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StateChange {
    Idle(bool),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DesktopState {
    pub occupancy: Option<bool>,
}
