use hyper;
use serde;
use serde_json;
use std;

use client::{self, ClientState};
use docid::DocumentId;
use docpath::DocumentPath;
use document::Document;
use error::{self, Error};
use revision::Revision;
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_get_document<'a, T>(client_state: &'a ClientState, path: DocumentPath)
    -> GetDocument<'a, T>
    where T: serde::Deserialize
{
    GetDocument {
        client_state: client_state,
        path: path,
        if_none_match: None,
        _content_type: std::marker::PhantomData,
    }
}

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
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json()
            .if_none_match_revision(self.if_none_match);
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let mut resp_body = try!(client::decode_json::<serde_json::Value>(&s));
                match (|| {
                    let (rev, id) = {
                        let mut dot = match resp_body.as_object_mut() {
                            None => { return None; },
                            Some(x) => x,
                        };
                        let rev = match dot.remove("_rev") {
                            None => { return None; },
                            Some(x) => match x {
                                serde_json::Value::String(x) => x,
                                _ => { return None; },
                            },
                        };
                        let rev = Revision::from(rev);
                        let id = match dot.remove("_id") {
                            None => { return None; },
                            Some(x) => match x {
                                serde_json::Value::String(x) =>
                                    DocumentId::from(x),
                                _ => { return None; },
                            },
                        };
                        //body_map.remove("_deleted");
                        //body_map.remove("_attachments");
                        //body_map.remove("_conflicts");
                        //body_map.remove("_deleted_conflicts");
                        //body_map.remove("_local_seq");
                        //body_map.remove("_revs_info");
                        //body_map.remove("_revisions");
                        (rev, id)
                    };
                    let content = match serde_json::from_value::<T>(resp_body) {
                        Ok(x) => x,
                        Err(_) => { return None; },
                    };
                    Some(
                        Document::<T> {
                            content: content,
                            revision: rev,
                            id: id,
                        }
                    )
                })() {
                    None => Err(Error::UnexpectedContent { got: s } ),
                    Some(x) => Ok(Some(x)),
                }
            },
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_request(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
