use std;
use std::borrow::Cow;

/// `Error` is the principal type of the `couchdb` crate.
#[derive(Debug)]
pub enum Error {
    BadDesignDocumentId,

    #[doc(hidden)]
    BadDigest,

    #[doc(hidden)]
    BadPath { what: &'static str },

    BadRevision,

    #[doc(hidden)]
    Io {
        what: Cow<'static, str>,
        cause: std::io::Error,
    },
}

impl Error {
    #[doc(hidden)]
    pub fn bad_path(what: &'static str) -> Self {
        Error::BadPath { what: what }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let d = std::error::Error::description(self);
        match *self {
            Error::BadPath { what } => write!(f, "{}: {}", d, what),
            Error::Io { ref cause, .. } => write!(f, "{}: {}", d, cause),
            _ => f.write_str(d),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadDesignDocumentId => "The string is not a valid CouchDB design document id",
            Error::BadDigest => "The string is not a valid CouchDB attachment digest",
            Error::BadPath { .. } => "The CouchDB path is not valid",
            Error::BadRevision => "The string is not a valid CouchDB document revision",
            Error::Io { ref what, .. } => what.as_ref(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io { ref cause, .. } => Some(cause),
            _ => None,
        }
    }
}

impl<T: Into<Cow<'static, str>>> From<(T, std::io::Error)> for Error {
    fn from((what, cause): (T, std::io::Error)) -> Self {
        Error::Io {
            what: what.into(),
            cause: cause,
        }
    }
}
