//! Module for individual command types.
//!
//! Applications should not access the `command` module directly. Instead, they
//! should use the appropriate `Client` method to construct a command. For
//! example, the method `post_to_database` constructs a `PostToDatabase`
//! command.

macro_rules! impl_command_public_methods {
    ($command_output:ty) => {
        /// Sends the command request and waits for the response.
        pub fn run(self) -> Result<$command_output, Error> {
            command::run_command(self)
        }
    }
}

macro_rules! make_couchdb_error {
    ($error_variant:ident, $response:expr) => {
        Error::$error_variant(Some(try!(json::decode_json::<_, ErrorResponse>($response))))
    }
}

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

use Error;
use Revision;
use error::TransportKind;

// The Command trait abstracts the machinery for executing CouchDB commands.
// Types implementing the Command trait define only how they construct requests
// and process responses. This separates the command logic from the
// responsibility of sending a request and receiving its response.
trait Command: Sized {
    type Output;
    type State;
    fn make_request(self) -> Result<(Request, Self::State), Error>;
    fn take_response(resp: hyper::client::Response,
                     state: Self::State)
                     -> Result<Self::Output, Error>;
}

fn run_command<C>(cmd: C) -> Result<C::Output, Error>
    where C: Command
{
    let (resp, state) = {
        use std::io::Write;
        let (req, state) = try!(cmd.make_request());
        let mut stream = try!(req.request
                                 .start()
                                 .map_err(|e| Error::Transport(TransportKind::Hyper(e))));
        try!(stream.write_all(&req.body)
                   .map_err(|e| Error::Transport(TransportKind::Io(e))));
        let resp = try!(stream.send()
                              .map_err(|e| Error::Transport(TransportKind::Hyper(e))));
        (resp, state)
    };
    C::take_response(resp, state)
}

struct Request {
    request: hyper::client::Request<hyper::net::Fresh>,
    body: Vec<u8>,
}

impl Request {
    pub fn new(method: hyper::method::Method, uri: hyper::Url) -> Result<Self, Error> {
        let r = try!(hyper::client::Request::new(method, uri)
                         .map_err(|e| Error::Transport(TransportKind::Hyper(e))));

        Ok(Request {
            request: r,
            body: Vec::new(),
        })
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn accept_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            vec![hyper::header::qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]
        };
        self.request.headers_mut().set(hyper::header::Accept(qitems));
        self
    }

    pub fn content_type_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            Mime(TopLevel::Application, SubLevel::Json, vec![])
        };
        self.request.headers_mut().set(hyper::header::ContentType(qitems));
        self
    }

    pub fn if_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.request.headers_mut().set(hyper::header::IfMatch::Items(etags));
                self
            }
        }
    }

    pub fn if_none_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.request
                    .headers_mut()
                    .set(hyper::header::IfNoneMatch::Items(etags));
                self
            }
        }
    }
}

fn new_revision_etags(rev: &Revision) -> Vec<hyper::header::EntityTag> {
    vec![hyper::header::EntityTag::new(false, rev.to_string())]
}

// Returns an error if the HTTP response doesn't have a Content-Type of
// `application/json`.
fn content_type_must_be_application_json(headers: &hyper::header::Headers) -> Result<(), Error> {
    match headers.get::<hyper::header::ContentType>() {
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
