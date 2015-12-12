use serde;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Database {
    pub committed_update_seq: u64,
    pub compact_running: bool,
    pub db_name: String,
    pub disk_format_version: i32,
    pub data_size: u64,
    pub disk_size: u64,
    pub doc_count: u64,
    pub doc_del_count: u64,
    pub instance_start_time: String,
    pub purge_seq: u64,
    pub update_seq: u64,
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
                            instance_start_time = Some(try!(visitor.visit_value()));
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
                    Some(x) => x,
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

    use super::*;
    use jsontest;

    #[test]
    fn test_deserialization() {

        let fields = [r#""db_name": "stuff""#,
                      r#""doc_count": 1"#,
                      r#""doc_del_count": 2"#,
                      r#""update_seq": 3"#,
                      r#""purge_seq": 4"#,
                      r#""compact_running": false"#,
                      r#""disk_size": 5"#,
                      r#""data_size": 6"#,
                      r#""instance_start_time": "1234""#,
                      r#""disk_format_version": 7"#,
                      r#""committed_update_seq": 8"#];

        // Verify: All fields present.
        let s = jsontest::make_complete_json_object(&fields);
        let v = serde_json::from_str::<Database>(&s).unwrap();
        assert_eq!(v.committed_update_seq, 8);
        assert_eq!(v.compact_running, false);
        assert_eq!(v.db_name, "stuff".to_string());
        assert_eq!(v.disk_format_version, 7);
        assert_eq!(v.data_size, 6);
        assert_eq!(v.disk_size, 5);
        assert_eq!(v.doc_count, 1);
        assert_eq!(v.doc_del_count, 2);
        assert_eq!(v.instance_start_time, "1234".to_string());
        assert_eq!(v.purge_seq, 4);
        assert_eq!(v.update_seq, 3);

        // Verify: Each field missing, one at a time.
        let s = jsontest::make_json_object_with_missing_field(&fields, "db_name");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "doc_count");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "doc_del_count");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "update_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "purge_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "compact_running");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "disk_size");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "data_size");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "instance_start_time");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "disk_format_version");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = jsontest::make_json_object_with_missing_field(&fields, "committed_update_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
    }
}
