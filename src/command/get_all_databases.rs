use hyper;

use client::{self, ClientState};
use dbpath::DatabasePath;
use error::Error;
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_get_all_databases(client_state: &ClientState) -> GetAllDatabases {
    GetAllDatabases {
        client_state: client_state,
    }
}

/// Command to get all database names.
pub struct GetAllDatabases<'a> {
    client_state: &'a ClientState,
}

impl<'a> GetAllDatabases<'a> {

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// This command has no specific errors.
    ///
    pub fn run(self) -> Result<Vec<DatabasePath>, Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for GetAllDatabases<'a> {

    type Output = Vec<DatabasePath>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = {
            let mut uri = self.client_state.uri.clone();
            uri.path_mut().unwrap()[0] = "_all_dbs".to_string();
            uri
        };
        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json();
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                Ok(try!(client::decode_json::<Vec<DatabasePath>>(&s)))
            },
            _ => Err(Error::UnexpectedHttpStatus {
                got: resp.status,
            })
        }
    }
}
