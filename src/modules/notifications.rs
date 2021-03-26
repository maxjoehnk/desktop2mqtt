use crate::core::{Worker, MqttMessage, MqttCommand};
use futures_util::future::BoxFuture;
use crate::config::Config;
use tokio::sync::broadcast;
use notify_rust::Notification;
use futures_util::FutureExt;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

pub struct NotificationsModule {
    receiver: broadcast::Receiver<MqttMessage>,
    sender: UnboundedSender<MqttCommand>,
}

impl NotificationsModule {
    pub fn new(receiver: broadcast::Receiver<MqttMessage>, sender: UnboundedSender<MqttCommand>) -> Self {
        NotificationsModule {
            receiver,
            sender,
        }
    }
}

impl Worker for NotificationsModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let topic = format!("desktop2mqtt/{}/notify", config.hass.entity_id);
        if config.modules.notifications.is_none() {
            return futures_util::future::ok(()).boxed();
        }
        async move {
            self.sender.send(MqttCommand::Subscribe(topic.clone()))?;
            while let Ok(msg) = self.receiver.recv().await {
                if msg.topic != topic {
                    continue;
                }
                let data: NotificationData = msg.deserialize()?;
                let mut notification = Notification::new();
                notification
                    .summary(&data.title)
                    .appname("desktop2mqtt");
                if let Some(message) = data.message {
                    notification.body(&message);
                }
                notification.show()?;
            }

            Ok(())
        }.boxed()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationData {
    #[serde(default)]
    message: Option<String>,
    title: String,
}
