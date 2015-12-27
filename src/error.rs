use hyper;
use serde_json;
use std;
use url;

use dbtype;

/// Principal error type.
///
/// The public API provides no guarantees for hidden variants. Applications
/// should not match against hidden variants.
///
#[derive(Debug)]
pub enum Error {
    /// The database already exists.
    DatabaseExists(ErrorResponse),

    // JSON-decoding error.
    #[doc(hidden)]
    Decode(DecodeErrorKind),

    /// The client request conflicts with an existing document.
    DocumentConflict(ErrorResponse),

    // JSON-encoding error.
    #[doc(hidden)]
    Encode(EncodeErrorKind),

    /// An internal server error occurred.
    InternalServerError(ErrorResponse),

    /// The database name is invalid.
    InvalidDatabaseName(ErrorResponse),

    /// The client request is invalid.
    InvalidRequest(ErrorResponse),

    // I/O error with a compile-time description.
    #[doc(hidden)]
    Io {
        cause: std::io::Error,
        description: &'static str,
    },

    // The CouchDB server responded without a Content-Type header.
    #[doc(hidden)]
    NoContentTypeHeader {
        expected: &'static str,
    },

    /// The resource does not exist.
    ///
    /// In case of a HEAD request, the response value is `None` (because the
    /// server doesn't send response content for HEAD requests). For all other
    /// request methods, the response value is `Some`.
    ///
    NotFound(Option<ErrorResponse>),

    // Channel-receiver error with a compile-time description and thread-join
    // error.
    #[doc(hidden)]
    ReceiveFromThread {
        cause: std::sync::mpsc::RecvError,
        description: &'static str,
    },

    // HTTP-transport error.
    #[doc(hidden)]
    Transport(TransportKind),

    /// The client is unauthorized to carry out the operation.
    Unauthorized(ErrorResponse),

    // The CouchDB server responded with a Content-Type header that the client
    // didn't expect.
    #[doc(hidden)]
    UnexpectedContentTypeHeader {
        expected: &'static str,
        got: String,
    },

    // The CouchDB server responded with an HTTP status code that the client
    // didn't expect.
    #[doc(hidden)]
    UnexpectedHttpStatus {
        got: hyper::status::StatusCode,
    },

