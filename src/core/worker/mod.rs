mod empty;

pub use self::empty::*;

use crate::config::Config;
use futures_util::future::{BoxFuture, LocalBoxFuture};

pub trait Worker {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>>;
}

pub trait LocalWorker {
    fn run(&mut self, config: &Config) -> LocalBoxFuture<anyhow::Result<()>>;
}

