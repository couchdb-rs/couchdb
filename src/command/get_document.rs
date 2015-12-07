use hyper;
use serde;
use std;

use client::ClientState;
use dbpath::DatabasePath;
use docpath::DocumentPath;
use document::{self, Document};
use error::{Error, ErrorResponse};
use revision::Revision;
use transport::{self, Command, Request};

/// Command to get a document.
pub struct GetDocument<'a, T> where T: serde::Deserialize
{
    client_state: &'a ClientState,
    path: DocumentPath,
    if_none_match: Option<&'a Revision>,
    _content_type: std::marker::PhantomData<T>,
}

impl<'a, T> GetDocument<'a, T> where T: serde::Deserialize
{
    #[doc(hidden)]
    pub fn new_get_document(client_state: &'a ClientState, path: DocumentPath)
        -> Self
    {
        GetDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
            _content_type: std::marker::PhantomData,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    /// Send the command request and wait for the response.
    ///
    /// # Return
    ///
    /// Return `None` if an If-None-Match revision is given and the document
    /// hasn't been modified since that revision. Otherwise, return `Some` with
    /// the document meta-information and content.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The document does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<Option<Document<T>>, Error> {
        transport::run_command(self)
    }
}

impl<'a, T> Command for GetDocument<'a, T>
    where T: serde::Deserialize
{
    type Output = Option<Document<T>>;
    type State = DatabasePath;

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = self.path.database_path().clone();
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json()
            .if_none_match_revision(self.if_none_match);
        Ok((req, db_path))
    }

    fn take_response(resp: hyper::client::Response, db_path: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(transport::content_type_must_be_application_json(&resp.headers));
                let doc = try!(document::document_from_json(resp, db_path));
                Ok(Some(doc))
            },
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest =>
                Err(Error::InvalidRequest { response: try!(ErrorResponse::from_reader(resp)) }),
            hyper::status::StatusCode::Unauthorized =>
                Err(Error::Unauthorized { response: try!(ErrorResponse::from_reader(resp)) }),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: Some(try!(ErrorResponse::from_reader(resp))) }),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
