use regex;
use std;
use tempdir;

use error::Error;

/// RAII wrapper for a child process that kills the process when dropped.
struct AutoKillProcess(std::process::Child);

impl Drop for AutoKillProcess {
    fn drop(&mut self) {
        let AutoKillProcess(ref mut process) = *self;
        process.kill().unwrap();
        process.wait().unwrap();
    }
}

/// RAII wrapper for running a CouchDB server process.
///
/// # Remarks
///
/// The Server type is provided for testing purposes.
pub struct Server {
    // Rust drops structure fields in forward order, not reverse order. The child process must exit
    // before we remove the temporary directory.
    _process: AutoKillProcess,
    _tmp_root: tempdir::TempDir,
    uri: String,
}

impl Server {

    /// Spawn a CouchDB server process.
    pub fn new() -> Result<Server, Error> {

        let tmp_root = try!(
            tempdir::TempDir::new("couchdb_client_test")
            .or_else(|e| {
                Err(Error::Io {
                    description: "Failed to create temporary directory for CouchDB server",
                    cause: e,
                })
            })
        );

        {
            use std::io::Write;
            let path = tmp_root.path().join("couchdb.conf");
            let mut f = try!(
                std::fs::File::create(&path)
                .or_else(|e| {
                    Err(Error::Io {
                        description: "Failed to open CouchDB server configuration file",
                        cause: e,
                    })
                })
            );
            try!(f.write_all(b"[couchdb]\n\
                database_dir = var\n\
                uri_file = couchdb.uri\n\
                view_index_dir = view\n\
                \n\
                [log]\n\
                file = couchdb.log\n\
                \n\
                [httpd]\n\
                port = 0\n\
                ")
                .or_else(|e| {
                    Err(Error::Io {
                        description: "Failed to write CouchDB server configuration file",
                        cause: e,
                    })
                })
            );
        }

        let mut process = AutoKillProcess(
            try!(
                std::process::Command::new("couchdb")
                .arg("-a")
                .arg("couchdb.conf")
                .current_dir(tmp_root.path())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .or_else(|e| {
                    Err(Error::Io {
                        description: "Failed to spawn CouchDB server process",
                        cause: e
                    })
                })
            )
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let mut process_out;
        {
            let AutoKillProcess(ref mut process) = process;
            let stdout = std::mem::replace(&mut process.stdout, None).unwrap();
            process_out = std::io::BufReader::new(stdout);
        }

        let t = std::thread::spawn(move || {

            let re = regex::Regex::new(r"Apache CouchDB has started on (http.*)").unwrap();
            let mut line = String::new();

            loop {
                use std::io::BufRead;
                line.clear();
                process_out.read_line(&mut line).unwrap();
                let line = line.trim_right();
                match re.captures(line) {
                    None => (),
                    Some(caps) => {
                        tx.send(caps.at(1).unwrap().to_string()).unwrap();
                        break;
                    }
                }
            }

            // Drain stdout.
            loop {
                use std::io::BufRead;
                line.clear();
                process_out.read_line(&mut line).unwrap();
            }
        });

        // Wait for CouchDB server to start its HTTP service.
        let uri = try!(
            rx.recv()
            .or_else(|e| {
                t.join().unwrap_err();
                Err(Error::ReceiveFromThread {
                    description: "Failed to extract URI from CouchDB server",
                    cause: e
                })
            })
        );

        Ok(Server {
            _process: process,
            _tmp_root: tmp_root,
            uri: uri,
        })
    }

    /// Get the CouchDB server URI.
    pub fn uri(&self) -> &str {
        &self.uri
    }
}
