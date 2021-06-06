use crate::config::{SensorType, Config};
use systemstat::platform::{PlatformImpl, Platform};
use crate::core::{StateChange, Worker};
use tokio::sync::mpsc::UnboundedSender;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use std::time::Duration;

pub struct SensorsModule {
    sender: UnboundedSender<StateChange>,
}

impl SensorsModule {
    pub fn new(sender: UnboundedSender<StateChange>) -> Self {
        SensorsModule {
            sender,
        }
    }

    pub fn get_sensors(enabled: &[SensorType]) -> anyhow::Result<Vec<Sensor>> {
        let mut sensors = Vec::new();
        for enabled_sensor in enabled {
            let mut meta = enabled_sensor.get_meta();
            sensors.append(&mut meta);
        }
        Ok(sensors)
    }
}

impl Worker for SensorsModule {
    fn run(&mut self, config: &Config) -> BoxFuture<anyhow::Result<()>> {
        if config.modules.sensors.types.is_empty() {
            futures_util::future::ok(()).boxed()
        } else {
            let config = config.modules.sensors.clone();
            async move {
                let handle = tokio::runtime::Handle::try_current()?;
                loop {
                    for sensor_type in &config.types {
                        let sensor_type = sensor_type.clone();
                        let values = handle.spawn_blocking(move || sensor_type.get_values()).await??;
                        for (name, value) in values {
                            self.sender.send(StateChange::Sensor { name, value: (value * 100.0).round() / 100.0  })?;
                        }
                    }
                    tokio::time::sleep(config.poll_rate).await;
                }
            }.boxed()
        }
    }
}

pub struct Sensor {
    pub name: String,
    pub id: String,
    pub class: SensorClass,
    pub icon: Option<String>,
}

pub enum SensorClass {
    Generic,
    Temperature,
    Battery,
}

trait SensorTypeExt {
    fn get_meta(&self) -> Vec<Sensor>;
    fn get_values(&self) -> anyhow::Result<Vec<(String, f32)>>;
}

impl SensorTypeExt for SensorType {
    fn get_meta(&self) -> Vec<Sensor> {
        match self {
            SensorType::CoreTemperature => {
                vec![Sensor {
                    name: "Core Temperature".into(),
                    id: "core_temp".into(),
                    class: SensorClass::Temperature,
                    icon: None,
                }]
            },
            SensorType::Load => {
                vec![Sensor {
                    name: "CPU Load".into(),
                    id: "cpu_load".into(),
                    class: SensorClass::Generic,
                    icon: None,
                }]
            },
            SensorType::Memory => {
                vec![Sensor {
                    name: "Memory Usage".into(),
                    id: "memory_usage".into(),
                    class: SensorClass::Generic,
                    icon: None,
                }]
            },
            SensorType::Battery => {
                vec![Sensor {
                    name: "".into(),
                    id: "battery_usage".into(),
                    class: SensorClass::Battery,
                    icon: None,
                }]
            },
            SensorType::DiskUsage { disks } => {
                disks.iter()
                    .map(|disk| {
                        Sensor {
                            name: format!("Disk Usage {}", disk),
                            id: get_disk_usage_id(disk),
                            class: SensorClass::Generic,
                            icon: Some("mdi:harddisk".into()),
                        }
                    })
                    .collect()
            }
        }

    }

    fn get_values(&self) -> anyhow::Result<Vec<(String, f32)>> {
        let platform = PlatformImpl::new();
        match self {
            SensorType::CoreTemperature => {
                Ok(vec![(
                    "core_temp".into(),
                    platform.cpu_temp()?,
                )])
            }
            SensorType::Load => {
                let load = platform.cpu_load_aggregate()?;
                std::thread::sleep(Duration::from_millis(500));
                Ok(vec![(
                    "cpu_load".into(),
                    load.done()?.user * 100f32
                )])
            }
            SensorType::Memory => {
                let memory = platform.memory()?;
                Ok(vec![(
                    "memory_usage".into(),
                    (1f32 - (memory.free.as_u64() as f64 / memory.total.as_u64() as f64) as f32) * 100f32
                )])
            }
            SensorType::Battery => {
                let battery = platform.battery_life()?;
                Ok(vec![(
                    "battery_usage".into(),
                    battery.remaining_capacity,
                )])
            }
            SensorType::DiskUsage { disks } => {
                let mut mounts = Vec::new();
                for disk in disks {
                    mounts.push(platform.mount_at(disk)?);
                }
                log::trace!("{:#?}", mounts);

                let sensors = mounts.into_iter()
                    .map(|fs| {
                        let usage = fs.free.as_u64() as f64 / fs.total.as_u64() as f64;
                        let usage = ((1f64 - usage) * 100.0) as f32;
                        (get_disk_usage_id(&fs.fs_mounted_on), usage)
                    })
                    .collect();
                Ok(sensors)
            }
        }
    }
}

fn get_disk_usage_id(disk: &str) -> String {
    format!("disk_usage_{}", disk.replace("/", "_"))
}
