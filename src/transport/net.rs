use {Error, futures, reqwest, serde_json, tokio_core};
use futures::Future;
use serde::Deserialize;
use transport::{Method, Request, Response, StatusCode, Transport};
use url::Url;

#[derive(Clone, Debug)]
pub struct NetTransport {
    server_url: Url,
    http_client: reqwest::unstable::async::Client,
}

impl NetTransport {
    pub fn new_with_external_executor(
        server_url: Url,
        reactor_handle: &tokio_core::reactor::Handle,
    ) -> Result<Self, Error> {
        Ok(NetTransport {
            server_url: server_url,
            http_client: reqwest::unstable::async::Client::new(reactor_handle)
                .map_err(|e| (("Failed to construct HTTP client", e)))?,
        })
    }
}

impl Transport for NetTransport {
    type Request = NetRequest;

    // TODO: Unbox this type.
    type RequestFuture = Box<Future<Item = Self::Request, Error = Error>>;

    fn request<P: AsRef<str>>(&self, method: Method, url_path: Result<P, Error>) -> Self::RequestFuture {
        Box::new(futures::future::result(url_path.and_then(move |p| {
            let mut url = self.server_url.clone();
            url.set_path(p.as_ref());
            let method_clone = method.clone();
            self.http_client
                .request(method, url)
                .map_err(|e| Error::from(("Failed to construct HTTP request", e)))
                .map(move |x| NetRequest::new(method_clone, x))
        })))
    }
}

#[derive(Debug)]
pub struct NetRequest {
    method: reqwest::Method,
    http_request_builder: reqwest::unstable::async::RequestBuilder,
}

impl NetRequest {
    fn new(method: reqwest::Method, http_request_builder: reqwest::unstable::async::RequestBuilder) -> Self {
        NetRequest {
            method: method,
            http_request_builder: http_request_builder,
        }
    }
}

impl Request for NetRequest {
    type Response = NetResponse;

    // TODO: Unbox this type.
    type Future = Box<Future<Item = Self::Response, Error = Error>>;

    fn accept_application_json(&mut self) {
        self.http_request_builder.header(
            reqwest::header::Accept::json(),
        );
    }

    fn send_without_body(mut self) -> Self::Future {
        let method = self.method;
        Box::new(
            self.http_request_builder
                .send()
                .map_err(|e| Error::from(("Failed to complete HTTP request", e)))
                .map(move |x| NetResponse::new(method, x)),
        )
    }
}

#[derive(Debug)]
pub struct NetResponse {
    method: reqwest::Method,
    http_response: reqwest::unstable::async::Response,
}

impl NetResponse {
    fn new(method: reqwest::Method, http_response: reqwest::unstable::async::Response) -> Self {
        NetResponse {
            method: method,
            http_response: http_response,
        }
    }
}

impl Response for NetResponse {
    fn status_code(&self) -> StatusCode {
        self.http_response.status()
    }

    fn json_body<T>(&mut self) -> Box<Future<Item = T, Error = Error>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        // This special check for HEAD is a workaround for a bug in the reqwest
        // crate (as of 0.7.1), whereby it hangs when polling the body of a HEAD
        // response.
        //
        // When the bug is resolved upstream, then we should remove the `method`
        // field from the NetRequest and NetResponse structs, as well as this
        // special check.
        //
        // Bug URL: https://github.com/seanmonstar/reqwest/issues/167

        if self.method == reqwest::Method::Head {
            return Box::new(futures::future::result(
                serde_json::from_slice(b"").map_err(|e| {
                    Error::from(("Failed to decode HTTP response body as JSON", e))
                }),
            ));
        }

        Box::new(self.http_response.json().map_err(|e| {
            Error::from(("Failed to decode HTTP response body as JSON", e))
        }))
    }
}
