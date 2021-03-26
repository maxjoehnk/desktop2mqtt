use crate::config::Config;
use super::{LocalWorker, Worker};
use futures_util::future::{BoxFuture, LocalBoxFuture};
use futures_util::FutureExt;

pub struct EmptyWorker;

impl LocalWorker for EmptyWorker {
    fn run(&mut self, _: &Config) -> LocalBoxFuture<anyhow::Result<()>> {
        async { Ok(()) }.boxed_local()
    }
}

impl Worker for EmptyWorker {
    fn run(&mut self, _: &Config) -> BoxFuture<anyhow::Result<()>> {
        async { Ok(()) }.boxed()
    }
}
