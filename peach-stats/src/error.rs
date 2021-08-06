use std::{error, io};

use jsonrpc_core::{types::error::Error, ErrorCode};
use probes::ProbeError;
use serde_json::Error as SerdeError;
use snafu::Snafu;

pub type BoxError = Box<dyn error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum StatError {
    #[snafu(display("Failed to retrieve CPU statistics: {}", source))]
    ReadCpuStat { source: ProbeError },

    #[snafu(display("Failed to retrieve disk usage statistics: {}", source))]
    ReadDiskUsage { source: ProbeError },

    #[snafu(display("Failed to retrieve load average statistics: {}", source))]
    ReadLoadAvg { source: ProbeError },

    #[snafu(display("Failed to retrieve memory statistics: {}", source))]
    ReadMemStat { source: ProbeError },

    #[snafu(display("Failed to retrieve system uptime: {}", source))]
    ReadUptime { source: io::Error },

    #[snafu(display("JSON serialization failed: {}", source))]
    SerdeSerialize { source: SerdeError },
}

impl From<StatError> for Error {
    fn from(err: StatError) -> Self {
        match &err {
            StatError::ReadCpuStat { source } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!("Failed to retrieve CPU statistics: {}", source),
                data: None,
            },
            StatError::ReadDiskUsage { source } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!("Failed to retrieve disk usage statistics: {}", source),
                data: None,
            },
            StatError::ReadLoadAvg { source } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!("Failed to retrieve load average statistics: {}", source),
                data: None,
            },
            StatError::ReadMemStat { source } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!("Failed to retrieve memory statistics: {}", source),
                data: None,
            },
            StatError::ReadUptime { source } => Error {
                code: ErrorCode::ServerError(-32001),
                message: format!("Failed to retrieve system uptime: {}", source),
                data: None,
            },
            StatError::SerdeSerialize { source } => Error {
                code: ErrorCode::ServerError(-32002),
                message: format!("JSON serialization failed: {}", source),
                data: None,
            },
        }
    }
}
