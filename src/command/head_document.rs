use hyper;
use std;

use client::ClientState;
use document::{self, DocumentType, Revision};
use error::{self, Error};
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_head_document<'a, D>(
    client_state: &'a ClientState,
    db_name: &'a str,
    doc_id: &'a str)
    -> HeadDocument<'a, D>
    where D: DocumentType
{
    HeadDocument {
        client_state: client_state,
        doc_type: std::marker::PhantomData,
        db_name: db_name,
        doc_id: doc_id,
        if_none_match: None,
    }
}

/// Command to get document meta-information.
pub struct HeadDocument<'a, D>
    where D: DocumentType
{
    client_state: &'a ClientState,
    doc_type: std::marker::PhantomData<D>,
    db_name: &'a str,
    doc_id: &'a str,
    if_none_match: Option<&'a Revision>,
}

impl<'a, D> HeadDocument<'a, D> where D: DocumentType {

    /// Set the If-None-Match header.
    pub fn if_none_match(
        mut self,
        rev: &'a Revision)
        -> HeadDocument<'a, D>
        where D: DocumentType
    {
        self.if_none_match = Some(rev);
        self
    }

    /// Send the command request and wait for the response.
    ///
    /// # Return
    ///
    /// Return `None` if an If-None-Match revision is given and the document
    /// hasn't been modified since that revision. Otherwise, return `Some`.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The document does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<Option<()>, Error> {
        transport::run_command(self)
    }
}

impl<'a, D> Command for HeadDocument<'a, D> where D: DocumentType {

    type Output = Option<()>;

    fn make_request(self) -> Result<Request, Error> {
        let uri = document::new_uri::<D>(
            &self.client_state.uri,
            self.db_name,
            self.doc_id);
        let req = try!(Request::new(hyper::Head, uri))
            .if_none_match_revision(self.if_none_match);
        Ok(req)
    }

    fn take_response(mut resp: hyper::client::Response)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(Some(())),
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: None } ),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
