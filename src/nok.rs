use std::marker::PhantomData;

/// `Nok` contains the content of an error response from the CouchDB server.
///
/// # Summary
///
/// * `Nok` has public members instead of accessor methods because there are no
///   invariants restricting the data.
///
/// * `Nok` implements `Deserialize`.
///
/// # Remarks
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
/// assert_eq!(nok.error, "file_exists");
/// assert_eq!(nok.reason,
///            "The database could not be created, the file already exists.");
/// ```
///
/// # Compatibility
///
/// `Nok` contains a dummy private member in order to prevent applications from
/// directly constructing a `Nok` instance. This allows new fields to be added
/// to `Nok` in future releases without it being a breaking change.
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nok {
    pub error: String,
    pub reason: String,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}
