use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PowerControllerError {
    #[error("Failed to open serial port '{0}': {1}")]
    PortOpenError(String, #[source] serialport::Error),

    #[error("Failed to configure serial port '{0}': {1}")]
    ConfigError(String, #[source] serialport::Error),

    #[error("I/O error during communication: {0}")]
    IoError(#[from] io::Error),

    #[error("Timeout while waiting for device response")]
    Timeout,

    #[error("Invalid device side specified")]
    InvalidDeviceSide,
}

pub type Result<T> = std::result::Result<T, PowerControllerError>;
