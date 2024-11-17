use thiserror::Error;

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
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Serial port error: {0}")]
    SerialError(#[from] SerialError),
    
    #[error("Failed to send data through channel: {0}")]
    ChannelSendError(#[from] std::sync::mpsc::SendError<String>),
    
    #[error("Received invalid UTF-8 data: {0}")]
    InvalidUtf8Error(#[from] std::str::Utf8Error),
    
    #[error("Error reading from serial port {port}: {error}")]
    SerialReadError { port: String, error: isize },
    
    #[error("Failed to receive data from channel: {0}")]
    ChannelReceiveError(#[from] std::sync::mpsc::RecvTimeoutError),
}

impl From<isize> for SerialError {
    fn from(error: isize) -> Self {
        match error {
            -1 => SerialError::ARG,
            -2 => SerialError::FAIL,
            -3 => SerialError::MEM,
            -4 => SerialError::SUPP,
            _ => SerialError::FAIL,
        }
    }
}
