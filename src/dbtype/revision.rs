use serde;
use std;

use Error;

fn nibble_to_hex(x: u8) -> char {
    match x {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        x @ _ => {
            panic!("Invalid nibble value {}", x);
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Md5Hash([u8; 16]);

impl std::fmt::Display for Md5Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        // TODO: Refactor this function to eliminate heap memory allocation.

        let mut o = String::new();
        o.reserve(32);
        let Md5Hash(ref hash) = *self;
        for i in 0..16 {
            o.push(nibble_to_hex(hash[i] >> 4));
            o.push(nibble_to_hex(hash[i] & 0xf));
        }
        o.fmt(f)
    }
}

impl std::str::FromStr for Md5Hash {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        if s.len() != 32 {
            return Err(Error::BadMd5Hash);
        }

        let mut h = [0 as u8; 32];
        let mut i = 0;
        for c in s.chars() {
            h[i] = try!(c.to_digit(16).ok_or(Error::BadMd5Hash)) as u8;
            i += 1;
        }

        let mut o = [0 as u8; 16];
        for i in 0..16 {
            o[i] = (h[2 * i] << 4) + h[2 * i + 1];
        }

        Ok(Md5Hash(o))
    }
}

impl From<Md5Hash> for String {
    fn from(hash: Md5Hash) -> Self {
        format!("{}", hash)
    }
}

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
    fn nibble_to_hex() {
        assert_eq!('0', super::nibble_to_hex(0));
        assert_eq!('1', super::nibble_to_hex(1));
        assert_eq!('2', super::nibble_to_hex(2));
        assert_eq!('3', super::nibble_to_hex(3));
        assert_eq!('4', super::nibble_to_hex(4));
        assert_eq!('5', super::nibble_to_hex(5));
        assert_eq!('6', super::nibble_to_hex(6));
        assert_eq!('7', super::nibble_to_hex(7));
        assert_eq!('8', super::nibble_to_hex(8));
        assert_eq!('9', super::nibble_to_hex(9));
        assert_eq!('a', super::nibble_to_hex(10));
        assert_eq!('b', super::nibble_to_hex(11));
        assert_eq!('c', super::nibble_to_hex(12));
        assert_eq!('d', super::nibble_to_hex(13));
        assert_eq!('e', super::nibble_to_hex(14));
        assert_eq!('f', super::nibble_to_hex(15));
    }

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
                        _ => { panic!("Got unexpected error result: {}", e); }
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
