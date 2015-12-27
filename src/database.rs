use std;

use dbpath::DatabasePath;
use dbtype;
use error::{Error, DecodeErrorKind};

/// Database resource, as returned from a command to GET a database.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Database {
    /// The path of the database—the value for specifying the location of this
    /// database in CouchDB commands.
    pub db_path: DatabasePath,

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

    /// Actual data size in bytes of the database data.
    pub data_size: u64,

    /// Size in bytes of the data as stored on the disk. Views indexes are not
    /// included in the calculation.
    pub disk_size: u64,

    /// The number of purge operations on the database.
    pub purge_seq: u64,

    /// Set to true if the database compaction routine is operating on this
    /// database.
    pub compact_running: bool,
}

impl Database {
    #[doc(hidden)]
    pub fn from_db_database(mut db: dbtype::Database) -> Result<Self, Error> {

        // CouchDB returns the `instance_start_time` field as a string, not a
        // number. We convert it to a number for stronger type-safety.

        let instance_start_time = {
            let s = std::mem::replace(&mut db.instance_start_time, String::new());
            try!(u64::from_str_radix(&s, 10)
                     .map_err(|e| Error::Decode(DecodeErrorKind::InstanceStartTime { got: s, cause: e })))
        };

        let db = Database {
            db_path: DatabasePath::from(db.db_name),
            update_seq: db.update_seq,
            committed_update_seq: db.committed_update_seq,
            instance_start_time: instance_start_time,
            disk_format_version: db.disk_format_version,
            doc_count: db.doc_count,
            doc_del_count: db.doc_del_count,
            data_size: db.data_size,
            disk_size: db.disk_size,
            purge_seq: db.purge_seq,
            compact_running: db.compact_running,
        };

        Ok(db)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dbtype;
    use error::{DecodeErrorKind, Error};

    #[test]
    fn test_database_from_db_type() {

        // Verify: Conversion succeeds.

        let db = dbtype::Database {
            committed_update_seq: 1,
            compact_running: true,
            db_name: "stuff".into(),
            disk_format_version: 2,
            data_size: 3,
            disk_size: 4,
            doc_count: 5,
            doc_del_count: 6,
            instance_start_time: "7".into(),
            purge_seq: 8,
            update_seq: 9,
        };

        let db = Database::from_db_database(db).unwrap();
        assert_eq!(db.db_path, "stuff".into());
        assert_eq!(db.update_seq, 9);
        assert_eq!(db.committed_update_seq, 1);
        assert_eq!(db.instance_start_time, 7);
        assert_eq!(db.disk_format_version, 2);
        assert_eq!(db.doc_count, 5);
        assert_eq!(db.doc_del_count, 6);
        assert_eq!(db.data_size, 3);
        assert_eq!(db.disk_size, 4);
        assert_eq!(db.purge_seq, 8);
        assert_eq!(db.compact_running, true);

        // Verify: Bad `instance_start_time` field.

        let db = dbtype::Database {
            committed_update_seq: 1,
            compact_running: true,
            db_name: "stuff".into(),
            disk_format_version: 2,
            data_size: 3,
            disk_size: 4,
            doc_count: 5,
            doc_del_count: 6,
            instance_start_time: "not_a_valid_number".into(),
            purge_seq: 8,
            update_seq: 9,
        };

        let e = Database::from_db_database(db).unwrap_err();
        match e {
            Error::Decode(kind) => {
                match kind {
                    DecodeErrorKind::InstanceStartTime { .. } => (),
                    _ => {
                        panic!("Got unexpected error kind: {}", kind);
                    }
                }
            }
            _ => {
                panic!("Got unexpected error: {}", e);
            }
        }
    }
}
