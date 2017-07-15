/// `Nok` contains the content of an error response from the CouchDB server.
///
/// # Summary
///
/// * Applications normally **do not** need to use `Nok` because actions already
///   return this information via [`Error`](struct.Error.html), if available.
///
/// * `Nok` contains the “error” and “reason” strings that the CouchDB server
///   responds with in case of an error.
///
/// # Remarks
///
/// `Nok` could be useful to an application if the application communicates
/// directly with the CouchDB server using some other HTTP transport, such as
/// the [`hyper`](https://crates.io/crates/hyper) crate.
///
/// When the CouchDB server responds with a 4xx- or 5xx status code, the
/// response usually has a body containing a JSON object with an “error” string
/// and a “reason” string. For example:
///
/// ```text
/// {
///   "error": "file_exists",
///   "reason": "The database could not be created, the file already exists."
/// }
/// ```
///
/// The `Nok` type contains the information from the response body.
///
/// ```
/// extern crate couchdb;
/// extern crate serde_json;
///
/// # let body = br#"{
/// # "error": "file_exists",
/// # "reason": "The database could not be created, the file already exists."
/// # }"#;
/// #
/// let nok: couchdb::Nok = serde_json::from_slice(body).unwrap();
///
/// assert_eq!(nok.error(), "file_exists");
/// assert_eq!(nok.reason(),
///            "The database could not be created, the file already exists.");
/// ```
///
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nok {
    error: String,
    reason: String,
}

impl Nok {
    // Exposed only for the Nok doc test. Would it make sense to expose this
    // method or some other means of construction?
    #[doc(hidden)]
    pub fn new(error: String, reason: String) -> Self {
        Nok {
            error: error,
            reason: reason,
        }
    }

    /// Returns the “error” string from the response body.
    ///
    /// The “error” string is the high-level name of the error—e.g.,
    /// “file_exists”.
    ///
    pub fn error(&self) -> &str {
        &self.error
    }

    /// Returns the ”reason” string from the response body.
    ///
    /// The “reason” string is the low-level description of the error—e.g., “The
    /// database could not be created, the file already exists.”
    ///
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
