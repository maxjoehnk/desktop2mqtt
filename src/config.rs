use crate::options::CliOptions;
use directories_next::ProjectDirs;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_POLL_RATE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub hass: HomeAssistantConfig,
    #[serde(default)]
    pub modules: Modules
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Modules {
    #[serde(default)]
    pub backlight: Option<BacklightProvider>,
    #[serde(default)]
    pub idle: Option<IdleModuleConfig>,
    #[serde(default)]
    // TODO: add configuration options for icon and app name
    pub notifications: Option<bool>,
    #[serde(default)]
    pub sensors: SensorsConfig,
}

#[derive(Default, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SensorsConfig {
    /// Sensor poll rate in seconds
    #[serde(default = "sensor_poll_rate", with = "humantime_serde")]
    pub poll_rate: Duration,
    pub types: Vec<SensorType>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum SensorType {
    Load,
    Memory,
    Battery,
    CoreTemperature,
    DiskUsage {
        #[serde(default)]
        disks: Vec<String>,
    },
}

fn sensor_poll_rate() -> Duration {
    Duration::from_secs(1)
}

#[derive(Debug, Clone, Deserialize, Copy, PartialEq, Eq)]
pub struct IdleModuleConfig {
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default = "default_poll_rate", with = "humantime_serde")]
    pub poll_rate: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAssistantConfig {
    pub entity_id: String,
    pub name: String,
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BacklightProvider {
    RaspberryPi,
    Stub,
}

fn default_poll_rate() -> Duration {
    DEFAULT_POLL_RATE
}

pub(crate) fn get_config(options: &CliOptions) -> anyhow::Result<Config> {
    let path = get_config_file_path(options);
    log::debug!("Loading config file from {:?}", &path);
    let config_file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(&config_file)?;

    Ok(config)
}

fn get_config_file_path(options: &CliOptions) -> PathBuf {
    let default_file = Path::new("config.yml");
    let user_dir_file = get_user_dir_path();

    match (&options.config, default_file, user_dir_file) {
        (Some(config), _, _) => config.clone(),
        (None, config, _) if config.exists() => config.to_path_buf(),
        (None, _, Some(config)) if config.exists() => config,
        _ => panic!("No config file found"),
    }
}

fn get_user_dir_path() -> Option<PathBuf> {
    if let Some(project_dirs) = ProjectDirs::from("me", "maxjoehnk", "desktop2mqtt") {
        let config_dir = project_dirs.config_dir();
        Some(config_dir.join("config.yml"))
    } else {
        None
    }
}
