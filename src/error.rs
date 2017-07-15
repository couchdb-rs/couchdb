use {Nok, std};
use std::fmt::Display;
use transport::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCategory {
    DatabaseExists,
    NotFound,
    Unauthorized,
}

impl Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ErrorCategory::NotFound => "The resource does not exist or is deleted".fmt(f),
            &ErrorCategory::DatabaseExists => "The database already exists".fmt(f),
            &ErrorCategory::Unauthorized => "The client is not authorized to complete the action".fmt(f),
        }
    }
}

/// `Error` is the `couchdb` crate's principal error type.
///
/// # Summary
///
/// * `Error` contains information about an error that originated in the client
///   or the server.
///
/// * `Error` implements the `Sync` trait so that actions' futures may be sent
///   between threads.
///
#[derive(Debug)]
pub struct Error {
    description: String,
    category: Option<ErrorCategory>,
    cause: Option<String>,
}

impl Error {
    #[doc(hidden)]
    pub fn from_server_response(status_code: StatusCode, nok: Option<Nok>, category: Option<ErrorCategory>) -> Self {
        let mut s = format!(
            "The server responded with an error or an unexpected status code (status: {}",
            status_code
        );
        if let Some(nok) = nok {
            s = format!(
                "{}, error: {:?}, reason: {:?}",
                s,
                nok.error(),
                nok.reason()
            );
        }
        s = format!("{})", s);
        let mut e = Error::from(s);
        e.category = category;
        e
    }

    /// Constructs an `Error` with another `Error` as its cause, preserving the
    /// cause's error category, if any.
    pub fn chain<D>(description: D, cause: Error) -> Self
    where
        D: Into<String>,
    {
        Error {
            description: description.into(),
            category: cause.category,
            cause: Some(cause.to_string()),
        }
    }

    /// Returns true if and only if the CouchDB server responded with an error
    /// because the database already exists.
    pub fn is_database_exists(&self) -> bool {
        match self.category {
            Some(ErrorCategory::DatabaseExists) => true,
            _ => false,
        }
    }

    /// Returns true if and only if the CouchDB server responded with an error
    /// because the target resource (e.g., database, document, etc.) does not
    /// exist or is deleted.
    pub fn is_not_found(&self) -> bool {
        match self.category {
            Some(ErrorCategory::NotFound) => true,
            _ => false,
        }
    }

    /// Returns true if and only if the CouchDB server responded with an error
    /// because the client is unauthorized to complete the action.
    pub fn is_unauthorized(&self) -> bool {
        match self.category {
            Some(ErrorCategory::Unauthorized) => true,
            _ => false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::error::Error::description(self).fmt(f)?;
        if let Some(ref cause) = self.cause {
            write!(f, ": {}", cause)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

// We implement From<'static str> and From<String> separately so that we don't
// conflict with From<std::io::Error>.

impl From<&'static str> for Error {
    fn from(description: &'static str) -> Error {
        Error {
            description: String::from(description),
            category: None,
            cause: None,
        }
    }
}

impl From<String> for Error {
    fn from(description: String) -> Error {
        Error {
            description: description,
            category: None,
            cause: None,
        }
    }
}

impl<D> From<(D, ErrorCategory)> for Error
where
    D: Into<String>,
{
    fn from((description, category): (D, ErrorCategory)) -> Self {
        Error {
            description: description.into(),
            category: Some(category),
            cause: None,
        }
    }
}

impl<D, E> From<(D, E)> for Error
where
    D: Into<String>,
    E: std::error::Error,
{
    fn from((description, cause): (D, E)) -> Self {
        Error {
            description: description.into(),
            category: None,
            cause: Some(cause.to_string()),
        }
    }
}

impl<D, E> From<(D, ErrorCategory, E)> for Error
where
    D: Into<String>,
    E: std::error::Error,
{
    fn from((description, category, cause): (D, ErrorCategory, E)) -> Self {
        Error {
            description: description.into(),
            category: Some(category),
            cause: Some(cause.to_string()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(cause: std::io::Error) -> Self {
        Error::from(("An I/O error occurred", cause))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_implements_send() {
        fn requires_send<T: Send>() {}
        requires_send::<Error>();
    }
}
