use crate::config::Config;
use crate::modules::StateChange::Idle;
use crate::modules::{LocalModule, StateChange};
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use xidlehook_core::modules::Xcb;
use xidlehook_core::Timer;
use xidlehook_core::Xidlehook;

pub struct IdleModule {
    sender: UnboundedSender<StateChange>,
}

impl IdleModule {
    pub fn new(sender: UnboundedSender<StateChange>) -> Self {
        IdleModule { sender }
    }
}

impl LocalModule for IdleModule {
    fn run(&mut self, config: &Config) -> LocalBoxFuture<anyhow::Result<()>> {
        let idle_timeout = config.idle_timeout;
        async move {
            self.sender.send(StateChange::Idle(false))?;
            let timer = OccupancyTimer {
                sender: self.sender.clone(),
                time: Duration::from_secs(idle_timeout),
            };
            let mut idle_hook = Xidlehook::new(vec![timer]);
            let xcb = Xcb::new().unwrap();
            idle_hook.main_async(&xcb).await.unwrap();

            Ok(())
        }
        .boxed_local()
    }
}

pub struct OccupancyTimer {
    pub time: Duration,
    pub sender: UnboundedSender<StateChange>,
}

impl Timer for OccupancyTimer {
    fn time_left(&mut self, idle_time: Duration) -> xidlehook_core::Result<Option<Duration>> {
        Ok(self
            .time
            .checked_sub(idle_time)
            .filter(|&d| d != Duration::default()))
    }

    fn abort_urgency(&self) -> Option<Duration> {
        Some(Duration::from_secs(5))
    }

    fn activate(&mut self) -> xidlehook_core::Result<()> {
        self.sender.send(Idle(true))?;
        Ok(())
    }

    fn abort(&mut self) -> xidlehook_core::Result<()> {
        self.sender.send(Idle(false))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateChange;
    use tokio::sync::mpsc;

    #[test]
    fn abort_urgency_should_return_5_seconds() {
        let timer = OccupancyTimer {
            time: Default::default(),
            sender: mpsc::unbounded_channel().0,
        };

        let urgency = timer.abort_urgency();

        assert_eq!(Some(Duration::from_secs(5)), urgency);
    }

    #[test]
    fn activate_should_send_idle_true() {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let mut timer = OccupancyTimer {
            time: Default::default(),
            sender,
        };

        timer.activate().unwrap();

        assert_eq!(StateChange::Idle(true), receiver.try_recv().unwrap());
    }

    #[test]
    fn abort_should_send_idle_false() {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let mut timer = OccupancyTimer {
            time: Default::default(),
            sender,
        };

        timer.abort().unwrap();

        assert_eq!(StateChange::Idle(false), receiver.try_recv().unwrap());
    }
}
