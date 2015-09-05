use client;
use hyper;

use document::Revision;
use error::{self, Error};

/// Command to create a database.
pub struct HeadDocument<'a> {
    client_state: &'a client::ClientState,
    uri: hyper::Url,
    if_none_match: Revision,
}

impl<'a> HeadDocument<'a> {

    pub fn new_db_document(client_state: &'a client::ClientState,
                           db_name: &str,
                           doc_id: &str) -> HeadDocument<'a> {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push(doc_id.to_string());
        HeadDocument {
            client_state: client_state,
            uri: u,
            if_none_match: Revision::new(),
        }
    }

    pub fn new_design_document(client_state: &'a client::ClientState,
                               db_name: &str,
                               ddoc_id: &str) -> HeadDocument<'a> {
        let mut u = client_state.uri.clone();
        u.path_mut().unwrap()[0] = db_name.to_string();
        u.path_mut().unwrap().push("_design".to_string());
        u.path_mut().unwrap().push(ddoc_id.to_string());
        HeadDocument {
            client_state: client_state,
            uri: u,
            if_none_match: Revision::new(),
        }
    }

    /// Set the If-None-Match header.
    pub fn if_none_match(mut self, rev: Revision) -> HeadDocument<'a> {
        self.if_none_match = rev;
        self
    }

    /// Send the command request and wait for the response.
    // TODO: Document error variants.
    pub fn run(self) -> Result<Option<()>, Error> {

        let mut resp = {
            let mut req = self.client_state.http_client.head(self.uri);
            if !self.if_none_match.is_empty() {
                req = req.header(hyper::header::IfNoneMatch::Items(
                        vec![hyper::header::EntityTag::new(false,
                                                           self.if_none_match.as_str().to_string())]));
            }
            try!(
                req.send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };

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
