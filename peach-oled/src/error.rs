use std::error;

use jsonrpc_core::{types::error::Error, ErrorCode};
use linux_embedded_hal as hal;
use snafu::Snafu;

pub type BoxError = Box<dyn error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum OledError {
    #[snafu(display("Failed to create interface for I2C device: {}", source))]
    I2CError {
        source: hal::i2cdev::linux::LinuxI2CError,
    },

    #[snafu(display("Coordinate {} out of range {}: {}", coord, range, value))]
    InvalidCoordinate {
        coord: String,
        range: String,
        value: i32,
    },

    // TODO: implement for validate() in src/lib.rs
    #[snafu(display("Font size invalid: {}", font))]
    InvalidFontSize { font: String },

    #[snafu(display("String length out of range 0-21: {}", len))]
    InvalidString { len: usize },

    #[snafu(display("Missing expected parameter: {}", e))]
    MissingParameter { e: Error },

    #[snafu(display("Failed to parse parameter: {}", e))]
    ParseError { e: Error },
}

impl From<OledError> for Error {
    fn from(err: OledError) -> Self {
        match &err {
            OledError::I2CError { source } => Error {
                code: ErrorCode::ServerError(-32000),
                message: format!("Failed to create interface for I2C device: {}", source),
                data: None,
            },
            OledError::InvalidCoordinate {
                coord,
                value,
                range,
            } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!(
                    "Validation error: coordinate {} out of range {}: {}",
                    coord, range, value
                ),
                data: None,
            },
            OledError::InvalidFontSize { font } => Error {
                code: ErrorCode::ServerError(-32002),
                message: format!("Validation error: {} is not an accepted font size. Use 6x8, 6x12, 8x16 or 12x16 instead", font),
                data: None,
            },
            OledError::InvalidString { len } => Error {
                code: ErrorCode::ServerError(-32003),
                message: format!("Validation error: string length {} out of range 0-21", len),
                data: None,
            },
            OledError::MissingParameter { e } => e.clone(),
            OledError::ParseError { e } => e.clone(),
        }
    }
}
