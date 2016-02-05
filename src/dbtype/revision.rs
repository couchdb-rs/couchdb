use serde;
use std;

use Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Md5Hash([u8; 16]);

impl_hex16_base_type!(Md5Hash, hash, BadMd5Hash);

/// Revision of a document.
///
/// A document revision comprises a number and an MD5 hash sum. In serialized
/// form, a revision looks something like `42-1234567890abcdef1234567890abcdef`.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision {
    number: u64,
    hash: Md5Hash,
}

impl Revision {
    /// Constructs a new `Revision` from the given string.
    ///
    /// The string must be of a form like `42-1234567890abcdef1234567890abcdef`.
    ///
    pub fn parse(s: &str) -> Result<Self, Error> {
        use std::str::FromStr;
        Revision::from_str(s)
    }

    /// Returns the update number part of the revision.
    ///
    /// The update number is the `999` part of the revision
    /// `999-1234567890abcdef1234567890abcdef`.
    ///
    pub fn update_number(&self) -> u64 {
        self.number
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.number, self.hash)
    }
}

impl std::str::FromStr for Revision {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        // TODO: Refactor this function to eliminate heap memory allocation.

        let parts = s.split('-').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(Error::BadRevision);
        }

        let number = match try!(u64::from_str_radix(parts[0], 10)
                                    .map_err(|_| Error::BadRevision)) {
            0 => {
                return Err(Error::BadRevision);
            }
            x @ _ => x,
        };

        let rev = Revision {
            number: number,
            hash: try!(Md5Hash::from_str(parts[1]).map_err(|_| Error::BadRevision)),
        };

        Ok(rev)
    }
}

impl From<Revision> for String {
    fn from(rev: Revision) -> Self {
        format!("{}", rev)
    }
}

impl serde::Serialize for Revision {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let s = format!("{}", self);
        serializer.visit_str(&s)
    }
}

impl serde::Deserialize for Revision {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Revision;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                use std::error::Error;
                Revision::parse(v).map_err(|e| E::invalid_value(e.description()))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use Error;
    use super::{Md5Hash, Revision};

    #[test]
    fn md5_hash_display() {
        let expected = "1234567890abcdef1234567890abcdef";
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56,
                                0x78, 0x90, 0xab, 0xcd, 0xef];
        let source = Md5Hash(source);
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn md5_hash_from_str_ok() {
        use std::str::FromStr;
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54,
                                0x76, 0x98, 0xba, 0xdc, 0xfe];
        let expected = Md5Hash(source);
        let got = Md5Hash::from_str("1234567890abcdef1032547698BADCFE").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn md5_hash_from_str_nok() {
        macro_rules! expect_error {
            ($input: expr) => {
                {
                    use std::str::FromStr;
                    match Md5Hash::from_str($input) {
                        Ok(_) => { panic!("Got unexpected OK result"); },
                        Err(e) => match e {
                            Error::BadMd5Hash => (),
                            _ => { panic!("Got unexpected error result: {}", e); }
                        }
                    }
                }
            }
        }

        use std::str::FromStr;

        expect_error!("");
        expect_error!("bad_revision");
        expect_error!("12345678");
        expect_error!("1234567812345678123456781234567");
        expect_error!("12345678123456781234567812345678a");
        expect_error!("1234567812345678123456781234567z");
        expect_error!("z2345678123456781234567812345678");
        expect_error!("12345678123456z81234567812345678");
        expect_error!("12345678123456g81234567812345678");
        expect_error!("12345678123456_81234567812345678");
        expect_error!("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    }

    #[test]
    fn string_from_md5_hash() {
        let expected = "1234567890abcdef1234567890abcdef";
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56,
                                0x78, 0x90, 0xab, 0xcd, 0xef];
        let source = Md5Hash(source);
        let got = String::from(source);
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_parse_ok() {
        use std::str::FromStr;
        let expected = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let got = Revision::parse("42-1234567890abcdeffedcba0987654321").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_parse_nok() {
        Revision::parse("bad_revision").unwrap_err();
    }

    #[test]
    fn revision_update_number() {
        let rev = Revision::parse("999-1234567890abcdef1234567890abcdef").unwrap();
        assert_eq!(999, rev.update_number());
    }

    #[test]
    fn revision_display() {
        use std::str::FromStr;
        let expected = "42-1234567890abcdeffedcba0987654321";
        let source = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_from_str_ok() {
        use std::str::FromStr;
        let expected = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let got = Revision::from_str("42-1234567890abcdeffedcba0987654321").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_from_str_nok() {
        macro_rules! expect_error {
            ($input: expr) => {
                match Revision::from_str($input) {
                    Ok(_) => { panic!("Got unexpected OK result"); },
                    Err(e) => match e {
                        Error::BadRevision => (),
                        _ => { panic!("Got unexpected error variant: {}", e); }
                    }
                }
            }
        }

        use std::str::FromStr;

        expect_error!("12345678123456781234567812345678");
        expect_error!("-12345678123456781234567812345678");
        expect_error!("-42-12345678123456781234567812345678");
        expect_error!("18446744073709551616-12345678123456781234567812345678"); // overflow
        expect_error!("0-12345678123456781234567812345678"); // zero number not allowed
        expect_error!("1-z2345678123456781234567812345678");
        expect_error!("1-1234567812345678123456781234567");
        expect_error!("bad_revision_blah_blah_blah");
    }

    #[test]
    fn string_from_revision() {
        use std::str::FromStr;
        let expected = "42-1234567890abcdeffedcba0987654321";
        let source = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_eq_same() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 == r2);
    }

    #[test]
    fn revision_eq_different_numbers() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("7-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 != r2);
    }

    #[test]
    fn revision_eq_different_hashes() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-9999567890abcdef1234567890abcdef").unwrap();
        assert!(r1 != r2);
    }

    #[test]
    fn revision_eq_case_insensitive() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890ABCDEF").unwrap();
        let r2 = Revision::parse("1-1234567890ABCDEf1234567890abcdef").unwrap();
        assert!(r1 == r2);
    }

    #[test]
    fn revision_ord_same() {
        let r1 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 <= r2 && r2 <= r1);
    }

    #[test]
    fn revision_ord_different_numbers() {
        // The number part is compared numerically, not lexicographically.
        let r1 = Revision::parse("7-1234567890abcdef1234567890abcdef").unwrap();
        let r2 = Revision::parse("13-1234567890abcdef1234567890abcdef").unwrap();
        assert!(r1 < r2);
    }

    #[test]
    fn revision_ord_case_insensitive() {
        let r1 = Revision::parse("1-1234567890ABCDEF1234567890abcdef").unwrap();
        let r2 = Revision::parse("1-1234567890abcdef1234567890ABCDEF").unwrap();
        assert!(r1 <= r2 && r2 <= r1);
    }

    #[test]
    fn revision_serialization() {
        use std::str::FromStr;
        let expected = serde_json::Value::String("42-1234567890abcdeffedcba0987654321".to_string());
        let source = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_deserialization_ok() {
        use std::str::FromStr;
        let expected = Revision {
            number: 42,
            hash: Md5Hash::from_str("1234567890abcdeffedcba0987654321").unwrap(),
        };
        let source = serde_json::Value::String("42-1234567890abcdeffedcba0987654321".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn revision_deserialization_nok() {
        let source = serde_json::Value::String("bad_revision".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Revision>(&s);
        expect_json_error_invalid_value!(got);
    }
}
