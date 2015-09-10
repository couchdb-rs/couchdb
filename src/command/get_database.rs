use hyper;
use serde_json;
use std;

use client;
use database::Database;
use error::{self, Error};

/// Command to get a database.
pub struct GetDatabase<'a> {
    client_state: &'a client::ClientState,
    db_name: &'a str,
}

impl<'a> GetDatabase<'a> {

    pub fn new(
        client_state: &'a client::ClientState,
        db_name: &'a str)
        -> GetDatabase<'a>
    {
        GetDatabase {
            client_state: client_state,
            db_name: db_name,
        }
    }

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The database does not exist.
    ///
    pub fn run(self) -> Result<Database, Error> {

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = self.db_name.to_string();
            try!(
                self.client_state.http_client.get(u)
                .header(hyper::header::Accept(vec![
                    hyper::header::qitem(
                        Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
                .send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };

        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let mut resp_body = try!(client::decode_json::<serde_json::Value>(&s));
                (|| {
                    let dot = match resp_body.as_object_mut() {
                        None => { return None; },
                        Some(x) => x,
                    };
                    let doc_count = match dot.get("doc_count") {
                        None => { return None; },
                        Some(x) => match x.as_u64() {
                            None => { return None; },
                            Some(x) => x,
                        },
                    };
                    let doc_del_count = match dot.get("doc_del_count") {
                        None => { return None; },
                        Some(x) => match x.as_u64() {
                            None => { return None; },
                            Some(x) => x,
                        },
                    };
                    let db_name = match dot.get_mut("db_name") {
                        None => { return None; },
                        Some(x) => match *x {
                            serde_json::Value::String(ref mut x) =>
                                std::mem::replace(x, String::new()),
                            _ => { return None; },
                        },
                    };
                    Some(Database {
                        doc_count: doc_count,
                        doc_del_count: doc_del_count,
                        db_name: db_name,
                    })
                })()
                .ok_or(Error::UnexpectedContent { got: s } )
            },
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
