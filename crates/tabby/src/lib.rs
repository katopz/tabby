//! Core tabby functionality. Defines primary API and CLI behavior.
pub mod routes;
pub mod services;

pub mod download;
pub mod serve;

#[cfg(feature = "ee")]
pub mod worker;

#[macro_export]
macro_rules! fatal {
    ($msg:expr) => {
        ({
            tracing::error!($msg);
            std::process::exit(1);
        })
    };

    ($fmt:expr, $($arg:tt)*) => {
        ({
            tracing::error!($fmt, $($arg)*);
            std::process::exit(1);
        })
    };
}

#[derive(clap::ValueEnum, strum::Display, PartialEq, Clone)]
pub enum Device {
    #[strum(serialize = "cpu")]
    Cpu,

    #[cfg(feature = "cuda")]
    #[strum(serialize = "cuda")]
    Cuda,

    #[cfg(feature = "rocm")]
    #[strum(serialize = "rocm")]
    Rocm,

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[strum(serialize = "metal")]
    Metal,

    #[cfg(feature = "experimental-http")]
    #[strum(serialize = "experimental_http")]
    ExperimentalHttp,
}

impl Device {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub fn ggml_use_gpu(&self) -> bool {
        *self == Device::Metal
    }

    #[cfg(feature = "cuda")]
    pub fn ggml_use_gpu(&self) -> bool {
        *self == Device::Cuda
    }

    #[cfg(feature = "rocm")]
    pub fn ggml_use_gpu(&self) -> bool {
        *self == Device::Rocm
    }

    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        feature = "cuda",
        feature = "rocm",
    )))]
    pub fn ggml_use_gpu(&self) -> bool {
        false
    }
}
