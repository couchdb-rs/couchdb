use {Error, futures, reqwest, tokio_core};
use error::{ErrorCategory, Nok};
use futures::{Async, Future, Poll};
pub use reqwest::{Method, StatusCode, header};
use url::Url;

#[derive(Clone, Debug)]
pub struct Transport {
    http_client: reqwest::unstable::async::Client,
    server_url: Url,
}

impl Transport {
    pub fn new(reactor_handle: &tokio_core::reactor::Handle, server_url: Url) -> Result<Self, Error> {
        Ok(Transport {
            http_client: reqwest::unstable::async::Client::new(reactor_handle)
                .map_err(|e| ("Failed to construct HTTP client", e))?,
            server_url: server_url,
        })
    }

    pub fn server_url(&self) -> &Url {
        &self.server_url
    }

    pub fn request<F>(
        &self,
        method: Method,
        url: Url,
        request_prep: F,
    ) -> Box<Future<Item = reqwest::unstable::async::Response, Error = Error>>
    where
        F: FnOnce(&mut reqwest::unstable::async::RequestBuilder) + 'static,
    {
        Box::new(
            futures::future::result(self.http_client.request(method, url))
                .map_err(|e| Error::from(("Could not construct HTTP request", e)))
                .and_then(|mut request_builder| {
                    request_prep(&mut request_builder);
                    request_builder.send().map_err(|e| {
                        Error::from(("HTTP request failed", e))
                    })
                }),
        )
    }
}

pub enum ServerResponseFuture<T> {
    Ok(Option<T>),
    AwaitingBody(StatusCode, Option<ErrorCategory>, Box<Future<Item = Nok, Error = reqwest::Error>>),
}

impl<T> ServerResponseFuture<T> {
    pub fn ok(item: T) -> Self {
        ServerResponseFuture::Ok(Some(item))
    }

    pub fn err(mut response: reqwest::unstable::async::Response, category: Option<ErrorCategory>) -> Self {
        ServerResponseFuture::AwaitingBody(
            response.status(),
            category,
            Box::new(response.json::<Nok>()),
        )
    }
}

impl<T> Future for ServerResponseFuture<T> {
    type Item = T;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self {
            &mut ServerResponseFuture::Ok(ref mut item) => Ok(Async::Ready(
                item.take().expect("Future has already completed"),
            )),
            &mut ServerResponseFuture::AwaitingBody(status_code, maybe_category, ref mut nok_future) => {
                match nok_future.poll() {
                    Err(_) => Err(Error::from_server_response(status_code, None, None)),
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
