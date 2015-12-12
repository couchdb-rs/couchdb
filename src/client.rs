use hyper;
use serde;

use command;
use dbpath::DatabasePath;
use docpath::DocumentPath;
use error::Error;
use revision::Revision;
use viewpath::ViewPath;

/// Trait for converting some types into a URI.
pub trait IntoUrl: hyper::client::IntoUrl {}
impl<T: hyper::client::IntoUrl> IntoUrl for T {}

// Encapsulates state borrowed by command instances. The reason this struct is separated from
// Client is so that the struct fields aren't exposed in this crate's public API.
pub struct ClientState {
    pub http_client: hyper::Client,
    pub uri: hyper::Url,
}

/// CouchDB client.
///
/// The Client is the principal type for communicating with a CouchDB server.
/// All CouchDB commands (e.g., PUT database, GET document, etc.) go through a
/// Client instance.
///
/// A Client communicates with exactly one CouchDB server, as specified by its
/// URI during Client construction.
///
pub struct Client {
    state: ClientState,
}

impl<'a> Client {
    /// Construct a CouchDB client.
    pub fn new<U: IntoUrl>(uri: U) -> Result<Client, Error> {
        Ok(Client {
            state: ClientState {
                http_client: hyper::Client::new(),
                uri: try!(uri.into_url()
                             .or_else(|e| Err(Error::UriParse { cause: e }))),
            },
        })
    }

    /// Get the server URI the client connects to.
    pub fn uri(&self) -> &hyper::Url {
        &self.state.uri
    }

    /// Build a command to GET all database names.
    pub fn get_all_databases(&'a self) -> command::GetAllDatabases<'a> {
        command::GetAllDatabases::new_get_all_databases(&self.state)
    }

    /// Build a command to HEAD a database.
    pub fn head_database<P>(&'a self, path: P) -> command::HeadDatabase<'a>
        where P: Into<DatabasePath>
    {
        command::HeadDatabase::new_head_database(&self.state, path.into())
    }

    /// Build a command to GET a database.
    pub fn get_database<P>(&'a self, path: P) -> command::GetDatabase<'a>
        where P: Into<DatabasePath>
    {
        command::GetDatabase::new_get_database(&self.state, path.into())
    }

    /// Build a command to PUT a database.
    pub fn put_database<P>(&'a self, path: P) -> command::PutDatabase<'a>
        where P: Into<DatabasePath>
    {
        command::PutDatabase::new_put_database(&self.state, path.into())
    }

    /// Build a command to DELETE a database.
    pub fn delete_database<P>(&'a self, path: P) -> command::DeleteDatabase<'a>
        where P: Into<DatabasePath>
    {
        command::DeleteDatabase::new_delete_database(&self.state, path.into())
    }

    /// Build a command to HEAD a document.
    pub fn head_document<P>(&'a self, path: P) -> command::HeadDocument<'a>
        where P: Into<DocumentPath>
    {
        command::HeadDocument::new_head_document(&self.state, path.into())
    }

    /// Build a command to GET a document.
    pub fn get_document<P, T>(&'a self, path: P) -> command::GetDocument<'a, T>
        where P: Into<DocumentPath>,
              T: serde::Deserialize
    {
        command::GetDocument::new_get_document(&self.state, path.into())
    }

    /// Build a command to PUT a document.
    pub fn put_document<P, T>(&'a self, path: P, doc_content: &'a T) -> command::PutDocument<'a, T>
        where P: Into<DocumentPath>,
              T: serde::Serialize
    {
        command::PutDocument::new_put_document(&self.state, path.into(), doc_content)
    }

    /// Build a command to DELETE a document.
    pub fn delete_document<P>(&'a self, path: P, rev: &'a Revision) -> command::DeleteDocument<'a>
        where P: Into<DocumentPath>
    {
        command::DeleteDocument::new_delete_document(&self.state, path.into(), rev)
    }

    /// Build a command to GET a view.
    pub fn get_view<P, K, V>(&'a self, path: P) -> command::GetView<'a, K, V>
        where P: Into<ViewPath>,
              K: serde::Deserialize + serde::Serialize,
              V: serde::Deserialize
    {
        command::GetView::new_get_view(&self.state, path.into())
    }
}
