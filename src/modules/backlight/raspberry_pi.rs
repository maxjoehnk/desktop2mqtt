use crate::modules::Backlight;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use tokio::fs::{File, OpenOptions};
use tokio::prelude::*;

const BACKLIGHT_PATH: &str = "/sys/class/backlight/rpi_backlight";
const POWER: &str = "bl_power";
const BRIGHTNESS: &str = "brightness";
const ACTUAL_BRIGHTNESS: &str = "actual_brightness";
const MAX_BRIGHTNESS: &str = "max_brightness";

pub struct RaspberryPiBacklight;

impl RaspberryPiBacklight {
    pub fn new() -> Self {
        RaspberryPiBacklight
    }
}

impl Backlight for RaspberryPiBacklight {
    fn set_brightness(&mut self, value: u32) -> BoxFuture<anyhow::Result<()>> {
        Self::set_brightness(value).boxed()
    }

    fn get_brightness(&self) -> BoxFuture<anyhow::Result<u32>> {
        Self::read_brightness().boxed()
    }

    fn set_power(&mut self, value: bool) -> BoxFuture<anyhow::Result<()>> {
        Self::set_power(value).boxed()
    }

    fn get_power(&self) -> BoxFuture<anyhow::Result<bool>> {
        Self::read_power().boxed()
    }
}

impl RaspberryPiBacklight {
    async fn read_power() -> anyhow::Result<bool> {
        let power = Self::read_value(POWER).await?;

        Ok(power != 1)
    }

    async fn set_power(power: bool) -> anyhow::Result<()> {
        let value = if power { 0 } else { 1 };

        Self::set_value(POWER, value).await
    }

    async fn read_brightness() -> anyhow::Result<u32> {
        let actual = Self::read_value(ACTUAL_BRIGHTNESS).await?;

        Ok(actual as u32)
    }

    async fn set_brightness(brightness: u32) -> anyhow::Result<()> {
        Self::set_value(BRIGHTNESS, brightness as i32).await
    }

    async fn read_value(name: &str) -> anyhow::Result<i32> {
        let mut file = Self::open_file(name, OpenOptions::new().read(true)).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        log::trace!("Read {} from {}", &content, name);
        let value = content.trim().parse()?;

        Ok(value)
    }

    async fn set_value(name: &str, value: i32) -> anyhow::Result<()> {
        let mut file = Self::open_file(name, OpenOptions::new().write(true)).await?;
        let content = value.to_string();
        log::trace!("Writing {} to {}", &content, name);
        file.write(content.as_bytes()).await?;

        Ok(())
    }

    async fn open_file(name: &str, options: &mut OpenOptions) -> anyhow::Result<File> {
        let file_path = format!("{}/{}", BACKLIGHT_PATH, name);
        log::trace!("Opening file '{}'...", &file_path);
        let file = options.open(&file_path).await?;

        Ok(file)
    }
}
