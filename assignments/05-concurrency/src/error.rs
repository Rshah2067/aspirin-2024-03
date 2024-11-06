use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ThreadPoolError {
    #[error("Number of threads must be greater than 0")]
    ZeroThreads,
    #[error("Failed to send job to worker")]
    Send,
    #[error("Requested to many threads, Ran out of System Resources to Allocate")]
    ThreadOverload,
}
