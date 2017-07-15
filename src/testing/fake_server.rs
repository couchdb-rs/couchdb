use {Error, regex, std, tempdir};

// RAII wrapper for a child process that kills the process when dropped.
struct AutoKillProcess(std::process::Child);

impl Drop for AutoKillProcess {
    fn drop(&mut self) {
        let AutoKillProcess(ref mut process) = *self;
        process.kill().unwrap();
        process.wait().unwrap();
    }
}

/// `FakeServer` manages a CouchDB server process, for application testing.
///
/// # Summary
///
/// * `FakeServer` is an RAII-wrapper for an external CouchDB server process.
///
/// * The external CouchDB process's underlying storage persists to the system's
///   default temporary directory (e.g., `/tmp`) and is deleted when the
///   `FakeServer` instance drops.
///
/// # Remarks
///
/// `FakeServer` is a fake, not a mock, meaning an application may use it to
/// send HTTP requests to a real CouchDB server and receive real responses.
/// Consequently, this means that CouchDB must be installed on the local machine
/// in order to use `FakeServer`.
///
/// The CouchDB server will open an unused port on the local machine. The
/// application may obtain the server's exact address via the `FakeServer::url`
/// method.
///
/// The CouchDB server remains up and running for the lifetime of the
/// `FakeServer` instance. When the instance drops, the server shuts down and
/// all of its data are deleted.
///
/// # Example
///
/// ```
/// extern crate couchdb;
/// extern crate tokio_core;
///
/// let server = couchdb::testing::FakeServer::new().unwrap();
///
/// let mut reactor = tokio_core::reactor::Core::new().unwrap();
/// let client = couchdb::Client::new(
///     server.url(),
///     couchdb::ClientOptions::default(),
///     &reactor.handle()
/// ).unwrap();
///
/// reactor.run(client.put_database("/baseball").send()).unwrap();
///
/// match reactor.run(client.head_database("/baseball").send()) {
///     Ok(_) => {}
///     x => panic!("Got unexpected result {:?}", x),
/// }
/// ```
///
pub struct FakeServer {
    // Rust drops structure fields in forward order, not reverse order. The
    // child process must exit before we remove the temporary directory.
    _process: AutoKillProcess,
    _tmp_root: tempdir::TempDir,
    url: String,
}

impl FakeServer {
    /// Spawns a CouchDB server process for testing.
    pub fn new() -> Result<FakeServer, Error> {

        let tmp_root = try!(tempdir::TempDir::new("couchdb_test").map_err(|e| {
            Error::from((
                "Failed to create temporary directory for CouchDB server",
                e,
            ))
        }));

        {
            use std::io::Write;
            let path = tmp_root.path().join("couchdb.conf");
            let mut f = try!(std::fs::File::create(&path).map_err(|e| {
                Error::from(("Failed to open CouchDB server configuration file", e))
            }));
            try!(
                f.write_all(
                    b"[couchdb]\n\
                database_dir = var\n\
                uri_file = couchdb.uri\n\
                view_index_dir = view\n\
                \n\
                [log]\n\
                file = couchdb.log\n\
                \n\
                [httpd]\n\
                port = 0\n\
                ",
                ).map_err(|e| {
                        Error::from(("Failed to write CouchDB server configuration file", e))
                    })
            );
        }

        let child = try!(new_test_server_command(&tmp_root).spawn().map_err(|e| {
            Error::from(("Failed to spawn CouchDB server process", e))
        }));
        let mut process = AutoKillProcess(child);

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
                        tx.send(caps.get(1).unwrap().as_str().to_owned()).unwrap();
                        break;
                    }
                }
            }

            // Drain stdout.
            loop {
                use std::io::BufRead;
                line.clear();
                process_out.read_line(&mut line).unwrap();
                if line.is_empty() {
                    break;
                }
            }
        });

        // Wait for the CouchDB server to start its HTTP service.
        let url = try!(rx.recv().map_err(|e| {
            t.join().unwrap_err();
            Error::from(("Failed to extract URL from CouchDB server", e))
        }));

        Ok(FakeServer {
            _process: process,
            _tmp_root: tmp_root,
            url: url,
        })
    }

    /// Returns the CouchDB server's URL.
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(any(windows))]
fn new_test_server_command(tmp_root: &tempdir::TempDir) -> std::process::Command {

    // Getting a one-shot CouchDB server running on Windows is tricky:
    // http://stackoverflow.com/questions/11812365/how-to-use-a-custom-couch-ini-on-windows
    //
    // TODO: Support CouchDB being installed in a non-default directory.

    let couchdb_dir = "c:/program files (x86)/apache software foundation/couchdb";

    let erl = format!("{}/bin/erl", couchdb_dir);
    let default_ini = format!("{}/etc/couchdb/default.ini", couchdb_dir);
    let local_ini = format!("{}/etc/couchdb/local.ini", couchdb_dir);

    let mut c = std::process::Command::new(erl);
    c.arg("-couch_ini");
    c.arg(default_ini);
    c.arg(local_ini);
    c.arg("couchdb.conf");
    c.arg("-s");
    c.arg("couch");
    c.current_dir(tmp_root.path());
    c.stdout(std::process::Stdio::piped());
    c
}

#[cfg(any(not(windows)))]
fn new_test_server_command(tmp_root: &tempdir::TempDir) -> std::process::Command {
    let mut c = std::process::Command::new("couchdb");
    c.arg("-a");
    c.arg("couchdb.conf");
    c.current_dir(tmp_root.path());
    c.stdout(std::process::Stdio::piped());
    c
}
