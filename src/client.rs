use hyper;
use serde;
use serde_json;

use command;
use design::Design;
use document::{
    DesignDocumentType,
    NormalDocumentType,
    Revision};
use error::{self, Error};

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
                uri: try!(
                    uri.into_url()
                    .or_else(|e| {
                        Err(Error::UriParse { cause: e } )
                    })
                ),
            },
        })
    }

    /// Get the server URI the client connects to.
    pub fn uri(&self) -> &hyper::Url {
        &self.state.uri
    }

    /// Build a command to GET all database names.
    pub fn get_all_databases(&'a self) -> command::GetAllDatabases<'a> {
        command::new_get_all_databases(&self.state)
    }

    /// Build a command to HEAD a database.
    pub fn head_database(
        &'a self,
        db_name: &'a str)
        -> command::HeadDatabase<'a>
    {
        command::new_head_database(&self.state, db_name)
    }

    /// Build a command to GET a database.
    pub fn get_database(
        &'a self,
        db_name: &'a str)
        -> command::GetDatabase<'a>
    {
        command::new_get_database(&self.state, db_name)
    }

    /// Build a command to PUT a database.
    pub fn put_database(
        &'a self,
        db_name: &'a str)
        -> command::PutDatabase<'a>
    {
        command::new_put_database(&self.state, db_name)
    }

    /// Build a command to DELETE a database.
    pub fn delete_database(
        &'a self,
        db_name: &'a str)
        -> command::DeleteDatabase<'a>
    {
        command::new_delete_database(&self.state, db_name)
    }

    /// Build a command to HEAD a document.
    pub fn head_document(
        &'a self,
        db_name: &'a str,
        doc_id: &'a str)
        -> command::HeadDocument<'a, NormalDocumentType>
    {
        command::new_head_document::<NormalDocumentType>(
            &self.state,
            db_name,
            doc_id)
    }

    /// Build a command to GET a document.
    pub fn get_document<
        T: serde::Deserialize>(
            &'a self,
            db_name: &'a str,
            doc_id: &'a str)
        -> command::GetDocument<'a, NormalDocumentType, T>
    {
        command::new_get_document::<NormalDocumentType, T>(
            &self.state,
            db_name,
            doc_id)
    }

    /// Build a command to PUT a document.
    pub fn put_document<
        T: serde::Serialize>(
            &'a self,
            db_name: &'a str,
            doc_id: &'a str,
            doc_content: &'a T)
        -> command::PutDocument<'a, NormalDocumentType, T>
    {
        command::new_put_document::<NormalDocumentType, T>(
            &self.state,
            db_name,
            doc_id,
            doc_content)
    }

    /// Build a command to DELETE a document.
    pub fn delete_document(
        &'a self,
        db_name: &'a str,
        doc_id: &'a str,
        rev: &'a Revision)
        -> command::DeleteDocument<'a, NormalDocumentType>
    {
        command::new_delete_document::<NormalDocumentType>(
            &self.state,
            db_name,
            doc_id,
            rev)
    }

    /// Build a command to HEAD a design document.
    pub fn head_design_document(
        &'a self,
        db_name: &'a str,
        ddoc_id: &'a str)
        -> command::HeadDocument<'a, DesignDocumentType>
    {
        command::new_head_document::<DesignDocumentType>(
            &self.state,
            db_name,
            ddoc_id)
    }

    /// Build a command to GET a design document.
    pub fn get_design_document(
        &'a self,
        db_name: &'a str,
        ddoc_id: &'a str)
        -> command::GetDocument<'a, DesignDocumentType, Design>
    {
        command::new_get_document::<DesignDocumentType, Design>(
            &self.state,
            db_name,
            ddoc_id)
    }

    /// Build a command to PUT a design document.
    pub fn put_design_document(
        &'a self,
        db_name: &'a str,
        ddoc_id: &'a str,
        ddoc_content: &'a Design)
        -> command::PutDocument<'a, DesignDocumentType, Design>
    {
        command::new_put_document::<DesignDocumentType, Design>(
            &self.state,
            db_name,
            ddoc_id,
            ddoc_content)
    }

    /// Build a command to DELETE a design document.
    pub fn delete_design_document(
        &'a self,
        db_name: &'a str,
        ddoc_id: &'a str,
        rev: &'a Revision)
        -> command::DeleteDocument<'a, DesignDocumentType>
    {
        command::new_delete_document::<DesignDocumentType>(
            &self.state,
            db_name,
            ddoc_id,
            rev)
    }

    /// Build a command to GET a view.
    pub fn get_view<K, V>(
        &'a self,
        db_name: &'a str,
        ddoc_id: &'a str,
        view_name: &'a str)
        -> command::GetView<'a, K, V>
        where K: serde::Deserialize + serde::Serialize,
              V: serde::Deserialize
    {
        command::new_get_view(&self.state, db_name, ddoc_id, view_name)
    }
}

/// Helper function for checking that an HTTP response has application/json
/// Content-Type and returning an error if not.
pub fn require_content_type_application_json(headers: &hyper::header::Headers)
                                             -> Result<(), Error > {
    match headers.get::<hyper::header::ContentType>() {
        None => Err(Error::NoContentTypeHeader {
            expected: "application/json",
        }),
        Some(content_type) => {
            use hyper::mime::*;
            let exp = hyper::mime::Mime(TopLevel::Application, SubLevel::Json, vec![]);
            let &hyper::header::ContentType(ref got) = content_type;
            if *got != exp {
                Err(Error::UnexpectedContentTypeHeader {
                    expected: "application/json",
                    got: format!("{}", got)
                })
            } else {
                Ok(())
            }
        },
    }
}

/// Helper function for reading application/json content from an HTTP response.
pub fn read_json_response(resp: &mut hyper::client::Response) -> Result<String, Error> {
    use std::io::Read;
    try!(require_content_type_application_json(&resp.headers));
    let mut s = String::new();
    try!(
        resp.read_to_string(&mut s)
        .or_else(|e| {
            Err(Error::Transport { cause: error::TransportCause::Io(e) } )
        })
    );
    Ok(s)
}

/// Helper function for decoding a JSON string.
pub fn decode_json<T: serde::Deserialize>(s: &String) -> Result<T, Error> {
    let x = try!(
        serde_json::from_str::<T>(&s)
        .or_else(|e| {
            Err(Error::Decode { cause: e } )
        })
    );
    Ok(x)
}
