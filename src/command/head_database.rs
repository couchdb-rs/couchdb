use hyper;

use client::ClientState;
use error::{self, Error};

#[doc(hidden)]
pub fn new_head_database<'a>(
    client_state: &'a ClientState,
    db_name: &'a str)
    -> HeadDatabase<'a>
{
    HeadDatabase {
        client_state: client_state,
        db_name: db_name,
    }
}

/// Command to get database meta-information.
pub struct HeadDatabase<'a> {
    client_state: &'a ClientState,
    db_name: &'a str,
}

impl<'a> HeadDatabase<'a> {

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The database does not exist.
    ///
    pub fn run(self) -> Result<(), Error> {
        let resp = {
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = self.db_name.to_string();
            try!(
                self.client_state.http_client.head(u)
                .send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: None } ),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}


