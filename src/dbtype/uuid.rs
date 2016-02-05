use serde;
use std;

use Error;

/// A universally unique identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Uuid([u8; 16]);

impl_hex16_base_type!(Uuid, uuid, BadUuid);

impl serde::Deserialize for Uuid {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Uuid;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                use std::str::FromStr;
                use std::error::Error;
                Uuid::from_str(v).map_err(|e| E::invalid_value(e.description()))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use Error;
    use super::Uuid;

    #[test]
    fn display() {
        let expected = "1234567890abcdef1234567890abcdef";
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56,
                                0x78, 0x90, 0xab, 0xcd, 0xef];
        let source = Uuid(source);
        let got = format!("{}", source);
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_ok() {
        use std::str::FromStr;
        let source: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54,
                                0x76, 0x98, 0xba, 0xdc, 0xfe];
        let expected = Uuid(source);
        let got = Uuid::from_str("1234567890abcdef1032547698BADCFE").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn from_str_nok() {
        macro_rules! expect_error {
            ($input: expr) => {
                {
                    use std::str::FromStr;
                    match Uuid::from_str($input) {
                        Ok(_) => { panic!("Got unexpected OK result"); },
                        Err(e) => match e {
                            Error::BadUuid => (),
                            _ => { panic!("Got unexpected error variant: {}", e); }
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
    fn deserialization_ok() {
        use std::str::FromStr;
        let expected = Uuid::from_str("1234567890abcdeffedcba0987654321").unwrap();
        let source = serde_json::Value::String("1234567890abcdeffedcba0987654321".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn deserialization_nok_bad_value() {
        let source = serde_json::Value::String("bad_uuid".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Uuid>(&s);
        expect_json_error_invalid_value!(got);
    }
}
