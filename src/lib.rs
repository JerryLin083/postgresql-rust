use std::fmt;

pub mod cmd;
pub mod conncetion;
pub mod frame;
pub mod pg_pool;

#[derive(Debug)]
pub enum Error {
    IoError(tokio::io::Error),
    PgError(tokio_postgres::Error),
    AcquireError(tokio::sync::AcquireError),
    EnvVarError(std::env::VarError),
    ConnectionError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::PgError(e) => write!(f, "PostgresSQL error: {}", e),
            Error::AcquireError(e) => write!(f, "Semaphore acquire error: {}", e),
            Error::EnvVarError(e) => write!(f, "Env variable error: {}", e),
            Error::ConnectionError => write!(f, "Failed to establish database connection"),
        }
    }
}

impl From<tokio::io::Error> for Error {
    fn from(err: tokio::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        Error::PgError(err)
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::EnvVarError(err)
    }
}

impl From<tokio::sync::AcquireError> for Error {
    fn from(err: tokio::sync::AcquireError) -> Self {
        Error::AcquireError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
