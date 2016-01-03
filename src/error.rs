use hyper;
use serde_json;
use std;
use url;

use ErrorResponse;

/// Principal error type.
///
/// An `Error` specifies an error originating from or propagated by the couchdb
/// crate.
///
/// Error variants with a value of the type `Option<ErrorResponse>` signify
/// error responses from the CouchDB server. The value for these variants is
/// `None` if the request causing the error was a HEAD request, and `Some`
/// otherwise. This reflects how the server returns no detailed error
/// information for HEAD requests.
///
#[derive(Debug)]
pub enum Error {
    /// The database path is in an invalid format.
    BadDatabasePath(BadPathKind),

    /// The design document path is in an invalid format.
    BadDesignDocumentPath(BadPathKind),

    /// The document path is in an invalid format.
    BadDocumentPath(BadPathKind),

    // The MD5 hash is in an invalid format.
    #[doc(hidden)]
    BadMd5Hash,

    /// The client request is invalid.
    BadRequest(Option<ErrorResponse>),

    /// The revision is in an invalid format.
    BadRevision,

    /// The view path is in an invalid format.
    BadViewPath(BadPathKind),

    /// The database already exists.
    DatabaseExists(Option<ErrorResponse>),

    // JSON-decoding error.
    #[doc(hidden)]
    Decode(DecodeErrorKind),

    /// The client request conflicts with an existing document.
    DocumentConflict(Option<ErrorResponse>),

    // JSON-encoding error.
    #[doc(hidden)]
    Encode(EncodeErrorKind),

    /// An internal server error occurred.
    InternalServerError(Option<ErrorResponse>),

    // The string has an invalid percent-encoding.
    #[doc(hidden)]
    InvalidPercentEncoding {
        encoding: String,
    },

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
    ///
    /// In case of a HEAD request, the response value is `None` (because the
    /// server doesn't send response content for HEAD requests). For all other
    /// request methods, the response value is `Some`.
    ///
    Unauthorized(Option<ErrorResponse>),

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
            BadDatabasePath(..) => "The database path is in an invalid format",
            BadDesignDocumentPath(..) => "The design document path is in an invalid format",
            BadDocumentPath(..) => "The document path is in an invalid format",
            BadMd5Hash => "The MD5 hash is in an invalid format",
            BadRequest(..) => "The client request is invalid",
            BadRevision => "The revision is in an invalid format",
            BadViewPath(..) => "The view path is in an invalid format",
            DatabaseExists(..) => "The database already exists",
            Decode(..) => "The client failed to decode a JSON response from the CouchDB server",
            DocumentConflict(..) => "The client request conflicts with an existing document",
            Encode(..) => "The client failed to encode a JSON request to the CouchDB server",
            InternalServerError(..) => "An internal server error occurred",
            InvalidPercentEncoding { .. } => "The string has an invalid percent-encoding",
            Io{ ref description, .. } => description,
            NoContentTypeHeader { .. } => {
                "The CouchDB server responded without a Content-Type header"
            }
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
            BadDatabasePath(ref kind) => kind.cause(),
            BadDesignDocumentPath(ref kind) => kind.cause(),
            BadDocumentPath(ref kind) => kind.cause(),
            BadMd5Hash => None,
            BadRequest(..) => None,
            BadRevision => None,
            BadViewPath(ref kind) => kind.cause(),
            DatabaseExists(..) => None,
            Decode(ref kind) => kind.cause(),
            DocumentConflict(..) => None,
            Encode(ref kind) => kind.cause(),
            InternalServerError(..) => None,
            InvalidPercentEncoding { .. } => None,
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
            BadDatabasePath(ref kind) => write!(f, "{}: {}", d, kind),
            BadDesignDocumentPath(ref kind) => write!(f, "{}: {}", d, kind),
            BadDocumentPath(ref kind) => write!(f, "{}: {}", d, kind),
            BadMd5Hash => write!(f, "{}", d),
            BadRequest(None) => write!(f, "{}", d),
            BadRequest(Some(ref response)) => write!(f, "{}: {}", d, response),
            BadRevision => write!(f, "{}", d),
            BadViewPath(ref kind) => write!(f, "{}: {}", d, kind),
            DatabaseExists(None) => write!(f, "{}", d),
            DatabaseExists(Some(ref response)) => write!(f, "{}: {}", d, response),
            Decode(ref kind) => write!(f, "{}: {}", d, kind),
            DocumentConflict(None) => write!(f, "{}", d),
            DocumentConflict(Some(ref response)) => write!(f, "{}: {}", d, response),
            Encode(ref kind) => write!(f, "{}: {}", d, kind),
            InternalServerError(None) => write!(f, "{}", d),
            InternalServerError(Some(ref response)) => write!(f, "{}: {}", d, response),
            InvalidPercentEncoding{ref encoding} => write!(f, "{}: Got '{}'", d, encoding),
            Io{ref cause, ..} => write!(f, "{}: {}", d, cause),
            NoContentTypeHeader{ref expected} => write!(f, "{}: Expected '{}'", d, expected),
            NotFound(None) => write!(f, "{}", d),
            NotFound(Some(ref response)) => write!(f, "{}: {}", d, response),
            ReceiveFromThread{ref cause, ..} => write!(f, "{}: {}", d, cause),
            Transport(ref kind) => write!(f, "{}: {}", d, kind),
            Unauthorized(None) => write!(f, "{}", d),
            Unauthorized(Some(ref response)) => write!(f, "{}: {}", d, response),
            UnexpectedContentTypeHeader{ref expected, ref got} => {
                write!(f, "{}: Expected '{}', got '{}'", d, expected, got)
            }
            UnexpectedHttpStatus{ref got} => write!(f, "{}: Got {}", d, got),
            UriParse{ref cause} => write!(f, "{}: {}", d, cause),
        }
    }
}

#[derive(Debug)]
pub enum BadPathKind {
    // TODO: It would be helpful to say 'why' the percent-encoding is invalid.
    BadPercentEncoding,
    NoLeadingSlash,
    NotDatabase,
    NotDesignDocument,
    NotDocument,
    NotView,
}

impl BadPathKind {
    fn cause(&self) -> Option<&std::error::Error> {
        use self::BadPathKind::*;
        match *self {
            BadPercentEncoding => None,
            NoLeadingSlash => None,
            NotDatabase => None,
            NotDesignDocument => None,
            NotDocument => None,
            NotView => None,
        }
    }
}

impl std::fmt::Display for BadPathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::BadPathKind::*;
        match *self {
            BadPercentEncoding => write!(f, "Path contains an invalid percent-encoding"),
            NoLeadingSlash => write!(f, "No leading slash"),
            NotDatabase => write!(f, "Path does not specify a database"),
            NotDesignDocument => write!(f, "Path does not specify a design document"),
            NotDocument => write!(f, "Path does not specify a document"),
            NotView => write!(f, "Path does not specify a view"),
        }
    }
}

#[derive(Debug)]
pub enum DecodeErrorKind {
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
            InvalidDocument { .. } => None,
            Serde { ref cause } => Some(cause),
        }
    }
}

impl std::fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::DecodeErrorKind::*;
        match *self {
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
