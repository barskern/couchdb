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
/// The `Server` type is provided for testing purposes. The database persists to
/// the system's default temporary directory (e.g., `/tmp`) and is deleted when
/// the `Server` instance drops.
///
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

        let tmp_root = try!(tempdir::TempDir::new("couchdb_client_test").or_else(|e| {
            Err(Error::Io {
                description: "Failed to create temporary directory for CouchDB server",
                cause: e,
            })
        }));

        {
            use std::io::Write;
            let path = tmp_root.path().join("couchdb.conf");
            let mut f = try!(std::fs::File::create(&path).or_else(|e| {
                Err(Error::Io {
                    description: "Failed to open CouchDB server configuration file",
                    cause: e,
                })
            }));
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
                  }));
        }

        let mut process = AutoKillProcess(try!(new_test_server_command(&tmp_root)
                                                   .spawn()
                                                   .or_else(|e| {
                                                       Err(Error::Io {
                                                           description: "Failed to spawn CouchDB server process",
                                                           cause: e,
                                                       })
                                                   })));

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
                if line.is_empty() {
                    break;
                }
            }
        });

        // Wait for CouchDB server to start its HTTP service.
        let uri = try!(rx.recv()
                         .or_else(|e| {
                             t.join().unwrap_err();
                             Err(Error::ReceiveFromThread {
                                 description: "Failed to extract URI from CouchDB server",
                                 cause: e,
                             })
                         }));

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

#[cfg(any(windows))]
fn new_test_server_command(tmp_root: &tempdir::TempDir) -> std::process::Command {
    // Getting a one-shot CouchDB server running on Windows is tricky:
    // http://stackoverflow.com/questions/11812365/how-to-use-a-custom-couch-ini-on-windows
    //
    // TODO: Support CouchDB being installed in a non-default directory.

    let mut c = std::process::Command::new("erl");
    c.arg("-couch_ini");
    c.arg("c:/program files (x86)/apache software foundation/couchdb/etc/couchdb/default.ini");
    c.arg("c:/program files (x86)/apache software foundation/couchdb/etc/couchdb/local.ini");
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
