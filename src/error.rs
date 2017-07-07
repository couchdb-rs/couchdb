use std;
use transport::{Response, StatusCode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ServerResponseCategory {
    DatabaseExists,
    Unauthorized,
}

impl std::fmt::Display for ServerResponseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ServerResponseCategory::DatabaseExists => "The database already exists".fmt(f),
            &ServerResponseCategory::Unauthorized => "CouchDB server administrator privileges required".fmt(f),
        }
    }
}

/// `Error` contains information about an error originating in the client
/// or server.
#[derive(Debug)]
pub struct Error {
    inner: ErrorInner,
}

#[derive(Debug)]
enum ErrorInner {
    ServerResponse {
        category: Option<ServerResponseCategory>,
        request_description: String,
        status_code: StatusCode,
        nok: Option<Nok>,
    },
    Regular {
        description: String,
        cause: Option<String>,
    },
}

impl Error {
    #[doc(hidden)]
    pub fn new_server_response_error<R, S>(
        category: Option<ServerResponseCategory>,
        request_description: S,
        response: &mut R,
    ) -> Self
    where
        R: Response,
        S: Into<String>,
    {
        Error {
            inner: ErrorInner::ServerResponse {
                category: category,
                request_description: request_description.into(),
                status_code: response.status_code(),
                nok: response.decode_json_body().ok(),
            },
        }
    }

    pub fn is_database_exists(&self) -> bool {
        match self.inner {
            ErrorInner::ServerResponse { category: Some(ServerResponseCategory::DatabaseExists), .. } => true,
            _ => false,
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        match self.inner {
            ErrorInner::ServerResponse { category: Some(ServerResponseCategory::Unauthorized), .. } => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let description = std::error::Error::description(self);
        match self.inner {
            ErrorInner::ServerResponse {
                ref category,
                ref request_description,
                ref status_code,
                ref nok,
            } => {
                write!(f, "{} (request: {:?}", description, request_description)?;
                if let &Some(ref x) = category {
                    write!(f, ", category: {:?}", x)?;
                }
                write!(f, ", status: {}", status_code)?;
                if let &Some(ref x) = nok {
                    write!(f, ", error: {:?}, reason: {:?}", x.error(), x.reason())?;
                }
                write!(f, ")")
            }
            ErrorInner::Regular { cause: Some(ref cause), .. } => write!(f, "{}: {}", description, cause),
            ErrorInner::Regular { .. } => write!(f, "{}", description),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self.inner {
            ErrorInner::ServerResponse { status_code, .. }
                if status_code.is_client_error() || status_code.is_server_error() => "The CouchDB server responded with an error",
            ErrorInner::ServerResponse { .. } => "The CouchDB responded with an unexpected status",
            ErrorInner::Regular { ref description, .. } => &description,
        }
    }
}

// We implement From<'static str> and From<String> separately so that we don't
// conflict with From<std::io::Error>.

impl From<&'static str> for Error {
    fn from(description: &'static str) -> Error {
        Error {
            inner: ErrorInner::Regular {
                description: String::from(description),
                cause: None,
            },
        }
    }
}

impl From<String> for Error {
    fn from(description: String) -> Error {
        Error {
            inner: ErrorInner::Regular {
                description: description,
                cause: None,
            },
        }
    }
}

impl<E, R> From<(R, E)> for Error
where
    E: std::error::Error,
    R: Into<String>,
{
    fn from((description, cause): (R, E)) -> Error {
        Error {
            inner: ErrorInner::Regular {
                description: description.into(),
                cause: Some(cause.to_string()),
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(cause: std::io::Error) -> Self {
        Error {
            inner: ErrorInner::Regular {
                description: String::from("An I/O error occurred"),
                cause: Some(cause.to_string()),
            },
        }
    }
}

/// `Nok` contains error information from the CouchDB server for a request that
/// failed.
///
/// # Examples
///
/// ```
/// extern crate couchdb;
/// extern crate serde_json;
///
/// let source = r#"{"error":"file_exists",
///                  "reason":"The database could not be created, the file already exists."}"#;
///
/// let x: couchdb::Nok = serde_json::from_str(source).unwrap();
///
/// assert_eq!(x.error(), "file_exists");
/// assert_eq!(x.reason(),
///            "The database could not be created, the file already exists.");
/// ```
///
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nok {
    error: String,
    reason: String,
}

impl Nok {
    #[doc(hidden)]
    pub fn new<T, U>(error: T, reason: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Nok {
            error: error.into(),
            reason: reason.into(),
        }
    }

    /// Returns the high-level name of the error—e.g., <q>file_exists</q>.
    pub fn error(&self) -> &String {
        &self.error
    }

    /// Returns the low-level description of the error—e.g., <q>The database could
    /// not be created, the file already exists.</q>
    pub fn reason(&self) -> &String {
        &self.reason
    }
}

impl std::fmt::Display for Nok {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.error, self.reason)
    }
}
