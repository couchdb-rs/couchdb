use std;
use std::fmt::Display;
use transport::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCategory {
    DatabaseExists,
    Unauthorized,
}

impl Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ErrorCategory::DatabaseExists => "The database already exists".fmt(f),
            &ErrorCategory::Unauthorized => "CouchDB server administrator privileges are required".fmt(f),
        }
    }
}

/// `Error` contains information about an error originating in the client
/// or server.
#[derive(Debug)]
pub struct Error {
    description: String,
    category: Option<ErrorCategory>,
    cause: Option<Box<std::error::Error>>,
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
    pub fn chain<D>(description: D, sub_error: Error) -> Self
    where
        D: Into<String>,
    {
        Error {
            description: description.into(),
            category: sub_error.category,
            cause: Some(Box::new(sub_error)),
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

    fn cause(&self) -> Option<&std::error::Error> {
        self.cause.as_ref().map(|x| x.as_ref())
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
    E: Into<Box<std::error::Error>>,
{
    fn from((description, cause): (D, E)) -> Self {
        Error {
            description: description.into(),
            category: None,
            cause: Some(cause.into()),
        }
    }
}

impl<D, E> From<(D, ErrorCategory, E)> for Error
where
    D: Into<String>,
    E: Into<Box<std::error::Error>>,
{
    fn from((description, category, cause): (D, ErrorCategory, E)) -> Self {
        Error {
            description: description.into(),
            category: Some(category),
            cause: Some(cause.into()),
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
