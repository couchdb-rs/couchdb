use {Error, serde, std};
use uuid::Uuid;

/// `Revision` contains a document revision.
///
/// # Summary
///
/// * `Revision` stores a revision, which is a string that looks like
///   `1-9c65296036141e575d32ba9c034dd3ee`.
///
/// * `Revision` implements `Deserialize` and `Serialize`.
///
/// # Remarks
///
/// A CouchDB document revision comprises a **sequence number** and an **MD5
/// digest**. The sequence number (usually) starts at `1` when the document is
/// created and increments by one each time the document is updated. The digest
/// is a hash of the document content, which the CouchDB server uses to detect
/// conflicts.
///
/// # Example
///
/// ```
/// extern crate couchdb;
///
/// let rev = couchdb::Revision::parse("42-1234567890abcdef1234567890abcdef")
///     .unwrap();
///
/// assert_eq!(rev.to_string(), "42-1234567890abcdef1234567890abcdef");
/// ```
///
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Revision {
    sequence_number: u64,
    digest: Uuid,
}

impl Revision {
    /// Constructs a new `Revision` from the given string.
    ///
    /// The string must be of the form `42-1234567890abcdef1234567890abcdef`.
    ///
    pub fn parse(s: &str) -> Result<Self, Error> {
        use std::str::FromStr;
        Revision::from_str(s)
    }

    /// Returns the sequence number part of the revision.
    ///
    /// The sequence number is the `123` part of the revision
    /// `123-00000000000000000000000000000000`.
    ///
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.sequence_number, self.digest.simple())
    }
}

const E_BAD_REVISION: &str = "The revision is badly formatted";
const E_NOT_ALL_HEX: &str = "The digest part contains one or more non-hexadecimal characters";
const E_DIGEST_PARSE: &str = "The digest part is badly formatted";
const E_NUMBER_PARSE: &str = "The number part is badly formatted";
const E_TOO_FEW_PARTS: &str = "Missing number part and/or digest part";
const E_ZERO_SEQUENCE_NUMBER: &str = "The number part is zero";

impl std::str::FromStr for Revision {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let mut parts = s.splitn(2, '-');

        let sequence_number_str = parts.next().ok_or(Error::from(format!(
            "{}: {}",
            E_BAD_REVISION,
            E_TOO_FEW_PARTS
        )))?;

        let sequence_number = u64::from_str_radix(sequence_number_str, 10).map_err(|e| {
            Error::from((format!("{}: {}", E_BAD_REVISION, E_NUMBER_PARSE), e))
        })?;

        if sequence_number == 0 {
            return Err(Error::from(
                format!("{}: {}", E_BAD_REVISION, E_ZERO_SEQUENCE_NUMBER),
            ));
        }

        let digest_str = parts.next().ok_or(Error::from(format!(
            "{}: {}",
            E_BAD_REVISION,
            E_TOO_FEW_PARTS
        )))?;

        let digest = Uuid::parse_str(digest_str).map_err(|e| {
            Error::from((format!("{}: {}", E_BAD_REVISION, E_DIGEST_PARSE), e))
        })?;

        if digest_str.chars().any(|c| !c.is_digit(16)) {
            return Err(Error::from(
                format!("{}: {}", E_BAD_REVISION, E_NOT_ALL_HEX),
            ));
        }

        Ok(Revision {
            sequence_number: sequence_number,
            digest: digest,
        })
    }
}

impl From<Revision> for String {
    fn from(revision: Revision) -> Self {
        revision.to_string()
    }
}

impl serde::Serialize for Revision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Revision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Revision;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                write!(f, "a string specifying a CouchDB document revision")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Revision::parse(v).map_err(|_e| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_test;

    #[test]
    fn parse_ok() {
        let expected = Revision {
            sequence_number: 42,
            digest: "1234567890abcdeffedcba0987654321".parse().unwrap(),
        };
        let got = Revision::parse("42-1234567890abcdeffedcba0987654321").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn parse_nok() {
        Revision::parse("bad_revision").unwrap_err();
    }

    #[test]
    fn sequence_number() {
        let rev = Revision::parse("999-1234567890abcdef1234567890abcdef").unwrap();
        assert_eq!(999, rev.sequence_number());
    }

    #[test]
    fn display() {
        let expected = "42-1234567890abcdeffedcba0987654321";
        let source = Revision {
            sequence_number: 42,
            digest: "1234567890abcdeffedcba0987654321".parse().unwrap(),
        };
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_ok() {
        use std::str::FromStr;
        let expected = Revision {
            sequence_number: 42,
            digest: "1234567890abcdeffedcba0987654321".parse().unwrap(),
        };
        let got = Revision::from_str("42-1234567890abcdeffedcba0987654321").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_nok() {
        macro_rules! expect_error {
            ($input: expr) => {
                match Revision::from_str($input) {
                    Err(Error::RevisionParse{..}) => (),
                    x => panic!("Got unexpected result {:?}", x),
                }
            }
        }

        use std::str::FromStr;

        Revision::from_str("12345678123456781234567812345678").unwrap_err();
        Revision::from_str("-12345678123456781234567812345678").unwrap_err();
        Revision::from_str("1-").unwrap_err();
        Revision::from_str("1-1234567890abcdef1234567890abcdef-").unwrap_err();
        Revision::from_str("-42-12345678123456781234567812345678").unwrap_err();
        Revision::from_str("18446744073709551616-12345678123456781234567812345678").unwrap_err(); // overflow
        Revision::from_str("0-12345678123456781234567812345678").unwrap_err(); // zero sequence_number not allowed
        Revision::from_str("1-z2345678123456781234567812345678").unwrap_err();
        Revision::from_str("1-1234567812345678123456781234567").unwrap_err();
        Revision::from_str("bad_revision_blah_blah_blah").unwrap_err();
    }

    #[test]
    fn string_from_revision() {
        let expected = "42-1234567890abcdeffedcba0987654321";
        let source = Revision {
            sequence_number: 42,
            digest: "1234567890abcdeffedcba0987654321".parse().unwrap(),
        };
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn eq_same() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 == r2);
    }

    #[test]
    fn eq_different_numbers() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("7-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 != r2);
    }

    #[test]
    fn eq_different_digests() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-9999567890abcdef1234567890abcdef").unwrap();
        assert!(r1 != r2);
    }

    #[test]
    fn eq_case_insensitive() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890ABCDEF").unwrap();
        let r2 = Revision::parse("1-1234567890ABCDEf1234567890abcdef").unwrap();
        assert!(r1 == r2);
    }

    #[test]
    fn serialization_and_deserialization_ok() {

        use serde_test::Token;

        let rev = Revision::parse("42-1234567890abcdeffedcba0987654321").unwrap();
        serde_test::assert_tokens(&rev, &[Token::Str("42-1234567890abcdeffedcba0987654321")]);
    }

    #[test]
    fn deserialization_enforces_revision_validity() {
        use serde_test::Token;
        serde_test::assert_de_tokens_error::<Revision>(
            &[Token::Str("obviously_bad")],
            "invalid value: string \"obviously_bad\", expected a string specifying a CouchDB document revision",
        );
    }
}
