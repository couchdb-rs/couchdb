use std;
use std::fmt::Display;
use transport::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCategory {
    DatabaseDoesNotExist,
    DatabaseExists,
    Unauthorized,
}

impl Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ErrorCategory::DatabaseDoesNotExist => "The database does not exist".fmt(f),
            &ErrorCategory::DatabaseExists => "The database already exists".fmt(f),
            &ErrorCategory::Unauthorized => "CouchDB server administrator privileges are required".fmt(f),
        }
    }
}

/// `Error` contains information about an error originating in the client
/// or server.
///
/// `Error` implements the `Sync` trait so that actions' futures may be sent
/// between threads.
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

    pub fn is_database_does_not_exist(&self) -> bool {
        match self.category {
            Some(ErrorCategory::DatabaseDoesNotExist) => true,
            _ => false,
        }
    }

    pub fn is_database_exists(&self) -> bool {
        match self.category {
            Some(ErrorCategory::DatabaseExists) => true,
            _ => false,
        }
    }

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

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nok {
    error: String,
    reason: String,
}

impl Nok {
    pub fn error(&self) -> &String {
        &self.error
    }

    pub fn reason(&self) -> &String {
        &self.reason
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
