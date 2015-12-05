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
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {

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

        impl serde::Deserialize for Field
        {
            fn deserialize<D>(d: &mut D) -> Result<Field, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor
                {
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

            fn visit_map<V>(&mut self, mut visitor: V)
                -> Result<Self::Value, V::Error> where V: serde::de::MapVisitor
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
                        },
                        Some(Field::CompactRunning) => {
                            compact_running = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DbName) => {
                            db_name = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DiskFormatVersion) => {
                            disk_format_version = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DataSize) => {
                            data_size = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DiskSize) => {
                            disk_size = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DocCount) => {
                            doc_count = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::DocDelCount) => {
                            doc_del_count = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::InstanceStartTime) => {
                            instance_start_time = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::PurgeSeq) => {
                            purge_seq = Some(try!(visitor.visit_value()));
                        },
                        Some(Field::UpdateSeq) => {
                            update_seq = Some(try!(visitor.visit_value()));
                        },
                        None => { break; },
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

        static FIELDS: &'static [&'static str] = &[
            "committed_update_seq",
            "compact_running",
            "db_name",
            "disk_format_version",
            "data_size",
            "disk_size",
            "doc_count",
            "doc_del_count",
            "instance_start_time",
            "purge_seq",
            "update_seq",
        ];
        d.visit_struct("Database", FIELDS, Visitor)
    }
}

#[cfg(test)]
mod tests {

    use serde_json;
    use std;

    use super::*;

    #[test]
    fn test_database_serialization() {

        // This function tests deserialization of the JSON string multiple
        // times, one time with the full object, which should succeed, and each
        // other time with one of the fields missing, which should fail. We
        // construct these JSON strings at runtime, using the following array
        // and helper functions.

        let fields = [
            r#""db_name": "stuff""#,
            r#""doc_count": 1"#,
            r#""doc_del_count": 2"#,
            r#""update_seq": 3"#,
            r#""purge_seq": 4"#,
            r#""compact_running": false"#,
            r#""disk_size": 5"#,
            r#""data_size": 6"#,
            r#""instance_start_time": "1234""#,
            r#""disk_format_version": 7"#,
            r#""committed_update_seq": 8"#,
        ];

        fn append(mut acc: String, item: & &str) -> String {
            if !acc.ends_with("{") {
                acc.push_str(", ");
            }
            acc.push_str(item);
            acc
        }

        fn join_json_string<'a, I, J>(head: I, tail: J) -> String
            where I: Iterator<Item = &'a &'a str>,
                  J: Iterator<Item = &'a &'a str>
        {
            let s = head.fold("{".to_string(), append);
            let s = tail.fold(s, append);
            let mut s = s;
            s.push_str("}");
            s
        }

        let complete_json_string = || {
            join_json_string(std::iter::empty(), fields.into_iter())
        };

        let incomplete_json_string = |key| {
            let key = format!(r#""{}""#, key);
            let pos = fields.into_iter().position(|&item| {
                item.starts_with(&key)
            }).unwrap();
            join_json_string(
                fields.into_iter().take(pos),
                fields.into_iter().skip(pos+1))
        };

        // Verify: All fields present.
        let s = complete_json_string();
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
        let s = incomplete_json_string("db_name");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("doc_count");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("doc_del_count");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("update_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("purge_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("compact_running");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("disk_size");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("data_size");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("instance_start_time");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("disk_format_version");
        assert!(serde_json::from_str::<Database>(&s).is_err());
        let s = incomplete_json_string("committed_update_seq");
        assert!(serde_json::from_str::<Database>(&s).is_err());
    }
}
