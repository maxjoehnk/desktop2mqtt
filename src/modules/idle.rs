use crate::config::Config;
use crate::modules::{Module, StateChange};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use user_idle::UserIdle;

pub struct IdleModule {
    sender: UnboundedSender<StateChange>,
}

impl IdleModule {
    pub fn new(sender: UnboundedSender<StateChange>) -> Self {
        IdleModule { sender }
    }
}

impl Module for IdleModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        let idle_timeout = config.idle_timeout;
        let poll_rate = config.idle_poll_rate;
        async move {
            self.sender.send(StateChange::Idle(false))?;

            loop {
                tokio::time::delay_for(Duration::from_secs(poll_rate)).await;
                let idle =
                    UserIdle::get_time().map_err(|err| anyhow::Error::msg(err.to_string()))?;
                if idle.as_seconds() >= idle_timeout {
                    self.sender.send(StateChange::Idle(true))?;
                } else {
                    self.sender.send(StateChange::Idle(false))?;
                }
            }
        }
        .boxed()
    }
}
