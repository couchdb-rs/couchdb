use hyper;

use client::ClientState;
use dbpath::DatabasePath;
use error::Error;
use transport::{self, Command, Request};

/// Command to get all database names.
pub struct GetAllDatabases<'a> {
    client_state: &'a ClientState,
}

impl<'a> GetAllDatabases<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState) -> Self {
        GetAllDatabases { client_state: client_state }
    }

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
        let req = try!(Request::new(hyper::Get, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response, _state: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(transport::content_type_must_be_application_json(&resp.headers));
                transport::decode_json::<_, Vec<DatabasePath>>(resp)
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
