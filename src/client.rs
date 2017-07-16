use {Error, IntoDatabasePath, action, tokio_core};
use std::marker::PhantomData;
use transport::NetTransport;
use url::Url;

/// `IntoUrl` converts a type into a `Url`.
///
/// # Summary
///
/// * `IntoUrl` is a workaround for Rust not yet having a stable `TryInto`
///   trait, and, as such, it will probably go away in a future release when
///   `TryInto` becomes stable.
///
pub trait IntoUrl {
    fn into_url(self) -> Result<Url, Error>;
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, Error> {
        Ok(self)
    }
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url, Error> {
        let u = Url::parse(self).map_err(|e| {
            ((format!("Failed to parse URL (url: {})", self), e))
        })?;
        Ok(u)
    }
}

impl<'a> IntoUrl for &'a String {
    fn into_url(self) -> Result<Url, Error> {
        self.as_str().into_url()
    }
}

/// `Client` is an HTTP client that communicates with a CouchDB server.
///
/// # Summary
///
/// * `Client` is the starting point by which an application communicates with
///   a CouchDB server.
///
/// * `Client` is asynchronous and requires an external tokio reactor to drive
///   I/O.
///
/// * `Client` communicates with exactly one CouchDB server. To communicate with
///   multiple CouchDB servers, the application must construct multiple `Client`
///   instances.
///
/// * `Client` drives communication with the CouchDB server via
///   [actions](action/index.html), which are the `couchdb` crate's abstractions
///   over HTTP requests and responses.
///
/// # Example
///
/// The following example demonstrates creating a database and verifying that
/// the database was created.
///
/// ```
/// extern crate couchdb;
/// extern crate tokio_core;
///
/// # let server = couchdb::testing::FakeServer::new().unwrap();
/// # let server_url = server.url();
/// #
/// let mut reactor = tokio_core::reactor::Core::new().unwrap();
/// let client = couchdb::Client::new(
///     server_url, // e.g., "http::/example.com"
///     couchdb::ClientOptions::default(),
///     &reactor.handle()
/// ).unwrap();
///
/// match reactor.run(client.head_database("/baseball").send()) {
///     Err(ref e) if e.is_not_found() => {}
///     x => panic!("Got unexpected result {:?}", x),
/// }
///
/// reactor.run(client.put_database("/baseball").send()).unwrap();
///
/// match reactor.run(client.head_database("/baseball").send()) {
///     Ok(_) => {}
///     x => panic!("Got unexpected result {:?}", x),
/// }
/// ```
///
#[derive(Debug)]
pub struct Client {
    transport: NetTransport,
}

impl Client {
    // TODO: Provide an alternative constructor that doesn't require an external
    // reactor. Currently, we cannot do this because the reqwest crate doesn't
    // support it.
    //
    // See this hyper issue for related information:
    // https://github.com/hyperium/hyper/issues/1002.
    //

    /// Constructs a client, given an asynchronous I/O reactor.
    pub fn new<U: IntoUrl>(
        server_url: U,
        _options: ClientOptions,
        reactor_handle: &tokio_core::reactor::Handle,
    ) -> Result<Self, Error> {

        let server_url = server_url.into_url()?;
        let transport = NetTransport::new_with_external_executor(server_url, reactor_handle)?;

        Ok(Client { transport: transport })
    }

    /// Constructs an action to GET the server's root resource (i.e., `/`).
    pub fn get_root(&self) -> action::GetRoot<NetTransport> {
        action::GetRoot::new(&self.transport)
    }

    /// Constructs an action to GET a list of all databases.
    pub fn get_all_databases(&self) -> action::GetAllDatabases<NetTransport> {
        action::GetAllDatabases::new(&self.transport)
    }

    /// Constructs an action to GET a database.
    pub fn get_database<P: IntoDatabasePath>(&self, db_path: P) -> action::GetDatabase<NetTransport> {
        action::GetDatabase::new(&self.transport, db_path)
    }

    /// Constructs an action to HEAD a database.
    pub fn head_database<P: IntoDatabasePath>(&self, db_path: P) -> action::HeadDatabase<NetTransport> {
        action::HeadDatabase::new(&self.transport, db_path)
    }

    /// Constructs an action to PUT a database.
    pub fn put_database<P: IntoDatabasePath>(&self, db_path: P) -> action::PutDatabase<NetTransport> {
        action::PutDatabase::new(&self.transport, db_path)
    }

    /// Constructs an action to DELETE a database.
    pub fn delete_database<P: IntoDatabasePath>(&self, db_path: P) -> action::DeleteDatabase<NetTransport> {
        action::DeleteDatabase::new(&self.transport, db_path)
    }
}

/// `ClientOptions` contains options for configuring a
/// [`Client`](struct.Client.html) instance with non-default behavior.
///
/// # Summary
///
/// * Currently, there are no non-default client options.
///
/// * An application should use `ClientOptions::default()` to construct a
///   default set of options.
///
#[derive(Debug, Default)]
pub struct ClientOptions {
    _apps_cannot_construct_this: PhantomData<()>,
}

impl ClientOptions {
    // Not exposed because there's no need to expose it yet. Applications can
    // use ClientOptions::default() instead.
    #[doc(hidden)]
    pub fn new() -> Self {
        ClientOptions::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_url_converts_string_ok() {
        let u = "http://example.com:5984/foo/bar".into_url().unwrap();
        assert_eq!(u.path(), "/foo/bar");
    }

    #[test]
    fn into_url_fails_for_invalid_string() {
        "not_a_valid_url".into_url().unwrap_err();
    }
}
