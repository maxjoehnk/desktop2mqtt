use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use mqtt_async_client::client::{Client, Publish};
use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::config::Config;
use crate::modules::Module;

pub struct MqttModule {
    receiver: UnboundedReceiver<MqttMessage>,
}

impl MqttModule {
    pub fn new(receiver: UnboundedReceiver<MqttMessage>) -> Self {
        MqttModule { receiver }
    }
}

impl Module for MqttModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let mqtt_config = config.mqtt.clone();
        async move {
            let mut client = Client::builder().set_host(mqtt_config.url).build()?;

            client.connect().await?;

            while let Some(msg) = self.receiver.recv().await {
                log::debug!("Publishing mqtt message {:?}...", &msg);
                let mut publish = Publish::from(msg);
                publish.set_retain(true);

                client.publish(&publish).await?;
            }

            Ok(())
        }
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
}

impl MqttMessage {
    pub fn new(topic: String, payload: String) -> Self {
        MqttMessage { topic, payload }
    }

    pub fn new_json<TPayload: Serialize>(
        topic: String,
        payload: &TPayload,
    ) -> anyhow::Result<Self> {
        let payload = serde_json::to_string(&payload)?;

        Ok(MqttMessage { topic, payload })
    }
}

impl From<MqttMessage> for Publish {
    fn from(msg: MqttMessage) -> Self {
        Publish::new(msg.topic, msg.payload.into_bytes())
    }
}
