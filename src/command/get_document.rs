use hyper;
use serde;
use serde_json;
use std;

use client;
use document::{self, Document, Revision};
use design::DesignDocument;
use error::{self, Error};

/// Command to get a document.
pub struct GetDocument<'a, T: serde::Deserialize> {
    client_state: &'a client::ClientState,
    uri: hyper::Url,
    if_none_match: Option<&'a Revision>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: serde::Deserialize> GetDocument<'a, T> {

    pub fn new_db_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        doc_id: &str)
        -> GetDocument<'a, T>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push(doc_id.to_string());
        GetDocument {
            client_state: client_state,
            uri: u,
            if_none_match: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> GetDocument<'a, T> {
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

        let mut resp = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            let mut req = self.client_state.http_client.get(self.uri)
                .header(hyper::header::Accept(vec![
                                              hyper::header::qitem(
                                                  Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
                ;
            if self.if_none_match.is_some() {
                req = req.header(
                    hyper::header::IfNoneMatch::Items(
                        vec![
                            hyper::header::EntityTag::new(
                                false,
                                self.if_none_match.unwrap().as_str().to_string())
                        ]
                    )
                );
            }
            try!(
                req.send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };

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
                        let rev = document::new_revision_from_string(rev);
                        let id = match dot.remove("_id") {
                            None => { return None; },
                            Some(x) => match x {
                                serde_json::Value::String(x) => x,
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

impl<'a> GetDocument<'a, DesignDocument> {
    pub fn new_design_document(
        client_state: &'a client::ClientState,
        db_name: &str,
        ddoc_id: &str)
        -> GetDocument<'a, DesignDocument>
    {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push("_design".to_string());
        u.path_mut().unwrap().push(ddoc_id.to_string());
        GetDocument {
            client_state: client_state,
            uri: u,
            if_none_match: None,
            _phantom: std::marker::PhantomData,
        }
    }
}
