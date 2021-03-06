use std::fmt;

use failure::{Backtrace, Context, Fail};

/// Any error that can occur while using `celery`.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// Error kinds that can occur while using `celery`.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    /// You tried to register a task but a task by that name already exists.
    #[fail(display = "Task named '{}' already exists", _0)]
    TaskAlreadyExists(String),

    /// Received an unregistered task.
    #[fail(display = "Received unregistered task named '{}'", _0)]
    UnregisteredTaskError(String),

    /// An AMQP broker error.
    #[fail(display = "AMQP error: {:?}", _0)]
    AMQPError(Option<lapin::Error>),

    /// Raised when broker URL can't be parsed.
    #[fail(display = "Broker URL is invalid: {}", _0)]
    InvalidBrokerUrl(String),

    /// An error occured while serializing or deserializing.
    #[fail(display = "Serialization error: {}", _0)]
    SerializationError(serde_json::Error),

    /// A consumed delivery was in an unknown format.
    #[fail(display = "Failed to parse message: ({})", _0)]
    AMQPMessageParseError(String),

    /// The queue you're attempting to use has not been defined.
    #[fail(display = "Unknown queue '{}'", _0)]
    UnknownQueueError(String),

    /// An error that is expected to happen every once in a while and should trigger
    /// the task to be retried without causes a fit.
    #[fail(display = "{}", _0)]
    ExpectedError(String),

    /// Should be used when a task encounters an error that is unexpected.
    #[fail(display = "{}", _0)]
    UnexpectedError(String),

    /// Should be used when an expired task is received.
    #[fail(display = "Task expired")]
    TaskExpiredError,

    /// Raise when a task should be retried.
    #[fail(display = "Retrying task")]
    Retry,

    /// When a mutex is poisened, for example.
    #[fail(display = "Sync error")]
    SyncError,

    /// An IO error.
    #[fail(display = "An IO error occured ({:?})", _0)]
    IoError(tokio::io::ErrorKind),

    /// Forced shutdown.
    #[fail(display = "Forced shutdown")]
    ForcedShutdown,

    /// Task timed out.
    #[fail(display = "Task timed out")]
    TimeoutError,

    /// Invalid routing glob pattern.
    #[fail(display = "Bad routing rule pattern: {:?}", _0)]
    BadRoutingRulePatternError(Option<String>),

    /// Broker connection failed.
    #[fail(display = "Broker connection error")]
    BrokerConnectionError,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    /// Get the inner `ErrorKind`.
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<Context<&str>> for Error {
    fn from(inner: Context<&str>) -> Error {
        Error {
            inner: Context::new(ErrorKind::UnexpectedError(
                (*inner.get_context()).to_string(),
            )),
        }
    }
}

impl From<lapin::Error> for Error {
    fn from(err: lapin::Error) -> Error {
        Error {
            inner: Context::new(match err {
                lapin::Error::NotConnected => ErrorKind::BrokerConnectionError,
                lapin::Error::ConnectionRefused => ErrorKind::BrokerConnectionError,
                lapin::Error::IOError(_) => ErrorKind::BrokerConnectionError,
                _ => ErrorKind::AMQPError(Some(err)),
            }),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::SerializationError(err)),
        }
    }
}

impl From<tokio::io::Error> for Error {
    fn from(err: tokio::io::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::IoError(err.kind())),
        }
    }
}

impl From<tokio::time::Elapsed> for Error {
    fn from(_err: tokio::time::Elapsed) -> Error {
        Error {
            inner: Context::new(ErrorKind::TimeoutError),
        }
    }
}

impl From<globset::Error> for Error {
    fn from(_err: globset::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::BadRoutingRulePatternError(
                _err.glob().map(|s| s.into()),
            )),
        }
    }
}
