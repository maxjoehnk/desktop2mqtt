use super::Backlight;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Default)]
pub struct StubBacklight {
    power: AtomicBool,
    brightness: AtomicU32,
}

impl StubBacklight {
    pub fn new() -> Self {
        StubBacklight {
            power: AtomicBool::new(true),
            brightness: AtomicU32::new(255),
        }
    }
}

impl Backlight for StubBacklight {
    fn set_brightness(&mut self, value: u32) -> BoxFuture<anyhow::Result<()>> {
        async move {
            log::info!("[Stub] Setting brightness {}", value);
            self.brightness.store(value, Ordering::Relaxed);

            Ok(())
        }
        .boxed()
    }

    fn get_brightness(&self) -> BoxFuture<anyhow::Result<u32>> {
        async move {
            let brightness = self.brightness.fetch_or(255, Ordering::Relaxed);

            Ok(brightness)
        }
        .boxed()
    }

    fn set_power(&mut self, value: bool) -> BoxFuture<anyhow::Result<()>> {
        async move {
            log::info!("[Stub] Setting power {}", value);
            self.power.store(value, Ordering::Relaxed);

            Ok(())
        }
        .boxed()
    }

    fn get_power(&self) -> BoxFuture<anyhow::Result<bool>> {
        async move {
            let power = self.power.fetch_or(true, Ordering::Relaxed);

            Ok(power)
        }
        .boxed()
    }
}
