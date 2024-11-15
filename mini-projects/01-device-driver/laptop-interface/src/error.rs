use thiserror::Error;

#[derive(Error,Debug)]
pub enum SerialError{
    // #[error("Failed to Find Serial With Given Name")]
    // FailedToFind,
    // #[error("Failed to list Ports")]
    // FailedToList,
    #[error("SP_ERR_ARG")]
    ARG,
    #[error("SP_ERR_FAIL")]
    FAIL,
    #[error("SP_ERR_MEM")]
    MEM,
    #[error("SP_ERR_SUPP")]
    SUPP
}