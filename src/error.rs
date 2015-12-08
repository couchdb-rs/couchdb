use hyper;
use serde_json;
use std;
use url;

use dbtype;

/// Principal error type.
///
/// The public API guarantees only the non-hidden variants and non-hidden
/// variant fields. Do not match against hidden variants or hidden variant
/// fields.
///
#[derive(Debug)]
pub enum Error {

    /// The database already exists.
    DatabaseExists {
        response: ErrorResponse,
    },

    /// JSON-decoding error.
    Decode {
        #[doc(hidden)]
        kind: DecodeKind,
    },

    /// The client request conflicts with an existing document.
    DocumentConflict {
        response: ErrorResponse,
    },

    /// JSON-encoding error.
    Encode {
        #[doc(hidden)]
        cause: serde_json::error::Error,
    },

    /// An internal server error occurred.
    InternalServerError {
        response: ErrorResponse,
    },

    /// The database name is invalid.
    InvalidDatabaseName {
        response: ErrorResponse,
    },

    /// The client request is invalid.
    InvalidRequest {
        response: ErrorResponse,
    },

    /// I/O error with a compile-time description.
    #[doc(hidden)]
    Io {
        description: &'static str,
        cause: std::io::Error,
    },

    /// The CouchDB server responded without a Content-Type header.
    #[doc(hidden)]
    NoContentTypeHeader {
        expected: &'static str,
    },

    /// The resource does not exist.
    NotFound {

        /// CouchDB server response.
        ///
        /// In case of a HEAD request, the response value is None (because the
        /// server doesn't send response content for HEAD requests). Otherwise,
        /// the response value is Some.
        response: Option<ErrorResponse>,
    },

    /// Channel-receiver error with a compile-time description and thread-join
    /// error.
    #[doc(hidden)]
    ReceiveFromThread {
        description: &'static str,
        cause: std::sync::mpsc::RecvError,
    },

    /// HTTP-transport error.
    Transport {
        #[doc(hidden)]
        kind: TransportKind,
    },

    /// The client is unauthorized to carry out the operation.
    Unauthorized {
        response: ErrorResponse,
    },

    /// The CouchDB server responded with a Content-Type header that the client
    /// didn't expect.
    #[doc(hidden)]
    UnexpectedContentTypeHeader {
        expected: &'static str,
        got: String,
    },

    /// The CouchDB server responded with an HTTP status code that the client
    /// didn't expect.
    #[doc(hidden)]
    UnexpectedHttpStatus {
        got: hyper::status::StatusCode,
    },

    /// URI-parse error.
    UriParse {
        #[doc(hidden)]
        cause: url::ParseError,
    },
}

impl std::error::Error for Error {

    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            DatabaseExists { .. } => "The database already exists",
            Decode { .. }  => "The client failed to decode a JSON response from CouchDB server",
            DocumentConflict { .. } => "The client request conflicts with an existing document",
            Encode { .. }  => "The client failed to encode a JSON request to CouchDB server",
            InternalServerError { .. } => "An internal server error occurred",
            InvalidDatabaseName { .. } => "The database name is invalid",
            InvalidRequest { .. } => "The client request is invalid",
            Io { ref description, .. } => description,
            NoContentTypeHeader { .. } =>
                "The CouchDB server responded without a Content-Type header",
            NotFound { .. } => "The resource does not exist",
            ReceiveFromThread { ref description, .. } => description,
            Transport { .. } => "An HTTP transport error occurred",
            Unauthorized { .. } => "The client is unauthorized to carry out the operation",
            UnexpectedContentTypeHeader { .. } =>
                "The CouchDB server responded with a Content-Type header the client did not expect",
            UnexpectedHttpStatus { .. } =>
                "The CouchDB server responded with an HTTP status code the client did not expect",
            UriParse { .. } => "Invalid URI argument",
        }
    }

    fn cause(&self) -> std::option::Option<&std::error::Error> {
        use self::Error::*;
        match *self {
            DatabaseExists { .. } => None,
            Decode { ref kind } => kind.cause(),
            DocumentConflict { .. } => None,
            Encode { ref cause } => Some(cause),
            InternalServerError { .. } => None,
            InvalidDatabaseName { .. } => None,
            InvalidRequest { .. } => None,
            Io { ref cause, .. } => Some(cause),
            NoContentTypeHeader { .. } => None,
            NotFound { .. } => None,
            ReceiveFromThread { ref cause, .. } => Some(cause),
            Transport { ref kind }  => kind.cause(),
            Unauthorized { .. } => None,
            UnexpectedContentTypeHeader { .. }  => None,
            UnexpectedHttpStatus { .. } => None,
            UriParse { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use std::error::Error;
        use self::Error::*;
        match *self {
            DatabaseExists { ref response } =>
                write!(f, "{}: {}", self.description(), response),
            Decode { ref kind } => write!(f, "{}: {}", self.description(), kind),
            DocumentConflict { ref response } =>
                write!(f, "{}: {}", self.description(), response),
            Encode { ref cause } => write!(f, "{}: {}", self.description(), cause),
            InternalServerError { ref response } =>
                write!(f, "{}: {}", self.description(), response),
            InvalidDatabaseName { ref response } =>
                write!(f, "{}: {}", self.description(), response),
            InvalidRequest { ref response } =>
                write!(f, "{}: {}", self.description(), response),
            Io { ref cause, .. } => write!(f, "{}: {}", self.description(), cause),
            NoContentTypeHeader { ref expected } =>
                write!(f, "{}: Expected '{}'", self.description(), expected),
            NotFound { ref response } => match *response {
                Some(ref response) =>
                    write!(f, "{}: {}", self.description(), response),
                None => write!(f, "{}", self.description()),
            },
            ReceiveFromThread { ref cause, .. } =>
                write!(f, "{}: {}", self.description(), cause),
            Transport { ref kind } => write!(f, "{}: {}", self.description(), kind),
            Unauthorized { ref response } => write!(f, "{}: {}", self.description(), response),
            UnexpectedContentTypeHeader { ref expected, ref got } =>
                write!(f, "{}: Expected '{}', got '{}'", self.description(), expected, got),
            UnexpectedHttpStatus { ref got } => write!(f, "{}: Got {}", self.description(), got),
            UriParse { ref cause } => write!(f, "{}: {}", self.description(), cause),
        }
    }
}

/// Response content from the CouchDB server in case of error.
#[derive(Debug)]
pub struct ErrorResponse {

    /// Error string returned by CouchDB Server.
    ///
    /// This is a high-level description of the error—e.g., “file_exists”.

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
            .map_err(|e| {
                Error::Decode { kind: DecodeKind::Serde { cause: e } }
            })
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
pub enum DecodeKind {
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

impl DecodeKind {
    fn cause(&self) -> Option<&std::error::Error> {
        use self::DecodeKind::*;
        match *self {
            InstanceStartTime { ref cause, .. } => Some(cause),
            InvalidDocument { .. } => None,
            Serde { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for DecodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::DecodeKind::*;
        match *self {
            InstanceStartTime { ref got, ref cause } =>
                write!(f, "Could not convert `instance_start_time` field to numeric type: {}: \
                       got {}", cause, got),
            InvalidDocument { ref what } => write!(f, "Unexpected document content: {}", what),
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
