use crate::options::CliOptions;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};

const DEFAULT_POLL_RATE: u64 = 5;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub hass: HomeAssistantConfig,
    pub idle_timeout: u64,
    #[serde(default = "default_poll_rate")]
    pub idle_poll_rate: u64,
    #[serde(default)]
    pub backlight: BacklightProvider,
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
    None,
    RaspberryPi,
    Stub,
}

impl Default for BacklightProvider {
    fn default() -> Self {
        BacklightProvider::None
    }
}

fn default_poll_rate() -> u64 {
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
