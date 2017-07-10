use {Error, futures, reqwest, tokio_core, transport};
use futures::Future;
use serde::Deserialize;
use transport::{Action, ActionFuture, Method, StatusCode, Transport};
use url::Url;

#[derive(Debug)]
pub struct AsyncTransport {
    http_client: reqwest::unstable::async::Client,
    server_url: Url,
}

impl AsyncTransport {
    pub fn new(reactor_handle: &tokio_core::reactor::Handle, server_url: Url) -> Result<Self, Error> {
        Ok(AsyncTransport {
            http_client: reqwest::unstable::async::Client::new(reactor_handle)
                .map_err(|e| ("Failed to construct HTTP client", e))?,
            server_url: server_url,
        })
    }

    pub fn transport_async<A: Action>(&self, actionable: &A) -> ActionFuture<A::Item> {
        let request_maker = RequestMaker::new(&self.http_client, &self.server_url);
        actionable.act(request_maker)
    }
}

impl Transport for AsyncTransport {}

#[derive(Debug)]
struct RequestMaker<'a> {
    http_client: &'a reqwest::unstable::async::Client,
    server_url: Url,
}

impl<'a> RequestMaker<'a> {
    fn new(http_client: &'a reqwest::unstable::async::Client, server_url: &Url) -> Self {
        RequestMaker {
            http_client: http_client,
            server_url: server_url.clone(),
        }
    }
}

impl<'a> transport::RequestMaker for RequestMaker<'a> {
    type Request = Request;

    // TODO: Unbox this type.
    type Future = Box<Future<Item = Self::Request, Error = Error>>;

    fn make_request(self, method: Method, url_path: &str) -> Self::Future {
        let mut url = self.server_url;
        url.set_path(url_path);

        Box::new(
            futures::future::result(self.http_client.request(method, url))
                .map_err(|e| Error::from(("Could not construct HTTP request", e)))
                .map(|x| Request::new(x)),
        )
    }
}

#[derive(Debug)]
struct Request {
    inner: reqwest::unstable::async::RequestBuilder,
}

impl Request {
    fn new(inner: reqwest::unstable::async::RequestBuilder) -> Self {
        Request { inner: inner }
    }

    // TODO: Unbox the return type when reqwest exports its underlying future
    // type.
    fn send(mut self) -> Box<Future<Item = Response, Error = Error>> {
        Box::new(
            self.inner
                .send()
                .map_err(|e| Error::from(("HTTP request failed", e)))
                .map(|x| Response::new(x)),
        )
    }
}

impl transport::Request for Request {
    type Response = Response;

    // TODO: Unbox this type.
    type Future = Box<Future<Item = Self::Response, Error = Error>>;

    fn set_accept_application_json(&mut self) {
        self.inner.header(reqwest::header::Accept::json());
    }

    fn send_without_body(self) -> Self::Future {
        self.send()
    }
}

#[derive(Debug)]
struct Response {
    inner: reqwest::unstable::async::Response,
}

impl Response {
    fn new(inner: reqwest::unstable::async::Response) -> Self {
        Response { inner: inner }
    }
}

impl transport::Response for Response {
    fn status_code(&self) -> StatusCode {
        self.inner.status()
    }

    fn json_body<T>(&mut self) -> Box<Future<Item = T, Error = Error>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        Box::new(self.inner.json().map_err(|e| {
            Error::from(("Could not decode HTTP response body as JSON", e))
        }))
    }
}
