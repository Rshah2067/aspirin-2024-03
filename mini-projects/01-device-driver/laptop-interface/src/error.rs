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
    #[error("SP_ERR_ARG: Invalid argument")]
    InvalidArgument,
    #[error("SP_ERR_FAIL: Operation failed")]
    OperationFailed,
    #[error("SP_ERR_MEM: Memory allocation failure")]
    AllocFail,
    #[error("SP_ERR_SUPP: Operation not supported")]
    OperationNotSupported,
    #[error("Timeout")]
    Timeout,
    #[error("Unknown error")]
    Unknown,
    #[error("Failed to open port")]
    ConfigBaudrate,
    #[error("Failed to configure port")]
    ConfigBits,
    #[error("Failed to configure port")]
    ConfigFlowcontrol,
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Serial port error: {0}")]
    SerialError(#[from] SerialError),

    #[error("Failed to send data through channel: {0}")]
    ChannelSendError(#[from] SendError<String>),

    #[error("Received invalid UTF-8 data: {0}")]
    InvalidUtf8Error(#[from] Utf8Error),

    #[error("Error reading from serial port {port}: {error}")]
    SerialReadError { port: String, error: isize },

    #[error("Failed to receive data from channel: {0}")]
    ChannelReceiveError(#[from] RecvTimeoutError),

    #[error("Failed to receive data from try_recv: {0}")]
    ChannelTryRecvError(#[from] TryRecvError),

    #[error("Failed to send message")]
    MessageSendError,

    #[error("Controller not found")]
    ControllerNotFound,

    #[error("Failed to send data through channel: {0}")]
    ChannelSendErrorTuple(#[from] SendError<(u32, String)>),

    #[error("Failed to execute controller command: {0}")]
    CommandError(String),
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
