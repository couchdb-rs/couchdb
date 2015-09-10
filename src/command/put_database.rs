use hyper;

use client;
use error::{self, Error};

/// Command to create a database.
pub struct PutDatabase<'a> {
    client_state: &'a client::ClientState,
    db_name: &'a str,
}

impl<'a> PutDatabase<'a> {

    pub fn new(
        client_state: &'a client::ClientState,
        db_name: &'a str)
        -> PutDatabase<'a>
    {
        PutDatabase {
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
    /// * `Error::DatabaseExists`: The database already exists.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(), Error> {
        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = self.db_name.to_string();
            try!(
                self.client_state.http_client.put(u)
                .header(hyper::header::Accept(vec![
                        hyper::header::qitem(
                            Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
                .send()
                .or_else(|e| {
                    Err(Error::Transport {
                        cause: error::TransportCause::Hyper(e),
                    })
                })
            )
        };

        match resp.status {
            hyper::status::StatusCode::Created =>
                Ok(try!(client::require_content_type_application_json(&resp.headers))),
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_database_name(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::PreconditionFailed =>
                Err(error::new_because_database_exists(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus {
                got: resp.status,
            })
        }
    }
}
