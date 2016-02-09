use base64;
use serde;

#[derive(Debug, PartialEq)]
pub struct Base64Blob(pub Vec<u8>);

impl serde::Deserialize for Base64Blob {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Base64Blob;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                use std::error::Error;
                let blob = try!(base64::u8de(v.as_bytes())
                                    .map_err(|e| E::invalid_value(e.description())));
                Ok(Base64Blob(blob))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use super::Base64Blob;

    #[test]
    fn decode_json_ok() {
        let expected = Base64Blob("hello".to_owned().into_bytes());
        let source = serde_json::Value::String("aGVsbG8=".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_nok_bad_mime() {
        let source = serde_json::Value::String(".?$%".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Base64Blob>(&source);
        expect_json_error_invalid_value!(got);
    }
}
