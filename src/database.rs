use {DatabaseName, serde, std};
use serde::Deserializer;
use std::marker::PhantomData;

/// `Database` contains the content of a database resource.
///
/// # Summary
///
/// * `Database` has public members instead of accessor methods because there
///   are no invariants restricting the data.
///
/// * `Database` implements `Deserialize`.
///
/// # Remarks
///
/// An application may obtain a database resource by sending an HTTP request to
/// GET `/{db}`.
///
/// # Compatibility
///
/// `Database` contains a dummy private member in order to prevent applications
/// from directly constructing a `Database` instance. This allows new fields to
/// be added to `Database` in future releases without it being a breaking
/// change.
///
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize)]
pub struct Database {
    pub committed_update_seq: u64,
    pub compact_running: bool,
    pub db_name: DatabaseName,
    pub disk_format_version: i32,
    pub data_size: u64,
    pub disk_size: u64,
    pub doc_count: u64,
    pub doc_del_count: u64,

    #[serde(deserialize_with = "deserialize_instance_start_time")]
    pub instance_start_time: u64,

    pub purge_seq: u64,
    pub update_seq: u64,

    #[serde(default = "PhantomData::default")]
    _private_guard: PhantomData<()>,
}

fn deserialize_instance_start_time<'a, D: Deserializer<'a>>(deserializer: D) -> Result<u64, D::Error> {

    struct Visitor;

    impl<'b> serde::de::Visitor<'b> for Visitor {
        type Value = u64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            f.write_str("a string specifying the CouchDB start time")
        }

        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
            u64::from_str_radix(s, 10).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(s), &self))
        }
    }

    deserializer.deserialize_str(Visitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::marker::PhantomData;

    #[test]
    fn database_deserializes_ok() {

        let source = r#"{
            "committed_update_seq": 292786,
            "compact_running": false,
            "data_size": 65031503,
            "db_name": "receipts",
            "disk_format_version": 6,
            "disk_size": 137433211,
            "doc_count": 6146,
            "doc_del_count": 64637,
            "instance_start_time": "1376269325408900",
            "purge_seq": 0,
            "update_seq": 292786
        }"#;

        let expected = Database {
            committed_update_seq: 292786,
            compact_running: false,
            data_size: 65031503,
            db_name: DatabaseName::from("receipts"),
            disk_format_version: 6,
            disk_size: 137433211,
            doc_count: 6146,
            doc_del_count: 64637,
            instance_start_time: 1376269325408900,
            purge_seq: 0,
            update_seq: 292786,
            _private_guard: PhantomData,
        };

        let got: Database = serde_json::from_str(source).unwrap();
        assert_eq!(got, expected);
    }
}
