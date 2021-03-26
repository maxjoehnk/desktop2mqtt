use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use mqtt_async_client::client::{Client, Publish, QoS, ReadResult, Subscribe, SubscribeTopic};
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedReceiver;

use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use crate::config::Config;
use crate::core::Worker;

pub struct MqttWorker<'a> {
    client: &'a mut Client,
    receiver: UnboundedReceiver<MqttCommand>,
    sender: broadcast::Sender<MqttMessage>,
}

impl<'a> MqttWorker<'a> {
    pub fn new(
        client: &'a mut Client,
        receiver: UnboundedReceiver<MqttCommand>,
        sender: broadcast::Sender<MqttMessage>,
    ) -> Self {
        MqttWorker {
            client,
            receiver,
            sender,
        }
    }
}

impl<'a> Worker for MqttWorker<'a> {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let entity_id = config.hass.entity_id.clone();
        async move {
            self.publish(MqttMessage {
                topic: format!("desktop2mqtt/{}/availability", entity_id),
                payload: "online".to_string(),
            })
            .await?;
            loop {
                tokio::select! {
                    Some(msg) = self.receiver.recv() => {
                        match msg {
                            MqttCommand::Subscribe(topic) => self.subscribe(topic).await?,
                            MqttCommand::Emit(msg) => self.publish(msg).await?,
                        }
                    }
                    msg = self.client.read_subscriptions() => {
                        Self::recv(msg, &self.sender).await?;
                    },
                    else => break
                }
            }

            Ok(())
        }
        .boxed()
    }
}

impl<'a> MqttWorker<'a> {
    async fn publish(&self, msg: MqttMessage) -> anyhow::Result<()> {
        log::debug!("Publishing mqtt message {:?}...", &msg);
        let mut publish = Publish::from(msg);
        publish.set_retain(true);

        self.client.publish(&publish).await?;

        Ok(())
    }

    async fn subscribe(&mut self, topic: String) -> anyhow::Result<()> {
        log::debug!("Subscribing to mqtt topic {}...", &topic);
        let topic = SubscribeTopic {
            topic_path: topic,
            qos: QoS::AtLeastOnce,
        };
        let subscription = Subscribe::new(vec![topic]);
        self.client.subscribe(subscription).await?;

        Ok(())
    }

    async fn recv(
        msg: mqtt_async_client::Result<ReadResult>,
        sender: &broadcast::Sender<MqttMessage>,
    ) -> anyhow::Result<()> {
        let msg = msg?;
        let msg = MqttMessage::try_from(msg)?;

        log::info!("{:?}", msg);

        sender.send(msg).unwrap();

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MqttCommand {
    Emit(MqttMessage),
    Subscribe(String),
}

#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
}

impl MqttCommand {
    pub fn new_json<TPayload: Serialize>(
        topic: String,
        payload: &TPayload,
    ) -> anyhow::Result<Self> {
        let payload = serde_json::to_string(&payload)?;

        Ok(MqttCommand::Emit(MqttMessage { topic, payload }))
    }

    pub fn subscribe(topic: String) -> Self {
        MqttCommand::Subscribe(topic)
    }
}

impl From<MqttMessage> for Publish {
    fn from(msg: MqttMessage) -> Self {
        Publish::new(msg.topic, msg.payload.into_bytes())
    }
}

impl TryFrom<ReadResult> for MqttMessage {
    type Error = anyhow::Error;

    fn try_from(msg: ReadResult) -> anyhow::Result<Self> {
        let payload = String::from_utf8(msg.payload().to_vec())?;

        Ok(MqttMessage {
            payload,
            topic: msg.topic().to_string(),
        })
    }
}

impl MqttMessage {
    pub fn deserialize<TPayload: DeserializeOwned>(&self) -> anyhow::Result<TPayload> {
        let payload = serde_json::from_str(&self.payload)?;

        Ok(payload)
    }
}
