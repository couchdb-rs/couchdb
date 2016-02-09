use mime;
use serde;

#[derive(Debug, PartialEq)]
pub struct ContentType(pub mime::Mime);

impl serde::Deserialize for ContentType {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = ContentType;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                let m = try!(v.parse().map_err(|_| E::invalid_value("Bad MIME string")));
                Ok(ContentType(m))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use super::ContentType;

    #[test]
    fn decode_json_ok() {
        let expected = ContentType("application/json".parse().unwrap());
        let source = serde_json::Value::String("application/json".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn decode_json_nok_bad_mime() {
        let source = serde_json::Value::String("bad mime".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<ContentType>(&source);
        expect_json_error_invalid_value!(got);
    }
}