    // URI-parse error.
    #[doc(hidden)]
    UriParse {
        cause: url::ParseError,
    },
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            DatabaseExists(..) => "The database already exists",
            Decode(..) => "The client failed to decode a JSON response from the CouchDB server",
            DocumentConflict(..) => "The client request conflicts with an existing document",
            Encode(..) => "The client failed to encode a JSON request to the CouchDB server",
            InternalServerError(..) => "An internal server error occurred",
            InvalidDatabaseName(..) => "The database name is invalid",
            InvalidRequest(..) => "The client request is invalid",
            Io{ ref description, .. } => description,
            NoContentTypeHeader { .. } => "The CouchDB server responded without a Content-Type header",
            NotFound(..) => "The resource does not exist",
            ReceiveFromThread { ref description, .. } => description,
            Transport(..) => "An HTTP transport error occurred",
            Unauthorized(..) => "The client is unauthorized to carry out the operation",
            UnexpectedContentTypeHeader { .. } => {
                "The CouchDB server responded with a Content-Type header the client did not expect"
            }
            UnexpectedHttpStatus { .. } => {
                "The CouchDB server responded with an HTTP status code the client did not expect"
            }
            UriParse { .. } => "Invalid URI argument",
        }
    }

    fn cause(&self) -> std::option::Option<&std::error::Error> {
        use self::Error::*;
        match *self {
            DatabaseExists(..) => None,
            Decode(ref kind) => kind.cause(),
            DocumentConflict(..) => None,
            Encode(ref kind) => kind.cause(),
            InternalServerError(..) => None,
            InvalidDatabaseName(..) => None,
            InvalidRequest(..) => None,
            Io{ref cause, ..} => Some(cause),
            NoContentTypeHeader { .. } => None,
            NotFound(..) => None,
            ReceiveFromThread { ref cause, .. } => Some(cause),
            Transport(ref kind) => kind.cause(),
            Unauthorized(..) => None,
            UnexpectedContentTypeHeader { .. } => None,
            UnexpectedHttpStatus { .. } => None,
            UriParse { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::Error::*;
        let d = {
            use std::error::Error;
            self.description()
        };
        match *self {
            DatabaseExists(ref response) => write!(f, "{}: {}", d, response),
            Decode(ref kind) => write!(f, "{}: {}", d, kind),
            DocumentConflict(ref response) => write!(f, "{}: {}", d, response),
            Encode(ref kind) => write!(f, "{}: {}", d, kind),
            InternalServerError(ref response) => write!(f, "{}: {}", d, response),
            InvalidDatabaseName(ref response) => write!(f, "{}: {}", d, response),
            InvalidRequest(ref response) => write!(f, "{}: {}", d, response),
            Io{ref cause, ..} => write!(f, "{}: {}", d, cause),
            NoContentTypeHeader { ref expected } => write!(f, "{}: Expected '{}'", d, expected),
            NotFound(ref response) => {
                match *response {
                    Some(ref response) => write!(f, "{}: {}", d, response),
                    None => write!(f, "{}", d),
                }
            }
            ReceiveFromThread { ref cause, .. } => write!(f, "{}: {}", d, cause),
            Transport(ref kind) => write!(f, "{}: {}", d, kind),
            Unauthorized(ref response) => write!(f, "{}: {}", d, response),
            UnexpectedContentTypeHeader { ref expected, ref got } => {
                write!(f, "{}: Expected '{}', got '{}'", d, expected, got)
            }
            UnexpectedHttpStatus { ref got } => write!(f, "{}: Got {}", d, got),
            UriParse { ref cause } => write!(f, "{}: {}", d, cause),
        }
    }
}

/// Response content from the CouchDB server in case of error.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorResponse {
    /// Error string returned by CouchDB Server.
    ///
    /// This is the high-level name of the error—e.g., “file_exists”.
    ///
    pub error: String,

    /// Reason string returned by CouchDB Server.
    ///
    /// This is a low-level description of the error—e.g., “The database could
    /// not be created, the file already exists.”
    ///
    pub reason: String,
}

impl ErrorResponse {
    pub fn from_reader<R>(r: R) -> Result<Self, Error>
        where R: std::io::Read
    {
        serde_json::from_reader::<_, dbtype::ErrorResponse>(r)
            .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
            .map(|x| {
                ErrorResponse {
                    error: x.error,
                    reason: x.reason,
                }
            })
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.error, self.reason)
    }
}

#[derive(Debug)]
pub enum DecodeErrorKind {
    InstanceStartTime {
        got: String,
        cause: std::num::ParseIntError,
    },
    InvalidDocument {
        what: &'static str,
    },
    Serde {
        cause: serde_json::Error,
    },
}

impl DecodeErrorKind {
    fn cause(&self) -> Option<&std::error::Error> {
        use self::DecodeErrorKind::*;
        match *self {
            InstanceStartTime { ref cause, .. } => Some(cause),
            InvalidDocument { .. } => None,
            Serde { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::DecodeErrorKind::*;
        match *self {
            InstanceStartTime { ref got, ref cause } => {
                write!(f,
                       "Could not convert `instance_start_time` field to numeric type: {}: got {}",
                       cause,
                       got)
            }
            InvalidDocument { ref what } => write!(f, "Unexpected document content: {}", what),
            Serde { ref cause } => cause.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum EncodeErrorKind {
    Serde {
        cause: serde_json::Error,
    },
}

impl EncodeErrorKind {
    fn cause(&self) -> Option<&std::error::Error> {
        use self::EncodeErrorKind::*;
        match *self {
            Serde { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for EncodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::EncodeErrorKind::*;
        match *self {
            Serde { ref cause } => cause.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum TransportKind {
    Hyper(hyper::error::Error),
    Io(std::io::Error),
}

impl TransportKind {
    fn cause(&self) -> std::option::Option<&std::error::Error> {
        use self::TransportKind::*;
        match *self {
            Hyper(ref e) => Some(e),
            Io(ref e) => Some(e),
        }
    }
}

impl std::fmt::Display for TransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::TransportKind::*;
        match *self {
            Hyper(ref e) => e.fmt(f),
            Io(ref e) => e.fmt(f),
        }
    }
}
