use std::error::Error;
use std::fmt;
use std::io;

use radia_math::MathError;

#[derive(Debug)]
pub enum RenderError {
    AdapterRequest(String),
    DeviceRequest(String),
    UnsupportedAdapter(String),
    UnsupportedFormat(String),
    GpuValidation(String),
    DeviceLost(String),
    BufferMap(String),
    DevicePoll(String),
    InvalidConfig(String),
    Math(MathError),
    Io(io::Error),
    PngEncoding(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterRequest(message) => {
                write!(formatter, "Vulkan adapter request failed: {message}")
            }
            Self::DeviceRequest(message) => {
                write!(formatter, "WGPU device request failed: {message}")
            }
            Self::UnsupportedAdapter(message) => {
                write!(formatter, "unsupported adapter: {message}")
            }
            Self::UnsupportedFormat(message) => {
                write!(formatter, "unsupported texture format: {message}")
            }
            Self::GpuValidation(message) => write!(formatter, "GPU validation failed: {message}"),
            Self::DeviceLost(message) => write!(formatter, "GPU device lost: {message}"),
            Self::BufferMap(message) => write!(formatter, "GPU readback map failed: {message}"),
            Self::DevicePoll(message) => write!(formatter, "GPU device poll failed: {message}"),
            Self::InvalidConfig(message) => write!(formatter, "invalid render config: {message}"),
            Self::Math(error) => write!(formatter, "math error: {error}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::PngEncoding(message) => write!(formatter, "PNG encoding failed: {message}"),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Math(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<MathError> for RenderError {
    fn from(error: MathError) -> Self {
        Self::Math(error)
    }
}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
