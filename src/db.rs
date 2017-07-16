//! The `db` module provides types that mirror CouchDB server types.

use DatabaseName;
use std::marker::PhantomData;
use uuid::Uuid;

/// `Database` contains the content of a database resource (e.g., `/{db}`).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Database {
    pub committed_update_seq: u64,
    pub compact_running: bool,
    pub db_name: DatabaseName,
    pub disk_format_version: i32,
    pub data_size: u64,
    pub disk_size: u64,
    pub doc_count: u64,
    pub doc_del_count: u64,
    pub instance_start_time: String,
    pub purge_seq: u64,
    pub update_seq: u64,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

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
/// assert_eq!(nok.error, "file_exists");
/// assert_eq!(nok.reason,
///            "The database could not be created, the file already exists.");
/// ```
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nok {
    pub error: String,
    pub reason: String,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

/// `Root` contains the content of a CouchDB server's root resource (`/`).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Root {
    pub couchdb: String,
    pub uuid: Uuid,
    pub vendor: Vendor,
    pub version: String,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

impl Root {
    /// Tries to parse the server's version into major, minor, and patch
    /// numbers.
    pub fn version_triple(&self) -> Option<(u64, u64, u64)> {
        parse_version(&self.version)
    }
}

/// `Vendor` contains information about the CouchDB server vendor.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Vendor {
    pub name: String,
    pub version: String,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {

    const BASE: u32 = 10;

    let parts = s.split(|c: char| !c.is_digit(BASE))
        .map(|s| {
            u64::from_str_radix(s, BASE).map(|x| Some(x)).unwrap_or(
                None,
            )
        })
        .take(3)
        .collect::<Vec<_>>();

    if parts.len() < 3 || parts.iter().any(|&x| x.is_none()) {
        return None;
    }

    Some((parts[0].unwrap(), parts[1].unwrap(), parts[2].unwrap()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn we_can_parse_couchdb_server_version() {
        use super::parse_version;
        assert_eq!(parse_version("1.6.1"), Some((1, 6, 1)));
        assert_eq!(parse_version("1.6.1_1"), Some((1, 6, 1))); // seen in Homebrew
        assert_eq!(parse_version("obviously_bad"), None);
    }
}
