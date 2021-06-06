use crate::config::Config;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use tokio::sync::mpsc::UnboundedSender;
use user_idle::UserIdle;
use crate::core::state::StateChange;
use crate::core::Worker;

pub struct IdleModule {
    sender: UnboundedSender<StateChange>,
}

impl IdleModule {
    pub fn new(sender: UnboundedSender<StateChange>) -> Self {
        IdleModule { sender }
    }
}

impl Worker for IdleModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        if let Some(config) = config.modules.idle {
            async move {
                self.sender.send(StateChange::Idle(false))?;

                loop {
                    tokio::time::delay_for(config.poll_rate).await;
                    let idle =
                        UserIdle::get_time().map_err(|err| anyhow::Error::msg(err.to_string()))?;
                    if idle.as_seconds() >= config.timeout.as_secs() {
                        self.sender.send(StateChange::Idle(true))?;
                    } else {
                        self.sender.send(StateChange::Idle(false))?;
                    }
                }
            }
                .boxed()
        }else {
            futures_util::future::ok(()).boxed()
        }
    }
}
