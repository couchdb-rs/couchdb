//! Module for individual action types.
//!
//! Applications should not access the `action` module directly. Instead, they
//! should use the appropriate `Client` method to construct an action. For
//! example, the method `post_to_database` constructs a `PostToDatabase`
//! action.

macro_rules! impl_action_public_methods {
    ($action_output:ty) => {
        /// Sends the action request and waits for the response.
        pub fn run(self) -> Result<$action_output, Error> {
            action::run_action(self)
        }
    }
}

macro_rules! make_couchdb_error {
    ($error_variant:ident, $response:expr) => {
        Error::$error_variant(Some(try!($response.decode_json::<ErrorResponse>())))
    }
}

#[macro_use]
mod test_macro;

mod delete_database;
mod delete_document;
mod get_all_databases;
mod get_database;
mod get_document;
mod get_view;
mod head_database;
mod head_document;
mod post_to_database;
mod put_database;
mod put_document;

pub use self::delete_database::DeleteDatabase;
pub use self::delete_document::DeleteDocument;
pub use self::get_all_databases::GetAllDatabases;
pub use self::get_database::GetDatabase;
pub use self::get_document::GetDocument;
pub use self::get_view::GetView;
pub use self::head_database::HeadDatabase;
pub use self::head_document::HeadDocument;
pub use self::post_to_database::PostToDatabase;
pub use self::put_database::PutDatabase;
pub use self::put_document::PutDocument;

use hyper;
use serde;
use serde_json;

use Error;
use Revision;
use error::{DecodeErrorKind, TransportKind};

// The Action trait abstracts the machinery for executing CouchDB actions. Types
// implementing the Action trait define only how they construct requests and
// process responses. This separates the action logic from the responsibility of
// sending the request and receiving its response.
trait Action: Sized {
    type Output;
    fn make_request(self) -> Result<Request, Error>;
    fn take_response<R: Response>(response: R) -> Result<Self::Output, Error>;
}

fn run_action<A>(action: A) -> Result<A::Output, Error>
    where A: Action
{
    let action_request = try!(action.make_request());

    let action_response = {
        use std::io::Write;
        let mut hyper_request = try!(hyper::client::Request::new(action_request.method,
                                                                 action_request.uri)
                                         .map_err(|e| Error::Transport(TransportKind::Hyper(e))));
        *hyper_request.headers_mut() = action_request.headers;
        let mut request_stream = try!(hyper_request.start()
                                                   .map_err(|e| {
                                                       Error::Transport(TransportKind::Hyper(e))
                                                   }));
        try!(request_stream.write_all(&action_request.body)
                           .map_err(|e| Error::Transport(TransportKind::Io(e))));
        let hyper_response = try!(request_stream.send().map_err(|e| {
            Error::Transport(TransportKind::Hyper(e))
        }));
        HyperResponse::new(hyper_response)
    };

    A::take_response(action_response)
}

struct Request {
    method: hyper::method::Method,
    uri: hyper::Url,
    headers: hyper::header::Headers,
    body: Vec<u8>,
}

impl Request {
    pub fn new(method: hyper::method::Method, uri: hyper::Url) -> Self {
        Request {
            method: method,
            uri: uri,
            headers: hyper::header::Headers::new(),
            body: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn method(&self) -> &hyper::method::Method {
        &self.method
    }

    #[cfg(test)]
    pub fn uri(&self) -> &hyper::Url {
        &self.uri
    }

    #[cfg(test)]
    pub fn headers(&self) -> &hyper::header::Headers {
        &self.headers
    }

    pub fn set_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn set_accept_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            vec![hyper::header::qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]
        };
        self.headers.set(hyper::header::Accept(qitems));
        self
    }

    pub fn set_content_type_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            Mime(TopLevel::Application, SubLevel::Json, vec![])
        };
        self.headers.set(hyper::header::ContentType(qitems));
        self
    }

    pub fn set_if_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.headers.set(hyper::header::IfMatch::Items(etags));
                self
            }
        }
    }

    pub fn set_if_none_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.headers.set(hyper::header::IfNoneMatch::Items(etags));
                self
            }
        }
    }
}

trait Response {

    fn status(&self) -> hyper::status::StatusCode {
        hyper::status::StatusCode::InternalServerError
    }

    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        Err(Error::NoContentTypeHeader { expected: "application/json" })
    }

    fn decode_json<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        serde_json::from_str("").map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
    }
}

struct HyperResponse {
    hyper_response: hyper::client::Response,
}

impl HyperResponse {
    fn new(hyper_response: hyper::client::Response) -> Self {
        HyperResponse { hyper_response: hyper_response }
    }
}

impl Response for HyperResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.hyper_response.status
    }

    // Returns an error if the HTTP response doesn't have a Content-Type of
    // `application/json`.
    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        // FIXME: Test this.
        match self.hyper_response.headers.get::<hyper::header::ContentType>() {
            None => Err(Error::NoContentTypeHeader { expected: "application/json" }),
            Some(content_type) => {
                use hyper::mime::*;
                let exp = hyper::mime::Mime(TopLevel::Application, SubLevel::Json, vec![]);
                let &hyper::header::ContentType(ref got) = content_type;
                if *got != exp {
                    Err(Error::UnexpectedContentTypeHeader {
                        expected: "application/json",
                        got: format!("{}", got),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    fn decode_json<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        serde_json::from_reader::<_, T>(&mut self.hyper_response).map_err(|e| {
            match e {
                serde_json::Error::IoError(e) => Error::Transport(TransportKind::Io(e)),
                _ => Error::Decode(DecodeErrorKind::Serde { cause: e }),
            }
        })
    }
}

// Mock response encapsulating a typical application/json response.
#[cfg(test)]
struct JsonResponse {
    status_code: hyper::status::StatusCode,
    body: String,
}

#[cfg(test)]
impl JsonResponse {
    fn new<T: serde::Serialize>(status_code: hyper::status::StatusCode, body: &T) -> Self {
        JsonResponse {
            status_code: status_code,
            body: serde_json::to_string(&body).unwrap(),
        }
    }
}

#[cfg(test)]
impl Response for JsonResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.status_code
    }

    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        Ok(())
    }

    fn decode_json<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        serde_json::from_str(&self.body)
            .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
    }
}

// Mock response encapsulating a response with no body.
#[cfg(test)]
struct NoContentResponse {
    status_code: hyper::status::StatusCode,
}

#[cfg(test)]
impl NoContentResponse {
    fn new(status_code: hyper::status::StatusCode) -> Self {
        NoContentResponse { status_code: status_code }
    }
}

#[cfg(test)]
impl Response for NoContentResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.status_code
    }
}

fn new_revision_etags(rev: &Revision) -> Vec<hyper::header::EntityTag> {
    // FIXME: Test this.
    vec![hyper::header::EntityTag::new(false, rev.to_string())]
}
