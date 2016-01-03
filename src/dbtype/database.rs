use serde;

use DatabaseName;

/// Database meta-information, as returned from a command to GET a database.
///
/// Although the `Database` type implements the `Ord` and `PartialOrd` traits,
/// it provides no guarantees how that ordering is defined and may change the
/// definition between any two releases of the couchdb crate. That is, for two
/// `Database` values `a` and `b`, the expression `a < b` may hold true now but
/// not in a subsequent release. Consequently, applications must not rely upon
/// any particular ordering definition.
///
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Database {
    /// The name of the database.
    pub db_name: DatabaseName,

    /// The current number of updates to the database.
    pub update_seq: u64,

    /// The number of committed updates.
    pub committed_update_seq: u64,

    /// Timestamp of when the database was opened, expressed in microseconds
    /// since the epoch.
    pub instance_start_time: u64,

    /// The version of the physical format used for the data when it is stored
    /// on disk.
    pub disk_format_version: i32,

    /// Number of documents in the database.
    pub doc_count: u64,

    /// Number of deleted documents.
    pub doc_del_count: u64,

    /// Actual data size, in bytes, of the database data.
    pub data_size: u64,

    /// Size, in bytes, of the data as stored on the disk. Views indexes are not
    /// included in the calculation.
    pub disk_size: u64,

    /// The number of purge operations on the database.
    pub purge_seq: u64,

    /// Set to true if the database compaction routine is operating on this
    /// database.
    pub compact_running: bool,
}

