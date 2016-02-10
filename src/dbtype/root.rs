use semver;
use serde;
use std;
use uuid;

use dbtype;

#[derive(Debug, PartialEq)]
struct SerializableVersion(semver::Version);

impl serde::Deserialize for SerializableVersion {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = SerializableVersion;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                use std::error::Error;
                semver::Version::parse(v)
                    .map(|v| SerializableVersion(v))
                    .map_err(|e| E::invalid_value(e.description()))
            }
        }

        deserializer.visit(Visitor)
    }
}

/// CouchDB server vendor information.
#[derive(Debug, PartialEq)]
pub struct Vendor {
    _dummy: std::marker::PhantomData<()>,

    /// Name of the CouchDB server vendor.
    pub name: String,

    /// Version of the CouchDB server.
    pub version: semver::Version,
}

impl serde::Deserialize for Vendor {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Name,
            Version,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "version" => Ok(Field::Version),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Vendor;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut name = None;
                let mut version = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Name) => {
                            name = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Version) => {
                            version = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let name = match name {
                    Some(x) => x,
                    None => try!(visitor.missing_field("name")),
                };

                let version = match version {
                    Some(x) => x,
                    None => try!(visitor.missing_field("version")),
                };
                let SerializableVersion(version) = version;

                Ok(Vendor {
                    _dummy: std::marker::PhantomData,
                    name: name,
                    version: version,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["name", "version"];
        deserializer.visit_struct("Vendor", FIELDS, Visitor)
    }
}

/// CouchDB server information.
#[derive(Debug, PartialEq)]
pub struct Root {
    _dummy: std::marker::PhantomData<()>,

    /// Welcome message returned by the CouchDB server.
    pub couchdb: String,

    /// Universally unique identifier for the CouchDB server.
    pub uuid: uuid::Uuid,

    /// Vendor information for the CouchDB server.
    pub vendor: Vendor,

    /// Version of the CouchDB server.
    pub version: semver::Version,
}

impl serde::Deserialize for Root {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            Couchdb,
            Uuid,
            Vendor,
            Version,
        }

        impl serde::Deserialize for Field {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = Field;

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error
                    {
                        match value {
                            "couchdb" => Ok(Field::Couchdb),
                            "uuid" => Ok(Field::Uuid),
                            "vendor" => Ok(Field::Vendor),
                            "version" => Ok(Field::Version),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Root;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut couchdb = None;
                let mut uuid = None;
                let mut vendor = None;
                let mut version = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::Couchdb) => {
                            couchdb = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Uuid) => {
                            uuid = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Vendor) => {
                            vendor = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::Version) => {
                            version = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let couchdb = match couchdb {
                    Some(x) => x,
                    None => try!(visitor.missing_field("couchdb")),
                };

                let uuid = match uuid {
                    Some(x) => x,
                    None => try!(visitor.missing_field("uuid")),
                };
                let dbtype::Uuid(uuid) = uuid;

                let vendor = match vendor {
                    Some(x) => x,
                    None => try!(visitor.missing_field("vendor")),
                };

                let version = match version {
                    Some(x) => x,
                    None => try!(visitor.missing_field("version")),
                };
                let SerializableVersion(version) = version;

                Ok(Root {
                    _dummy: std::marker::PhantomData,
                    couchdb: couchdb,
                    uuid: uuid,
                    vendor: vendor,
                    version: version,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["couchdb", "uuid", "vendor", "version"];
        deserializer.visit_struct("Root", FIELDS, Visitor)
    }
}

// RootBuilder is a quick-and-dirty to facilitate testing.
#[allow(dead_code)]
pub struct RootBuilder {
    target: Root,
}

impl RootBuilder {
    #[allow(dead_code)]
    pub fn new(couchdb: &str, uuid: &str, vendor: &str, version: &str) -> Self {
        RootBuilder {
            target: Root {
                _dummy: std::marker::PhantomData,
                couchdb: couchdb.to_string(),
                uuid: uuid.parse().unwrap(),
                vendor: Vendor {
                    _dummy: std::marker::PhantomData,
                    name: vendor.to_string(),
                    version: semver::Version::parse(version).unwrap(),
                },
                version: semver::Version::parse(version).unwrap(),
            },
        }
    }

    #[allow(dead_code)]
    pub fn unwrap(self) -> Root {
        self.target
    }
}

#[cfg(test)]
mod tests {

    use semver;
    use serde_json;
    use std;
    use uuid;

    use super::{Root, RootBuilder, SerializableVersion, Vendor};

    #[test]
    fn version_deserialization_ok() {
        let expected = SerializableVersion(semver::Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Vec::new(),
            build: Vec::new(),
        });
        let source = serde_json::Value::String("1.2.3".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn version_deserialization_nok_bad_value() {
        let source = serde_json::Value::String("bad_version".to_string());
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<SerializableVersion>(&s);
        expect_json_error_invalid_value!(got);
    }

    #[test]
    fn vendor_deserialization_ok() {
        let expected = Vendor {
            _dummy: std::marker::PhantomData,
            name: "The Apache Software Foundation".to_string(),
            version: semver::Version::parse("1.3.1").unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("name", "The Apache Software Foundation")
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn vendor_deserialization_nok_missing_name_field_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Vendor>(&s);
        expect_json_error_missing_field!(got, "name");
    }


    #[test]
    fn vendor_deserialization_nok_missing_version_field_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("name", "The Apache Software Foundation")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Vendor>(&s);
        expect_json_error_missing_field!(got, "version");
    }

    #[test]
    fn vendor_deserialization_nok_bad_version_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("name", "The Apache Software Foundation")
                         .insert("version", "bad version")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Vendor>(&s);
        expect_json_error_invalid_value!(got);
    }

    #[test]
    fn root_deserialization_ok() {
        use std::str::FromStr;
        let expected = Root {
            _dummy: std::marker::PhantomData,
            couchdb: "Welcome".to_string(),
            uuid: uuid::Uuid::from_str("85fb71bf700c17267fef77535820e371").unwrap(),
            vendor: Vendor {
                _dummy: std::marker::PhantomData,
                name: "The Apache Software Foundation".to_string(),
                version: semver::Version::parse("1.3.1").unwrap(),
            },
            version: semver::Version::parse("1.3.1").unwrap(),
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("couchdb", "Welcome")
                         .insert("uuid", "85fb71bf700c17267fef77535820e371")
                         .insert_object("vendor", |x| {
                             x.insert("name", "The Apache Software Foundation")
                              .insert("version", "1.3.1")
                         })
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn root_deserialization_nok_missing_couchdb_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("uuid", "85fb71bf700c17267fef77535820e371")
                         .insert_object("vendor", |x| {
                             x.insert("name", "The Apache Software Foundation")
                              .insert("version", "1.3.1")
                         })
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Root>(&s);
        expect_json_error_missing_field!(got, "rev");
    }

    #[test]
    fn root_deserialization_nok_missing_uuid_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("couchdb", "Welcome")
                         .insert_object("vendor", |x| {
                             x.insert("name", "The Apache Software Foundation")
                              .insert("version", "1.3.1")
                         })
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Root>(&s);
        expect_json_error_missing_field!(got, "rev");
    }

    #[test]
    fn root_deserialization_nok_missing_vendor_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("couchdb", "Welcome")
                         .insert("uuid", "85fb71bf700c17267fef77535820e371")
                         .insert("version", "1.3.1")
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Root>(&s);
        expect_json_error_missing_field!(got, "rev");
    }

    #[test]
    fn root_deserialization_nok_missing_version_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("couchdb", "Welcome")
                         .insert("uuid", "85fb71bf700c17267fef77535820e371")
                         .insert_object("vendor", |x| {
                             x.insert("name", "The Apache Software Foundation")
                              .insert("version", "1.3.1")
                         })
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Root>(&s);
        expect_json_error_missing_field!(got, "rev");
    }

    #[test]
    fn root_builder() {
        let expected = Root {
            _dummy: std::marker::PhantomData,
            couchdb: "Welcome".to_string(),
            uuid: "85fb71bf700c17267fef77535820e371".parse().unwrap(),
            vendor: Vendor {
                _dummy: std::marker::PhantomData,
                name: "The Apache Software Foundation".to_string(),
                version: semver::Version::parse("1.3.1").unwrap(),
            },
            version: semver::Version::parse("1.3.1").unwrap(),
        };
        let got = RootBuilder::new("Welcome",
                                   "85fb71bf700c17267fef77535820e371",
                                   "The Apache Software Foundation",
                                   "1.3.1")
                      .unwrap();
        assert_eq!(expected, got);
    }
}
