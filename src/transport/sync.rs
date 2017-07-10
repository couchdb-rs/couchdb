use {Error, futures, reqwest, transport};
use futures::Future;
use serde::Deserialize;
use transport::{Action, Method, StatusCode, Transport};
use url::Url;

#[derive(Debug)]
pub struct SyncTransport {
    http_client: reqwest::Client,
    server_url: Url,
}

impl SyncTransport {
    pub fn new(server_url: Url) -> Result<Self, Error> {
        Ok(SyncTransport {
            http_client: reqwest::Client::new().map_err(|e| {
                ("Failed to construct HTTP client", e)
            })?,
            server_url: server_url,
        })
    }

    pub fn transport_sync<A: Action>(&self, actionable: &A) -> Result<A::Item, Error> {

        // We can use `wait` on the future because we're using the synchronous
        // reqwest Client, which isn't driven by an asynchronous reactor.

        let request_maker = RequestMaker::new(&self.http_client, &self.server_url);
        actionable.act(request_maker).wait()
    }
}

impl Transport for SyncTransport {}

#[derive(Debug)]
struct RequestMaker<'a> {
    http_client: &'a reqwest::Client,
    server_url: Url,
}

impl<'a> RequestMaker<'a> {
    fn new(http_client: &'a reqwest::Client, server_url: &Url) -> Self {
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
    inner: reqwest::RequestBuilder,
}

impl Request {
    fn new(inner: reqwest::RequestBuilder) -> Self {
        Request { inner: inner }
    }

    fn send(mut self) -> futures::future::FutureResult<Response, Error> {
        futures::future::result(
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
        Box::new(self.send())
    }
}

#[derive(Debug)]
struct Response {
    inner: reqwest::Response,
}

impl Response {
    fn new(inner: reqwest::Response) -> Self {
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
        Box::new(futures::future::result(self.inner.json().map_err(|e| {
            Error::from(("Could not decode HTTP response body as JSON", e))
        })))
    }
}
