use crate::config::Config;
use futures_util::future::{BoxFuture, LocalBoxFuture};

mod backlight;
mod empty;
mod home_assistant;
mod idle;
pub mod mqtt;
pub mod state;

pub use self::backlight::*;
pub use self::empty::*;
pub use self::home_assistant::*;
pub use self::idle::*;
pub use self::mqtt::*;
pub use self::state::*;

pub trait Module {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>>;
}

pub trait LocalModule {
    fn run(&mut self, config: &Config) -> LocalBoxFuture<anyhow::Result<()>>;
}
