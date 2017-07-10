// The transport layer exists to isolate I/O logic away from the actions,
// thereby allowing the actions to focus purely on HTTP in an abstract
// sense--i.e., HTTP method, URL, HTTP headers, request body, and
// response-handling.
//
// This is not as easy as it may sound. Transport goals are:
//
// 1. Support asynchronous and synchronous I/O while letting the actions to
//    reuse all code for both modes.
// 2. Shield the actions from the details of our HTTP dependency (i.e.,
//    reqwest).
// 3. Allow test code to inspect the HTTP request an action generates and to
//    mock the response it will handle. I.e., allow full HTTP-round-trip testing
//    without doing any actual networking.
//
// And, of course, to do all this while adding only a minimal amount of
// overhead.

mod async;
#[cfg(test)]
mod mock;
mod sync;

pub use self::async::AsyncTransport;
#[cfg(test)]
pub use self::mock::MockTransport;
pub use self::sync::SyncTransport;
use Error;
use error::{ErrorCategory, Nok};
use futures::{Async, Future, Poll};
pub use reqwest::{Method, StatusCode, header};
use serde::Deserialize;

pub trait Transport {}

pub trait RequestMaker {
    type Request: Request;
    type Future: Future<Item = Self::Request, Error = Error> + 'static;
    fn make_request(self, method: Method, url_path: &str) -> Self::Future;
}

pub trait Request {
    type Response: Response;
    type Future: Future<Item = Self::Response, Error = Error> + 'static;
    fn set_accept_application_json(&mut self);
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

pub trait Action {
    type Item;
    fn act<R: RequestMaker>(&self, request_maker: R) -> ActionFuture<Self::Item>;
}

pub struct ActionFuture<T>(Box<Future<Item = T, Error = Error>>);

impl<T> ActionFuture<T> {
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
