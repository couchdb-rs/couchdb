use hyper;

use client;
use error::{self, Error};

/// Command to delete a database.
pub struct DeleteDatabase<'a, 'b> {
    client_state: &'a client::ClientState,
    db_name: &'b str,
}

impl<'a, 'b> DeleteDatabase<'a, 'b> {

    pub fn new(client_state: &'a client::ClientState, db_name: &'b str) -> DeleteDatabase<'a, 'b> {
        DeleteDatabase {
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
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(), Error> {

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = self.db_name.to_string();
            try!(
                self.client_state.http_client.delete(u)
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
            hyper::status::StatusCode::Ok =>
                Ok(try!(client::require_content_type_application_json(&resp.headers))),
            hyper::status::StatusCode::BadRequest =>
                // The CouchDB spec says this status may also mean the document id has been
                // "forgotten"--whatever that means!
                Err(error::new_because_invalid_database_name(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
