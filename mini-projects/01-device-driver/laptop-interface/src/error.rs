use std::fmt::Debug;
use std::str::Utf8Error;
use std::sync::mpsc::{RecvTimeoutError, SendError, TryRecvError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Serial Error: {0}")]
    SerialError(#[from] SerialError),

    #[error("Controller Error: {0}")]
    ControllerError(#[from] ControllerError),
}

#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Invalid argument")]
    InvalidArgument,

    #[error("Operation failed")]
    OperationFailed,

    #[error("Memory allocation failure")]
    AllocFail,

    #[error("Operation not supported")]
    OperationNotSupported,

    #[error("Timeout occurred")]
    Timeout,

    #[error("Failed to configure baud rate")]
    ConfigBaudrate,

    #[error("Failed to configure data bits")]
    ConfigBits,

    #[error("Failed to configure flow control")]
    ConfigFlowcontrol,

    #[error("Unknown serial error")]
    Unknown,
}

impl From<i32> for SerialError {
    fn from(error: i32) -> Self {
        match error {
            -1 => SerialError::InvalidArgument,
            -2 => SerialError::OperationFailed,
            -3 => SerialError::AllocFail,
            -4 => SerialError::OperationNotSupported,
            _ => SerialError::Unknown,
        }
    }
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Serial port error: {0}")]
    SerialError(#[from] SerialError),

    #[error("Failed to send message to channel: {0}")]
    ChannelSendError(String),

    #[error("Channel receive timeout: {0}")]
    ChannelReceiveTimeoutError(#[from] RecvTimeoutError),

    #[error("Channel receive error: {0}")]
    ChannelTryReceiveError(#[from] TryRecvError),

    #[error("UTF-8 parsing error: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Controller not found")]
    ControllerNotFound,

    #[error("Command execution failed: {0}")]
    CommandError(String),
}

impl<T: Debug> From<SendError<T>> for ControllerError {
    fn from(err: SendError<T>) -> Self {
        ControllerError::ChannelSendError(format!("{:?}", err))
    }
}
