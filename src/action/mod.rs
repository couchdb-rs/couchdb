//! Actions and their related types.
//!
//! Applications should construct actions (e.g., `PutDatabase`, `GetDocument`,
//! etc.) by calling the appropriate `Client` method. For example, the method
//! `Client::post_to_database` constructs a `PostToDatabase` action.

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
        Error::$error_variant(Some(try!($response.decode_json_all::<ErrorResponse>())))
    }
}

#[macro_use]
mod test_macro;

mod delete_database;
mod delete_document;
mod get_all_databases;
mod get_changes;
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
pub use self::get_changes::{GetChanges, GetChangesEvent};
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
use std;
use std::io::prelude::*;

#[cfg(test)]
use serde_json;

use Error;
use Revision;
use error::TransportKind;

// The Action trait abstracts the machinery for executing CouchDB actions. Types
// implementing the Action trait define only how they construct requests and
// process responses. This separates the action logic from the responsibility of
// sending the request and receiving its response.
trait Action: Sized {
    type Output;
		type State;
    fn make_request(self) -> Result<(Request, Self::State), Error>;
    fn take_response<R>(response: R, state: Self::State) -> Result<Self::Output, Error>
        where R: Response;
}

fn run_action<A>(action: A) -> Result<A::Output, Error>
    where A: Action
{
    let (action_request, action_state) = try!(action.make_request());

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

    A::take_response(action_response, action_state)
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

    // Returns the HTTP status code.
    fn status(&self) -> hyper::status::StatusCode {
        unimplemented!();
    }

    // Returns an error if and only if the response does not have a Content-Type
    // header equivalent to 'application/json'.
    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        unimplemented!();
    }

    // Decodes the entire response body as JSON.
    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        unimplemented!();
    }

    // Decodes the next line of the response body as JSON. Returns None if and
    // only if EOF is reached without reading a line.
    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        unimplemented!();
    }

    // Returns an error if and only if the response body has non-whitespace
    // remaining.
    fn no_more_content(&mut self) -> Result<(), Error> {
        unimplemented!();
    }
}

mod json {
    use serde;
    use serde_json;
    use std;

    use Error;
    use error::{DecodeErrorKind, TransportKind};

    pub fn decode_json_all<R, T>(reader: &mut R) -> Result<T, Error>
        where R: std::io::Read,
              T: serde::Deserialize
    {
        let reader = reader.by_ref();
        serde_json::from_reader(reader).map_err(|e| {
            match e {
                serde_json::Error::IoError(e) => Error::Transport(TransportKind::Io(e)),
                _ => Error::Decode(DecodeErrorKind::Serde { cause: e }),
            }
        })
    }

    pub fn decode_json_line<R, T>(reader: &mut R) -> Result<T, Error>
        where R: std::io::BufRead,
              T: serde::Deserialize
    {
        let mut s = String::new();
        try!(reader.read_line(&mut s)
                   .map_err(|e| Error::Transport(TransportKind::Io(e))));
        serde_json::from_str::<T>(&s)
            .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
    }

    pub fn no_more_content<R>(reader: &mut R) -> Result<(), Error>
        where R: std::io::Read
    {
        let mut s = String::new();
        try!(reader.read_to_string(&mut s)
                   .map_err(|e| Error::Transport(TransportKind::Io(e))));
        for c in s.chars() {
            match c {
                '\r' | '\n' | ' ' => (),
                _ => {
                    return Err(Error::Decode(DecodeErrorKind::TrailingContent));
                }
            }
        }
        Ok(())
    }
}

struct HyperResponse {
    hyper_response: std::io::BufReader<hyper::client::Response>,
}

impl HyperResponse {
    fn new(hyper_response: hyper::client::Response) -> Self {
        HyperResponse { hyper_response: std::io::BufReader::new(hyper_response) }
    }
}

impl Response for HyperResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.hyper_response.get_ref().status
    }

    // Returns an error if the HTTP response doesn't have a Content-Type of
    // `application/json`.
    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        headers_content_type_must_be_application_json(&self.hyper_response.get_ref().headers)
    }

    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_all(&mut self.hyper_response)
    }

    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_line(&mut self.hyper_response)
    }

    fn no_more_content(&mut self) -> Result<(), Error> {
        json::no_more_content(&mut self.hyper_response)
    }
}

// Mock response encapsulating a typical application/json response.
#[cfg(test)]
struct JsonResponse {
    status_code: hyper::status::StatusCode,
    body: std::io::BufReader<std::io::Cursor<String>>,
}

#[cfg(test)]
impl JsonResponse {
    fn new<T: serde::Serialize>(status_code: hyper::status::StatusCode, body: &T) -> Self {
        let body = serde_json::to_string(&body).unwrap();
        let body = std::io::BufReader::new(std::io::Cursor::new(body));
        JsonResponse {
            status_code: status_code,
            body: body,
        }
    }

    fn new_from_string<S>(status_code: hyper::status::StatusCode, body: S) -> Self
        where S: Into<String>
    {
        JsonResponse {
            status_code: status_code,
            body: std::io::BufReader::new(std::io::Cursor::new(body.into())),
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

    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_all(&mut self.body)
    }

    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_line(&mut self.body)
    }

    fn no_more_content(&mut self) -> Result<(), Error> {
        json::no_more_content(&mut self.body)
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

fn headers_content_type_must_be_application_json(headers: &hyper::header::Headers)
                                                 -> Result<(), Error> {
    use hyper::header::ContentType;
    use hyper::mime::{Mime, TopLevel, SubLevel};
    let c = "application/json";
    match headers.get::<ContentType>() {
        None => Err(Error::NoContentTypeHeader { expected: c }),
        Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, ref _param))) => Ok(()),
        Some(&ContentType(ref mime)) => {
            Err(Error::UnexpectedContentTypeHeader {
                expected: c,
                got: format!("{}", mime),
            })
        }
    }
}

fn new_revision_etags(rev: &Revision) -> Vec<hyper::header::EntityTag> {
    vec![hyper::header::EntityTag::new(false, rev.to_string())]
}

#[cfg(test)]
mod tests {

    #[test]
    fn headers_content_type_must_be_application_json_ok_with_charset() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("application/json; charset=utf-8".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap();
    }

    #[test]
    fn headers_content_type_must_be_application_json_ok_without_charset() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("application/json".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap();
    }

    #[test]
    fn headers_content_type_must_be_application_json_no_header() {
        use hyper::header::Headers;
        let headers = Headers::new();
        super::headers_content_type_must_be_application_json(&headers).unwrap_err();
    }

    #[test]
    fn headers_content_type_must_be_application_json_wrong_type() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("plain/text".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap_err();
    }
}
