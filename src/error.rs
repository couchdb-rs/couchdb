use hyper;
use serde_json;
use std;
use url;

/// Principal error type.
///
/// The public API guarantees only the non-hidden variants and non-hidden
/// variant fields. Do not match against hidden variants or hidden variant
/// fields.
///
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
        cause: serde_json::error::Error,
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
        cause: TransportCause,
    },

    /// The client is unauthorized to carry out the operation.
    Unauthorized {

        /// Error string returned by CouchDB Server.
        error: String,

        /// Reason string returned by CouchDB Server.
        reason: String,
    },

    /// The CouchDB server responded with content that the client did not
    /// expect.
    #[doc(hidden)]
    UnexpectedContent {
        got: String,
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
            UnexpectedContent { .. } =>
                "The CouchDB server responded with content that the client did not expect",
            UnexpectedContentTypeHeader { .. } =>
                "The CouchDB server responded with a Content-Type header that the client did not expect",
            UnexpectedHttpStatus { .. } =>
                "The CouchDB server responded with an HTTP status code that the client did not expect",
            UriParse { .. } => "Invalid URI argument",
        }
    }

    fn cause(&self) -> std::option::Option<&std::error::Error> {
        use self::Error::*;
        match *self {
            DatabaseExists { .. } => None,
            Decode { ref cause } => Some(cause),
            DocumentConflict { .. } => None,
            Encode { ref cause } => Some(cause),
            InternalServerError { .. } => None,
            InvalidDatabaseName { .. } => None,
            InvalidRequest { .. } => None,
            Io { ref cause, .. } => Some(cause),
            NoContentTypeHeader { .. } => None,
            NotFound { .. } => None,
            ReceiveFromThread { ref cause, .. } => Some(cause),
            Transport { ref cause }  => {
                use self::TransportCause::*;
                match *cause {
                    Hyper(ref cause) => Some(cause),
                    Io(ref cause) => Some(cause),
                }
            },
            Unauthorized { .. } => None,
            UnexpectedContent { .. } => None,
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
                write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
            Decode { ref cause } => write!(f, "{}: {}", self.description(), cause),
            DocumentConflict { ref response } =>
                write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
            Encode { ref cause } => write!(f, "{}: {}", self.description(), cause),
            InternalServerError { ref response } =>
                write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
            InvalidDatabaseName { ref response } =>
                write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
            InvalidRequest { ref response } =>
                write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
            Io { ref cause, .. } => write!(f, "{}: {}", self.description(), cause),
            NoContentTypeHeader { ref expected } =>
                write!(f, "{}: Expected '{}'", self.description(), expected),
            NotFound { ref response } => match *response {
                Some(ref response) =>
                    write!(f, "{}: {}: {}", self.description(), response.error, response.reason),
                None => write!(f, "{}", self.description()),
            },
            ReceiveFromThread { ref cause, .. } =>
                write!(f, "{}: {}", self.description(), cause),
            Transport { ref cause } => write!(f, "{}: {}", self.description(), cause),
            Unauthorized { ref error, ref reason } =>
                write!(f, "{}: {}: {}", self.description(), error, reason),
            UnexpectedContent { ref got } => write!(f, "{}: Got {}", self.description(), got),
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
    error: String,

    /// Reason string returned by CouchDB Server.
    reason: String,
}

#[derive(Debug)]
pub enum TransportCause {
    Hyper(hyper::error::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for TransportCause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::TransportCause::*;
        match *self {
            Hyper(ref e) => e.fmt(f),
            Io(ref e) => e.fmt(f),
        }
    }
}

fn extract_couchdb_error_and_reason(resp: &mut hyper::client::Response)
                                    -> Result<(String, String), Error> {
    let mut s = String::new();
    {
        use std::io::Read;
        try!(
            resp.read_to_string(&mut s)
            .or_else(|e| {
                Err(Error::Transport { cause: TransportCause::Io(e), } )
            })
        );
    }
    let s = s;
    let mut v = try!(
        serde_json::from_str::<serde_json::Value>(&s)
        .or_else(|e| { Err(Error::Decode { cause: e } ) })
    );

    (|| {
        let dot = match v.as_object_mut() {
            Some(x) => x,
            None => { return None; },
        };
        let error = {
            let x = match dot.get_mut("error") {
                Some(x) => x,
                None => { return None; },
            };
            match *x {
                serde_json::Value::String(ref mut x) => std::mem::replace(x, String::new()),
                _ => { return None; },
            }
        };
        let reason = {
            let x = match dot.get_mut("reason") {
                Some(x) => x,
                None => { return None; },
            };
            match *x {
                serde_json::Value::String(ref mut x) => std::mem::replace(x, String::new()),
                _ => { return None; },
            }
        };
        Some((error, reason))
    })()
    .ok_or(Error::UnexpectedContent { got: s } )
}

pub fn new_because_database_exists(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::DatabaseExists {
            response: ErrorResponse {
                error: error,
                reason: reason,
            },
        },
    }
}

pub fn new_because_document_conflict(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::DocumentConflict {
            response: ErrorResponse {
                error: error,
                reason: reason,
            }
        },
    }
}

pub fn new_because_internal_server_error(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::InternalServerError {
            response: ErrorResponse {
                error: error,
                reason: reason,
            },
        },
    }
}

pub fn new_because_invalid_database_name(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::InvalidDatabaseName {
            response: ErrorResponse {
                error: error,
                reason: reason,
            },
        },
    }
}

pub fn new_because_invalid_request(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::InvalidRequest {
            response: ErrorResponse {
                error: error,
                reason: reason,
            },
        },
    }
}

pub fn new_because_not_found(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::NotFound {
            response: Some(ErrorResponse {
                error: error,
                reason: reason,
            }),
        },
    }
}

pub fn new_because_unauthorized(resp: &mut hyper::client::Response) -> Error {
    match extract_couchdb_error_and_reason(resp) {
        Err(e) => e,
        Ok((error, reason)) => Error::Unauthorized {
            error: error,
            reason: reason,
        },
    }
}
