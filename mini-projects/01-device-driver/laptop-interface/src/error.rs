use thiserror::Error;

#[derive(Error,Debug,PartialEq)]
pub enum SerialError{
    #[error("Failed to Find Serial With Given Name")]
    FailedToFind,
    #[error("Failed to list Ports")]
    FailedToList,
}