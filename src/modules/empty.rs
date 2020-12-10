use crate::config::Config;
use crate::modules::{LocalModule, Module};
use futures_util::future::{BoxFuture, LocalBoxFuture};
use futures_util::FutureExt;

pub struct EmptyModule;

impl LocalModule for EmptyModule {
    fn run(&mut self, _: &Config) -> LocalBoxFuture<anyhow::Result<()>> {
        async { Ok(()) }.boxed_local()
    }
}

impl Module for EmptyModule {
    fn run(&mut self, _: &Config) -> BoxFuture<anyhow::Result<()>> {
        async { Ok(()) }.boxed()
    }
}
