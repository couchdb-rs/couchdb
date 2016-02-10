use serde;
use uuid;

// Uuid wraps a uuid::Uuid while implementing serialization.
#[derive(Debug, PartialEq)]
pub struct Uuid(pub uuid::Uuid);

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
                let uuid = try!(v.parse().map_err(|_| E::invalid_value("Bad UUID string")));
                Ok(Uuid(uuid))
            }
        }

        deserializer.visit(Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use super::Uuid;

    #[test]
    fn json_decodes_ok() {
        let expected = Uuid("85fb71bf700c17267fef77535820e371".parse().unwrap());
        let source = serde_json::Value::String("85fb71bf700c17267fef77535820e371".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&source).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn json_decodes_nok_bad_value() {
        let source = serde_json::Value::String("bad uuid".to_owned());
        let source = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Uuid>(&source);
        expect_json_error_invalid_value!(got);
    }
}
