use crate::modules::Backlight;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use tokio::fs::File;
use tokio::prelude::*;

const BACKLIGHT_PATH: &str = "/sys/class/backlight/rpi_backlight";
const POWER: &str = "bl_power";
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

        Ok(power != 0)
    }

    async fn set_power(power: bool) -> anyhow::Result<()> {
        let value = if power { 1 } else { 0 };

        Self::set_value(POWER, value).await
    }

    async fn read_brightness() -> anyhow::Result<u32> {
        let actual = Self::read_value(ACTUAL_BRIGHTNESS).await?;

        Ok(actual as u32)
    }

    async fn set_brightness(brightness: u32) -> anyhow::Result<()> {
        Self::set_value(POWER, brightness as i32).await
    }

    async fn read_value(name: &str) -> anyhow::Result<i32> {
        let mut file = Self::open_file(name).await?;
        let value = file.read_i32().await?;

        Ok(value)
    }

    async fn set_value(name: &str, value: i32) -> anyhow::Result<()> {
        let mut file = Self::open_file(name).await?;
        file.write_i32(value).await?;

        Ok(())
    }

    async fn open_file(name: &str) -> anyhow::Result<File> {
        let file_path = format!("{}/{}", BACKLIGHT_PATH, name);
        let file = File::open(&file_path).await?;

        Ok(file)
    }
}
