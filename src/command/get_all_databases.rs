use hyper;

use client;
use error::{self, Error};

/// Command to get all database names.
pub struct GetAllDatabases<'a> {
    client_state: &'a client::ClientState,
}

impl<'a> GetAllDatabases<'a> {

    pub fn new(client_state: &client::ClientState) -> GetAllDatabases {
        GetAllDatabases {
            client_state: client_state,
        }
    }

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// This command has no specific errors.
    ///
    pub fn run(self) -> Result<Vec<String>, Error> {

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = "_all_dbs".to_string();
            try!(
                self.client_state.http_client.get(u)
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
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                Ok(try!(client::decode_json::<Vec<String>>(&s)))
            },
            _ => Err(Error::UnexpectedHttpStatus {
                got: resp.status,
            })
        }
    }
}
