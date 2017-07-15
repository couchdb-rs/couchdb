// The transport layer exists to isolate I/O logic away from the actions,
// thereby allowing the actions to generate abstract HTTP headers and handle
// abstract HTTP responses.
//
// This is not as easy as it may sound. The goals for the transport layer are:
//
// 1. To support asynchronous and synchronous I/O while allowing the actions to
//    completely reuse code for both modes,
// 2. To shield the actions from the details of our HTTP dependency (i.e.,
//    reqwest), and,
// 3. To allow test code to inspect the HTTP request an action generates and to
//    mock the response it will handle.
//
// And, of course, to do all this while adding only a minimal amount of
// overhead.

#[cfg(test)]
mod mock;
mod net;

#[cfg(test)]
pub use self::mock::MockTransport;
pub use self::net::NetTransport;
use Error;
use error::{ErrorCategory, Nok};
use futures::{Async, Future, Poll};
pub use reqwest::{Method, StatusCode, header};
use serde::Deserialize;

pub trait Transport: Clone {
    type Request: Request;
    type RequestFuture: Future<Item = Self::Request, Error = Error> + 'static;
    fn request<P: AsRef<str>>(&self, method: Method, url_path: Result<P, Error>) -> Self::RequestFuture;
}

pub trait Request {
    type Response: Response;
    type Future: Future<Item = Self::Response, Error = Error> + 'static;
    fn accept_application_json(&mut self);
    fn send_without_body(self) -> Self::Future;
}

pub trait Response {
    fn status_code(&self) -> StatusCode;

    // TODO: The return type should be unboxed, as it should need only to
    // implement `Future<Item = T, Error = Error>`. However, Rust doesn't
    // support generic associated types, so we fall back to using the most
    // general concrete type, which is a boxed future.
    fn json_body<T>(&mut self) -> Box<Future<Item = T, Error = Error>>
    where
        for<'de> T: Deserialize<'de> + 'static;
}

/// `ActionFuture` holds the future result of an [action](action/index.html).
///
/// # Summary
///
/// * `ActionFuture` implements `Future<Item = T, Error = Error>`.
///
/// * `ActionFuture` is a workaround for Rust not yet having a stable “impl
///   Trait” feature. As such, this type may go away in a future release, when
///   Rust lands that feature.
///
pub struct ActionFuture<T>(Box<Future<Item = T, Error = Error>>);

impl<T> ActionFuture<T> {
    #[doc(hidden)]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Item = T, Error = Error> + 'static,
    {
        ActionFuture(Box::new(future))
    }
}

impl<T> Future for ActionFuture<T> {
    type Item = T;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

pub enum ServerResponseFuture<T> {
    Ok(Option<T>),
    // TODO: Unbox this type.
    AwaitingErrorBody(StatusCode, Option<ErrorCategory>, Box<Future<Item = Nok, Error = Error>>),
}

impl<T> ServerResponseFuture<T> {
    pub fn ok(item: T) -> Self {
        ServerResponseFuture::Ok(Some(item))
    }

    pub fn err<R: Response>(mut response: R, category: Option<ErrorCategory>) -> Self {
        // TODO: If the JSON decoding fails then we throw away the error result,
        // so it would be good to have an alternative method for decoding a JSON
        // body whereby no error is returned on error.
        ServerResponseFuture::AwaitingErrorBody(response.status_code(), category, response.json_body())
    }
}

impl<T> Future for ServerResponseFuture<T> {
    type Item = T;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // FIXME: Test this. This code is unlikely to execute in any of our
        // other tests because some of the edge cases involve the CouchDB server
        // sending, say, a non-JSON body.
        match self {
            &mut ServerResponseFuture::Ok(ref mut item) => Ok(Async::Ready(
                item.take().expect("Future has already completed"),
            )),
            &mut ServerResponseFuture::AwaitingErrorBody(status_code, maybe_category, ref mut nok_future) => {
                match nok_future.poll() {
                    Err(_) => Err(Error::from_server_response(
                        status_code,
                        None,
                        maybe_category,
                    )),
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Ok(Async::Ready(nok)) => Err(Error::from_server_response(
                        status_code,
                        Some(nok),
                        maybe_category,
                    )),
                }
            }
        }
    }
}
