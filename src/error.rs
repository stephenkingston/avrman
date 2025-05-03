use thiserror::Error;

#[derive(Error, Debug)]
pub enum AvrError {
    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Firmware error: {0}")]
    FirmwareError(String),
}

pub type AvrResult<T> = std::result::Result<T, AvrError>;
