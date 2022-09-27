use crate::config::{ButtonType, Config, CustomCommandConfig};
use crate::core::{MqttCommand, MqttMessage, Worker};
use crate::extensions::StringExt;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use tokio::process::Command;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;

pub struct CustomCommandsModule {
    receiver: broadcast::Receiver<MqttMessage>,
    sender: UnboundedSender<MqttCommand>,
}

impl CustomCommandsModule {
    pub fn new(
        receiver: broadcast::Receiver<MqttMessage>,
        sender: UnboundedSender<MqttCommand>,
    ) -> Self {
        CustomCommandsModule { receiver, sender }
    }

    pub fn get_commands(entity_id: &str, commands: &[CustomCommandConfig]) -> Vec<CustomCommand> {
        commands
            .iter()
            .map(|command| {
                let id = command.name.to_slug();
                CustomCommand {
                    name: command.name.clone(),
                    icon: command.icon.clone(),
                    class: command.button_type.into(),
                    topic: format!("desktop2mqtt/{entity_id}/{id}"),
                    id,
                    command: command.command.clone(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CustomCommand {
    pub name: String,
    pub id: String,
    pub class: ButtonClass,
    pub icon: Option<String>,
    pub topic: String,
    command: String,
}

impl CustomCommand {
    async fn execute(&self) -> anyhow::Result<()> {
        let mut parts = self.command.split(" ");
        let command = parts.next().unwrap();
        let mut child = Command::new(command).args(parts).spawn()?;

        child.wait().await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonClass {
    Generic,
    Restart,
    Update,
}

impl From<Option<ButtonType>> for ButtonClass {
    fn from(button_type: Option<ButtonType>) -> Self {
        match button_type {
            None => Self::Generic,
            Some(ButtonType::Restart) => Self::Restart,
            Some(ButtonType::Update) => Self::Update,
        }
    }
}

impl Worker for CustomCommandsModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        if config.modules.custom_commands.is_empty() {
            return futures_util::future::ok(()).boxed();
        }
        let commands = CustomCommandsModule::get_commands(
            &config.hass.entity_id,
            &config.modules.custom_commands,
        );
        async move {
            for command in &commands {
                self.sender
                    .send(MqttCommand::Subscribe(command.topic.clone()))?;
            }
            while let Ok(msg) = self.receiver.recv().await {
                if let Some(command) = commands.iter().find(|c| c.topic == msg.topic) {
                    let command = command.clone();
                    tokio::spawn(async move { command.execute().await });
                }
            }

            Ok(())
        }
        .boxed()
    }
}
