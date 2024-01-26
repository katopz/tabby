use std::env::consts::ARCH;

use anyhow::Result;
use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};
use strum::Display;
use sysinfo::{CpuExt, System, SystemExt};
use utoipa::ToSchema;

use crate::Device;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, ToSchema)]
pub struct HealthState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_model: Option<String>,
    pub device: String,
    pub arch: String,
    pub cpu_info: String,
    pub cpu_count: usize,
    pub cuda_devices: Vec<String>,
    pub version: Version,
}

impl HealthState {
    pub fn new(model: Option<&str>, chat_model: Option<&str>, device: &Device) -> Self {
        let (cpu_info, cpu_count) = read_cpu_info();

        let cuda_devices = match read_cuda_devices() {
            Ok(s) => s,
            Err(_) => vec![],
        };

        Self {
            model: model.map(|x| x.to_owned()),
            chat_model: chat_model.map(|x| x.to_owned()),
            device: device.to_string(),
            arch: ARCH.to_string(),
            cpu_info,
            cpu_count,
            cuda_devices,
            version: Version::new(),
        }
    }
}

pub fn read_cpu_info() -> (String, usize) {
    let mut system = System::new_all();
    system.refresh_cpu();
    let cpus = system.cpus();
    let count = cpus.len();
    let info = if count > 0 {
        let cpu = &cpus[0];
        cpu.brand().to_string()
    } else {
        "unknown".to_string()
    };

    (info, count)
}

pub fn read_cuda_devices() -> Result<Vec<String>> {
    // In cases of MacOS or docker containers where --gpus are not specified,
    // the Nvml::init() would return an error. In these scenarios, we
    // assign cuda_devices to be empty, indicating that the current runtime
    // environment does not support cuda interface.
    let nvml = Nvml::init()?;
    let mut cuda_devices = vec![];
    let device_count = nvml.device_count()?;
    for i in 0..device_count {
        let name = nvml.device_by_index(i)?.name()?;
        cuda_devices.push(name);
    }
    Ok(cuda_devices)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, ToSchema)]
pub struct Version {
    pub build_date: String,
    pub build_timestamp: String,
    pub git_sha: String,
    pub git_describe: String,
}

impl Version {
    fn new() -> Self {
        Self {
            build_date: env!("VERGEN_BUILD_DATE").to_string(),
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
            git_sha: env!("VERGEN_GIT_SHA").to_string(),
            git_describe: env!("VERGEN_GIT_DESCRIBE").to_string(),
        }
    }
}
