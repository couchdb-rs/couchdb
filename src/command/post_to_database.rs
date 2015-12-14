use hyper;
use serde;
use serde_json;

use client::ClientState;
use dbpath::DatabasePath;
use dbtype::PostToDatabaseResponse;
use docid::DocumentId;
use docpath::DocumentPath;
use error::{Error, ErrorResponse};
use revision::Revision;
use transport::{self, Command, Request};

/// Command to create a document.
pub struct PostToDatabase<'a, T>
    where T: 'a + serde::Serialize
{
    client_state: &'a ClientState,
    path: DatabasePath,
    doc_content: &'a T,
}

impl<'a, T> PostToDatabase<'a, T> where T: 'a + serde::Serialize
{
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: DatabasePath, doc_content: &'a T) -> Self
        where T: serde::Serialize
    {
        PostToDatabase {
            client_state: client_state,
            path: path,
            doc_content: doc_content,
        }
    }

    /// Send the command request and wait for the response.
    ///
    /// # Return
    ///
    /// Return the new revision and path for the document.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The document does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(Revision, DocumentPath), Error> {
        transport::run_command(self)
    }
}

impl<'a, T> Command for PostToDatabase<'a, T> where T: 'a + serde::Serialize
{
    type Output = (Revision, DocumentPath);
    type State = DatabasePath;

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = self.path.clone();
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let body = try!(serde_json::to_vec(self.doc_content).map_err(|e| Error::Encode { cause: e }));
        let req = try!(Request::new(hyper::method::Method::Post, uri))
                      .accept_application_json()
                      .content_type_application_json()
                      .body(body);
        Ok((req, db_path))
    }

    fn take_response(resp: hyper::client::Response, db_path: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Created => {
                try!(transport::content_type_must_be_application_json(&resp.headers));
                let content = try!(transport::decode_json::<_, PostToDatabaseResponse>(resp));
                let id: DocumentId = content.id.into();
                let path = DocumentPath::new(db_path, id);
                let rev: Revision = content.rev.into();
                Ok((rev, path))
            }
            hyper::status::StatusCode::BadRequest => {
                Err(Error::InvalidRequest { response: try!(ErrorResponse::from_reader(resp)) })
            }
            hyper::status::StatusCode::Unauthorized => {
                Err(Error::Unauthorized { response: try!(ErrorResponse::from_reader(resp)) })
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound { response: Some(try!(ErrorResponse::from_reader(resp))) })
            }
            hyper::status::StatusCode::Conflict => {
                // Need to include this error variant in the command's
                // documentation if we ever add support for an explicit id.
                Err(Error::DocumentConflict { response: try!(ErrorResponse::from_reader(resp)) })
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
