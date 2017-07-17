use std;
use std::marker::PhantomData;
use uuid::Uuid;

/// `Root` contains the content of a CouchDB server's root resource.
///
/// # Summary
///
/// * `Root` has public members instead of accessor methods because there are no
///   invariants restricting the data.
///
/// * `Root` implements `Deserialize`.
///
/// # Remarks
///
/// An application may obtain a CouchDB server's root resource by sending an
/// HTTP request to GET `/`.
///
/// # Compatibility
///
/// `Root` contains a dummy private member in order to prevent applications from
/// directly constructing a `Root` instance. This allows new fields to be added
/// to `Root` in future releases without it being a breaking change.
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
pub struct Root {
    pub couchdb: String,
    pub uuid: Uuid,
    pub vendor: Vendor,
    pub version: Version,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

/// `Vendor` contains information about a CouchDB server vendor.
///
/// # Summary
///
/// * `Vendor` has public members instead of accessor methods because there are
///   no invariants restricting the data.
///
/// * `Vendor` implements `Deserialize`.
///
/// # Remarks
///
/// `Vendor` is normally part of a [`Root`](struct.Root.html) instance.
///
/// # Compatibility
///
/// `Vendor` contains a dummy private member in order to prevent applications
/// from directly constructing a `Vendor` instance. This allows new fields to be
/// added to `Vendor` in future releases without it being a breaking change.
///
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
pub struct Vendor {
    pub name: String,
    pub version: Version,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

/// `Version` is a string specifying a version.
///
/// # Summary
///
/// * `Version` thinly wraps a string but may be parsed into its major, minor,
///   and patch numbers.
///
/// * `Version` implements `Deserialize`.
///
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
pub struct Version(String);

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Version> for String {
    fn from(v: Version) -> Self {
        v.0
    }
}

impl From<String> for Version {
    fn from(s: String) -> Self {
        Version(s)
    }
}

impl<'a> From<&'a str> for Version {
    fn from(s: &'a str) -> Self {
        Version(String::from(s))
    }
}

impl Version {
    /// Tries to obtain the major, minor, and patch numbers from the version
    /// string.
    pub fn triple(&self) -> Option<(u64, u64, u64)> {

        const BASE: u32 = 10;

        let parts = self.0
            .split(|c: char| !c.is_digit(BASE))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::marker::PhantomData;

    #[test]
    fn version_parses_triple() {
        assert_eq!(Version::from("1.6.1").triple(), Some((1, 6, 1)));

        // E.g., the Homebrew vendor appends an extra number onto their version.
        assert_eq!(Version::from("1.6.1_1").triple(), Some((1, 6, 1)));

        assert_eq!(Version::from("obviously_bad").triple(), None);
    }

    #[test]
    fn root_deserializes_ok() {

        let source = r#"{
            "couchdb": "Welcome",
            "uuid": "0762dcce5f0d7f6f79157f852186f149",
            "version": "1.6.1",
            "vendor": {
                "name": "Homebrew",
                "version": "1.6.1_9"
            }
        }"#;

        let expected = Root {
            couchdb: String::from("Welcome"),
            uuid: Uuid::parse_str("0762dcce5f0d7f6f79157f852186f149").unwrap(),
            vendor: Vendor {
                name: String::from("Homebrew"),
                version: Version::from("1.6.1_9"),
                _private_guard: PhantomData,
            },
            version: Version::from("1.6.1"),
            _private_guard: PhantomData,
        };

        let got: Root = serde_json::from_str(source).unwrap();
        assert_eq!(got, expected);
    }
}
