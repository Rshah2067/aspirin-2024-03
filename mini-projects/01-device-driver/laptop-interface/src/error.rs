use thiserror::Error;
use std::sync::mpsc::{RecvTimeoutError, SendError, TryRecvError};
use std::str::Utf8Error;
#[derive(Error, Debug)]
pub enum SerialError {
    #[error("SP_ERR_ARG: Invalid argument")]
    ARG,
    #[error("SP_ERR_FAIL: Operation failed")]
    FAIL,
    #[error("SP_ERR_MEM: Memory allocation failure")]
    MEM,
    #[error("SP_ERR_SUPP: Operation not supported")]
    SUPP,
    #[error("Timeout")]
    Timeout,
    #[error("Unknown error")]
    Unknown,
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
}

impl From<i32> for SerialError {
    fn from(error: i32) -> Self {
        match error {
            -1 => SerialError::ARG,
            -2 => SerialError::FAIL,
            -3 => SerialError::MEM,
            -4 => SerialError::SUPP,
            _ => SerialError::Unknown,
        }
    }
}