impl serde::Deserialize for Database {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        enum Field {
            CommittedUpdateSeq,
            CompactRunning,
            DbName,
            DiskFormatVersion,
            DataSize,
            DiskSize,
            DocCount,
            DocDelCount,
            InstanceStartTime,
            PurgeSeq,
            UpdateSeq,
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
                            "committed_update_seq" => Ok(Field::CommittedUpdateSeq),
                            "compact_running" => Ok(Field::CompactRunning),
                            "db_name" => Ok(Field::DbName),
                            "disk_format_version" => Ok(Field::DiskFormatVersion),
                            "data_size" => Ok(Field::DataSize),
                            "disk_size" => Ok(Field::DiskSize),
                            "doc_count" => Ok(Field::DocCount),
                            "doc_del_count" => Ok(Field::DocDelCount),
                            "instance_start_time" => Ok(Field::InstanceStartTime),
                            "purge_seq" => Ok(Field::PurgeSeq),
                            "update_seq" => Ok(Field::UpdateSeq),
                            _ => Err(E::unknown_field(value)),
                        }
                    }
                }

                d.visit(Visitor)
            }
        }

        struct Visitor;

        impl serde::de::Visitor for Visitor {
            type Value = Database;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut committed_update_seq = None;
                let mut compact_running = None;
                let mut db_name = None;
                let mut disk_format_version = None;
                let mut data_size = None;
                let mut disk_size = None;
                let mut doc_count = None;
                let mut doc_del_count = None;
                let mut instance_start_time = None;
                let mut purge_seq = None;
                let mut update_seq = None;
                loop {
                    match try!(visitor.visit_key()) {
                        Some(Field::CommittedUpdateSeq) => {
                            committed_update_seq = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::CompactRunning) => {
                            compact_running = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DbName) => {
                            db_name = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DiskFormatVersion) => {
                            disk_format_version = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DataSize) => {
                            data_size = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DiskSize) => {
                            disk_size = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DocCount) => {
                            doc_count = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::DocDelCount) => {
                            doc_del_count = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::InstanceStartTime) => {
                            instance_start_time = Some(try!(visitor.visit_value::<String>()));
                        }
                        Some(Field::PurgeSeq) => {
                            purge_seq = Some(try!(visitor.visit_value()));
                        }
                        Some(Field::UpdateSeq) => {
                            update_seq = Some(try!(visitor.visit_value()));
                        }
                        None => {
                            break;
                        }
                    }
                }

                try!(visitor.end());

                let committed_update_seq = match committed_update_seq {
                    Some(x) => x,
                    None => try!(visitor.missing_field("committed_update_seq")),
                };

                let compact_running = match compact_running {
                    Some(x) => x,
                    None => try!(visitor.missing_field("compact_running")),
                };

                let db_name = match db_name {
                    Some(x) => x,
                    None => try!(visitor.missing_field("db_name")),
                };

                let disk_format_version = match disk_format_version {
                    Some(x) => x,
                    None => try!(visitor.missing_field("disk_format_version")),
                };

                let data_size = match data_size {
                    Some(x) => x,
                    None => try!(visitor.missing_field("data_size")),
                };

                let disk_size = match disk_size {
                    Some(x) => x,
                    None => try!(visitor.missing_field("disk_size")),
                };

                let doc_count = match doc_count {
                    Some(x) => x,
                    None => try!(visitor.missing_field("doc_count")),
                };

                let doc_del_count = match doc_del_count {
                    Some(x) => x,
                    None => try!(visitor.missing_field("doc_del_count")),
                };

                let instance_start_time = match instance_start_time {
                    Some(x) => {
                        try!(u64::from_str_radix(&x, 10).map_err(|e| {
                            use std::error::Error;
                            use serde::de::Error as SerdeError;
                            V::Error::invalid_value(e.description())
                        }))
                    }
                    None => try!(visitor.missing_field("instance_start_time")),
                };

                let purge_seq = match purge_seq {
                    Some(x) => x,
                    None => try!(visitor.missing_field("purge_seq")),
                };

                let update_seq = match update_seq {
                    Some(x) => x,
                    None => try!(visitor.missing_field("update_seq")),
                };

                Ok(Database {
                    committed_update_seq: committed_update_seq,
                    compact_running: compact_running,
                    db_name: db_name,
                    disk_format_version: disk_format_version,
                    data_size: data_size,
                    disk_size: disk_size,
                    doc_count: doc_count,
                    doc_del_count: doc_del_count,
                    instance_start_time: instance_start_time,
                    purge_seq: purge_seq,
                    update_seq: update_seq,
                })
            }
        }

        static FIELDS: &'static [&'static str] = &["committed_update_seq",
                                                   "compact_running",
                                                   "db_name",
                                                   "disk_format_version",
                                                   "data_size",
                                                   "disk_size",
                                                   "doc_count",
                                                   "doc_del_count",
                                                   "instance_start_time",
                                                   "purge_seq",
                                                   "update_seq"];
        d.visit_struct("Database", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;

    use Database;
    use DatabaseName;

    #[test]
    fn database_deserialization_with_all_fields() {
        let expected = Database {
            committed_update_seq: 1,
            compact_running: true,
            db_name: DatabaseName::from("foo"),
            disk_format_version: 2,
            data_size: 3,
            disk_size: 4,
            doc_count: 5,
            doc_del_count: 6,
            instance_start_time: 7,
            purge_seq: 8,
            update_seq: 9,
        };
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str(&s).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn database_deserialization_with_no_committed_update_seq_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "committed_update_seq");
    }

    #[test]
    fn database_deserialization_with_no_compact_running_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "compact_running");
    }

    #[test]
    fn database_deserialization_with_no_db_name_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "db_name");
    }

    #[test]
    fn database_deserialization_with_no_disk_format_version_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "disk_format_version");
    }

    #[test]
    fn database_deserialization_with_no_data_size_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "data_size");
    }

    #[test]
    fn database_deserialization_with_no_disk_size_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "disk_size");
    }

    #[test]
    fn database_deserialization_with_no_doc_count_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "doc_count");
    }

    #[test]
    fn database_deserialization_with_no_doc_del_count_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "doc_del_count");
    }

    #[test]
    fn database_deserialization_with_no_instance_start_time_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "instance_start_time");
    }

    #[test]
    fn database_deserialization_with_bad_instance_start_time_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "not_a_number")
                         .insert("purge_seq", 8)
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_invalid_value!(got);
    }

    #[test]
    fn database_deserialization_with_no_purge_seq_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("update_seq", 9)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "purge_seq");
    }

    #[test]
    fn database_deserialization_with_no_update_seq_field() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("committed_update_seq", 1)
                         .insert("compact_running", true)
                         .insert("db_name", "foo")
                         .insert("disk_format_version", 2)
                         .insert("data_size", 3)
                         .insert("disk_size", 4)
                         .insert("doc_count", 5)
                         .insert("doc_del_count", 6)
                         .insert("instance_start_time", "7")
                         .insert("purge_seq", 8)
                         .unwrap();
        let s = serde_json::to_string(&source).unwrap();
        let got = serde_json::from_str::<Database>(&s);
        expect_json_error_missing_field!(got, "update_seq");
    }
}
