extern crate couchdb;
extern crate serde_json;

use couchdb::{DatabasePath, DocumentPath, ViewPath};

enum Db {
    Baseball,
    Basketball,
}

impl Into<DatabasePath> for Db {
    fn into(self) -> DatabasePath {
        match self {
            Db::Baseball => DatabasePath::from("baseball"),
            Db::Basketball => DatabasePath::from("basketball"),
        }
    }
}

#[derive(Debug)]
enum Doc {
    BabeRuth,
    HankAaron,
    MyDesign,
}

impl Into<DocumentPath> for Doc {
    fn into(self) -> DocumentPath {
        match self {
            Doc::BabeRuth => DocumentPath::new(Db::Baseball, "babe_ruth"),
            Doc::HankAaron => DocumentPath::new(Db::Baseball, "hank_aaron"),
            Doc::MyDesign => DocumentPath::new(Db::Baseball, "_design/my_design"),
        }
    }
}

#[test]
fn head_database() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    // Verify: Heading an existing database succeeds.
    client.put_database(Db::Baseball).run().unwrap();
    client.head_database(Db::Baseball).run().unwrap();

    // Verify: Heading a non-existing database fails.
    match client.head_database(Db::Basketball).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn get_database() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    // Verify: Getting an existing database succeeds.
    client.put_database(Db::Baseball).run().unwrap();
    client.get_database(Db::Baseball).run().unwrap();

    // Verify: Getting a non-existing database fails.
    match client.get_database(Db::Basketball).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn put_database() {

    use std::collections::HashSet;

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    let get_all_databases = || {
        client.get_all_databases()
              .run()
              .unwrap()
              .into_iter()
              .collect::<HashSet<_>>()
    };

    // Verify: Putting a non-existing database succeeds.
    let pre = get_all_databases();
    client.put_database(Db::Baseball).run().unwrap();
    let post = get_all_databases();
    assert_eq!(0 as usize, pre.difference(&post).count());
    let db_path: DatabasePath = Db::Baseball.into();
    let db_name: String = db_path.into();
    let exp = vec![db_name];
    let got = post.difference(&pre).map(|x| x.as_ref().to_string()).collect::<Vec<String>>();
    assert_eq!(exp, got);

    // Verify: Putting an existing database fails.
    match client.put_database(Db::Baseball).run().unwrap_err() {
        couchdb::Error::DatabaseExists { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Putting a database with an invalid name fails.
    let db_path = DatabasePath::from("_database_names_cannot_start_with_an_underscore");
    match client.put_database(db_path)
                .run()
                .unwrap_err() {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn delete_database() {

    use std::collections::HashSet;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    let get_all_databases = || {
        client.get_all_databases()
              .run()
              .unwrap()
              .into_iter()
              .collect::<HashSet<_>>()
    };

    // Verify: Deleting an existing database succeeds.
    let pre = get_all_databases();
    client.put_database(Db::Baseball).run().unwrap();
    client.delete_database(Db::Baseball).run().unwrap();
    let post = get_all_databases();
    assert_eq!(pre, post);

    // Verify: Deleting an non-existing database fails.
    match client.delete_database(Db::Baseball).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Deleting a database with an invalid name fails.
    let path = DatabasePath::from("_database_names_cannot_start_with_an_underscore");
    match client.delete_database(path)
                .run()
                .unwrap_err() {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn head_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database(Db::Baseball).run().unwrap();
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .unwrap();
    let rev1 = client.put_document(Doc::BabeRuth, &pdoc).run().unwrap();

    // Verify: Heading an existing document succeeds.
    client.head_document(Doc::BabeRuth).run().unwrap().unwrap();

    // Verify: Heading an existing document with a matching If-None-Match header
    // succeeds.
    assert!(client.head_document(Doc::BabeRuth)
                  .if_none_match(&rev1)
                  .run()
                  .unwrap()
                  .is_none());

    // Verify: Heading an existing document with a stale, non-matching
    // If-None-Match header succeeds.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .insert("hr", 714)
                   .unwrap();
    client.put_document(Doc::BabeRuth, &pdoc)
          .if_match(&rev1)
          .run()
          .unwrap();
    client.head_document(Doc::BabeRuth)
          .if_none_match(&rev1)
          .run()
          .unwrap()
          .unwrap();

    // Verify: Heading a non-existing document fails.
    match client.head_document(Doc::HankAaron).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }
}

#[test]
fn get_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database(Db::Baseball).run().unwrap();
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "babe_ruth")
                   .unwrap();
    let rev1 = client.put_document(Doc::BabeRuth, &pdoc).run().unwrap();

    // Verify: Getting an existing document succeeds.
    let doc1 = client.get_document::<_, serde_json::Value>(Doc::BabeRuth)
                     .run()
                     .unwrap()
                     .unwrap();
    assert_eq!(doc1.path, Doc::BabeRuth.into());
    assert_eq!(doc1.revision, rev1);
    assert_eq!(doc1.content, pdoc);

    // Verify: Getting a non-existing document fails.
    let e = client.get_document::<_, serde_json::Value>(Doc::HankAaron)
                  .run()
                  .unwrap_err();
    match e {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Getting an existing document with a matching If-None-Match header
    // succeeds with no document returned.
    match client.get_document::<_, serde_json::Value>(Doc::BabeRuth)
                .if_none_match(&rev1)
                .run()
                .unwrap() {
        Some(_) => {
            panic!("Got document, expected none");
        }
        None => (),
    }

    // Verify: Getting an existing document with a stale, non-matching
    // If-None-Match header succeeds.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "babe_ruth")
                   .insert("hr", 714)
                   .unwrap();
    let rev2 = client.put_document(Doc::BabeRuth, &pdoc)
                     .if_match(&rev1)
                     .run()
                     .unwrap();
    let doc2 = client.get_document::<_, serde_json::Value>(Doc::BabeRuth)
                     .if_none_match(&rev1)
                     .run()
                     .unwrap()
                     .unwrap();
    assert_eq!(doc2.revision, rev2);
}

#[test]
fn put_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database(Db::Baseball).run().unwrap();

    // Verify: Putting a non-existing document succeeds.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .unwrap();
    let rev1 = client.put_document(Doc::BabeRuth, &pdoc).run().unwrap();

    // Verify: Putting an existing document without a revision fails.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .insert("hr", 714)
                   .unwrap();
    match client.put_document(Doc::BabeRuth, &pdoc).run().unwrap_err() {
        couchdb::Error::DocumentConflict { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Putting an existing document with a matching revision succeeds.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .insert("hr", 714)
                   .unwrap();
    client.put_document(Doc::BabeRuth, &pdoc)
          .if_match(&rev1)
          .run()
          .unwrap();

    // Verify: Putting an existing document with a non-matching revision fails.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .insert("hr", 714)
                   .insert("hits", 2873)
                   .unwrap();
    match client.put_document(Doc::BabeRuth, &pdoc)
                .if_match(&rev1)
                .run()
                .unwrap_err() {
        couchdb::Error::DocumentConflict { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn delete_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database(Db::Baseball).run().unwrap();
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .unwrap();
    let rev1 = client.put_document(Doc::BabeRuth, &pdoc).run().unwrap();

    // Verify: Deleting an existing document with matching revision succeeds.
    client.delete_document(Doc::BabeRuth, &rev1).run().unwrap();
    match client.head_document(Doc::BabeRuth).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Deleting a non-existing document fails.
    match client.delete_document(Doc::BabeRuth, &rev1).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }

    // Verify: Deleting a document with a (stale) non-matching revision fails.
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Hank Aaron")
                   .unwrap();
    let rev2 = client.put_document(Doc::HankAaron, &pdoc).run().unwrap();
    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Hank Aaron")
                   .insert("hr", 755)
                   .unwrap();
    client.put_document(Doc::HankAaron, &pdoc)
          .if_match(&rev2)
          .run()
          .unwrap();
    match client.delete_document(Doc::HankAaron, &rev2).run().unwrap_err() {
        couchdb::Error::DocumentConflict { .. } => (),
        e => {
            panic!("Got unexpected error: {}", e);
        }
    }
}

#[test]
fn get_view() {

    enum View {
        ByName,
        ByHr,
    };

    impl Into<ViewPath> for View {
        fn into(self) -> ViewPath {
            match self {
                View::ByName => ViewPath::new(Doc::MyDesign, "by_name"),
                View::ByHr => ViewPath::new(Doc::MyDesign, "by_hr"),
            }
        }
    }

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database(Db::Baseball).run().unwrap();

    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert("by_name".to_string(),
                       couchdb::ViewFunction {
                           map: "function(doc) { emit(doc.name, doc.name); }".to_string(),
                           reduce: None,
                       });
    pddoc.views.insert("by_hr".to_string(),
                       couchdb::ViewFunction {
                           map: "function(doc) { emit(doc.name, doc.hr); }".to_string(),
                           reduce: Some("function(keys, values) { return sum(values); }".to_string()),
                       });
    client.put_document(Doc::MyDesign, &pddoc).run().unwrap();

    // Verify: Getting an empty view succeeds.
    let result = client.get_view::<_, String, u32>(View::ByName)
                       .run()
                       .unwrap();
    assert_eq!(Some(0), result.total_rows);
    assert_eq!(Some(0), result.offset);
    assert!(result.rows.is_empty());

    // Populate the database with some documents.

    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Babe Ruth")
                   .insert("hr", 714)
                   .unwrap();
    client.put_document(Doc::BabeRuth, &pdoc).run().unwrap();

    let pdoc = serde_json::builder::ObjectBuilder::new()
                   .insert("name", "Hank Aaron")
                   .insert("hr", 755)
                   .unwrap();
    client.put_document(Doc::HankAaron, &pdoc).run().unwrap();

    // Verify: Getting a nonempty view without 'reduce' succeeds.

    let result = client.get_view::<_, String, String>(View::ByName)
                       .run()
                       .unwrap();
    assert_eq!(Some(2), result.total_rows);
    assert_eq!(Some(0), result.offset);
    assert_eq!(result.rows,
               vec![
            couchdb::ViewRow::<String, String> {
                path: Some(Doc::BabeRuth.into()),
                key: Some("Babe Ruth".to_string()),
                value: "Babe Ruth".to_string(),
            },
            couchdb::ViewRow::<String, String> {
                path: Some(Doc::HankAaron.into()),
                key: Some("Hank Aaron".to_string()),
                value: "Hank Aaron".to_string(),
            },
        ]);

    // Verify: Getting a nonempty view with 'reduce' disabled succeeds.

    let result = client.get_view::<_, String, u32>(View::ByHr)
                       .reduce(false)
                       .run()
                       .unwrap();
    assert_eq!(Some(2), result.total_rows);
    assert_eq!(Some(0), result.offset);
    assert_eq!(result.rows,
               vec![
            couchdb::ViewRow::<String, u32> {
                path: Some(Doc::BabeRuth.into()),
                key: Some("Babe Ruth".to_string()),
                value: 714,
            },
            couchdb::ViewRow::<String, u32> {
                path: Some(Doc::HankAaron.into()),
                key: Some("Hank Aaron".to_string()),
                value: 755,
            },
        ]);

    // Verify: Getting a reduced view succeeds.

    let result = client.get_view::<_, String, u32>(View::ByHr)
                       .run()
                       .unwrap();
    assert_eq!(None, result.total_rows);
    assert_eq!(None, result.offset);
    assert_eq!(result.rows,
               vec![
            couchdb::ViewRow::<String, u32> {
                path: None,
                key: None,
                value: 714 + 755,
            },
        ]);

    // Verify: Getting a view with an explicit start-key succeeds.
    let result = client.get_view::<_, String, u32>(View::ByHr)
                       .reduce(false)
                       .startkey("h".to_string())
                       .run()
                       .unwrap();
    assert_eq!(Some(2), result.total_rows);
    assert_eq!(Some(1), result.offset);
    assert_eq!(result.rows,
               vec![
            couchdb::ViewRow::<String, u32> {
                path: Some(Doc::HankAaron.into()),
                key: Some("Hank Aaron".to_string()),
                value: 755,
            },
        ]);

    // Verify: Getting a view with an explicit end-key succeeds.
    let result = client.get_view::<_, String, u32>(View::ByHr)
                       .reduce(false)
                       .endkey("h".to_string())
                       .run()
                       .unwrap();
    assert_eq!(Some(2), result.total_rows);
    assert_eq!(Some(0), result.offset);
    assert_eq!(result.rows,
               vec![
            couchdb::ViewRow::<String, u32> {
                path: Some(Doc::BabeRuth.into()),
                key: Some("Babe Ruth".to_string()),
                value: 714,
            },
        ]);
}
