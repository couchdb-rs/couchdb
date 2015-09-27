extern crate couchdb;
extern crate serde_json;

#[test]
fn head_database() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    // Verify: Heading an existing database succeeds.
    client.put_database("cats").run().unwrap();
    client.head_database("cats").run().unwrap();

    // Verify: Heading a non-existing database fails.
    match client.head_database("dogs").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }
}

#[test]
fn get_database() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    // Verify: Getting an existing database succeeds.
    client.put_database("cats").run().unwrap();
    client.get_database("cats").run().unwrap();

    // Verify: Getting a non-existing database fails.
    match client.get_database("dogs").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
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
    client.put_database("cats").run().unwrap();
    let post = get_all_databases();
    assert_eq!(0 as usize, pre.difference(&post).count());
    assert_eq!(
        vec!["cats"],
        post.difference(&pre).map(|x| x.clone()).collect::<Vec<String>>());

    // Verify: Putting an existing database fails.
    match client.put_database("cats").run().unwrap_err() {
        couchdb::Error::DatabaseExists { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Putting a database with an invalid name fails.
    match client.put_database("_database_names_cannot_start_with_an_underscore")
        .run().unwrap_err()
    {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    /*

    The following two tests fail. The proximate problem is the CouchDB server
    sends a 404 Not Found instead of a 400 Bad Request. However, the ultimate
    problem is the Url type doesn't percent-encode paths, so our client sends a
    PUT document request ("cats/moochie"), not a PUT database request
    ("cats%2Fmoochie").
    
    See this bug report for more details:
    https://github.com/hyperium/hyper/issues/638
    
    // Verify: Putting a database with a slash ('/') in the name fails as an
    // invalid database name.
    match client.put_database("cats/moochie").run().unwrap_err() {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Putting a database with a slash ('/') in the name fails as an
    // invalid database name.
    match client.put_database("dogs/fido").run().unwrap_err() {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    */
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
    client.put_database("cats").run().unwrap();
    client.delete_database("cats").run().unwrap();
    let post = get_all_databases();
    assert_eq!(pre, post);

    // Verify: Deleting an non-existing database fails.
    match client.delete_database("cats").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Deleting a database with an invalid name fails.
    match client.delete_database("_database_names_cannot_start_with_an_underscore")
        .run().unwrap_err()
    {
        couchdb::Error::InvalidDatabaseName { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }
}

#[test]
fn head_document() {

    use serde_json::Value;
    type Object = std::collections::BTreeMap<String, Value>;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    let rev1 = client.put_document("cats", "nutmeg", &pdoc).run().unwrap();

    // Verify: Heading an existing document succeeds.
    client.head_document("cats", "nutmeg").run().unwrap().unwrap();

    // Verify: Heading an existing document with a matching If-None-Match header
    // succeeds.
    assert!(
        client.head_document("cats", "nutmeg")
            .if_none_match(&rev1)
            .run()
            .unwrap()
            .is_none());

    // Verify: Heading an existing document with a stale, non-matching
    // If-None-Match header succeeds.
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&7));
    client.put_document("cats", "nutmeg", &pdoc)
        .if_match(&rev1)
        .run()
        .unwrap();
    client.head_document("cats", "nutmeg")
        .if_none_match(&rev1)
        .run()
        .unwrap()
        .unwrap();

    // Verify: Heading a non-existing document fails.
    match client.head_document("cats", "emerald").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }
}

#[test]
fn get_document() {

    use serde_json::Value;
    type Object = std::collections::BTreeMap<String, Value>;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    let rev1 = client.put_document("cats", "nutmeg", &pdoc).run().unwrap();

    // Verify: Getting an existing document succeeds.
    let doc1 = client.get_document::<Value>("cats", "nutmeg")
        .run().unwrap().unwrap();
    assert!(doc1.id == "nutmeg");
    assert!(doc1.revision == rev1);
    assert!(doc1.content == pdoc);

    // Verify: Getting a non-existing document fails.
    match client.get_document::<Value>("cats", "emerald").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Getting an existing document with a matching If-None-Match header
    // succeeds with no document returned.
    match client.get_document::<Value>("cats", "nutmeg")
        .if_none_match(&rev1)
        .run()
        .unwrap()
    {
        Some(_) => { panic!("Got document, expected none"); },
        None => (),
    }

    // Verify: Getting an existing document with a stale, non-matching
    // If-None-Match header succeeds.
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&7));
    let rev2 = client.put_document("cats", "nutmeg", &pdoc)
        .if_match(&rev1)
        .run()
        .unwrap();
    let doc2 = client.get_document::<Value>("cats", "nutmeg")
        .if_none_match(&rev1)
        .run()
        .unwrap()
        .unwrap();
    assert_eq!(doc2.revision, rev2);
}

#[test]
fn put_document() {

    use serde_json::Value;
    type Object = std::collections::BTreeMap<String, Value>;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    // Verify: Putting a non-existing document succeeds.
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "name".to_string(),
        serde_json::to_value(&"Nutmeg"));
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&7));
    let rev1 = client.put_document("cats", "nutmeg", &pdoc).run().unwrap();

    // Verify: Putting an existing document without a revision fails.
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    match client.put_document("cats", "nutmeg", &pdoc).run().unwrap_err() {
        couchdb::Error::DocumentConflict { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Putting an existing document with a matching revision succeeds.
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    let rev2 = client.put_document("cats", "nutmeg", &pdoc)
        .if_match(&rev1)
        .run().unwrap();
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&8));
    client.put_document("cats", "nutmeg", &pdoc)
        .if_match(&rev2)
        .run().unwrap();

    // Verify: Putting an existing document with a non-matching revision fails.
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    match client.put_document("cats", "nutmeg", &pdoc)
        .if_match(&rev1)
        .run().unwrap_err()
    {
        couchdb::Error::DocumentConflict { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }
}

#[test]
fn delete_document() {

    use serde_json::Value;
    type Object = std::collections::BTreeMap<String, Value>;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"orange"));
    let rev1 = client.put_document("cats", "nutmeg", &pdoc).run().unwrap();

    // Verify: Deleting an existing document with matching revision succeeds.
    client.delete_document("cats", "nutmeg", &rev1).run().unwrap();
    match client.head_document("cats", "nutmeg").run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Deleting a non-existing document fails.
    match client.delete_document("cats", "nutmeg", &rev1).run().unwrap_err() {
        couchdb::Error::NotFound { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }

    // Verify: Deleting a document with a (stale) non-matching revision fails.
    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "color".to_string(),
        serde_json::to_value(&"brown"));
    let rev2 = client.put_document("cats", "emerald", &pdoc).run().unwrap();
    pdoc.as_object_mut().unwrap().insert(
        "eyes".to_string(),
        serde_json::to_value(&"green"));
    client.put_document("cats", "emerald", &pdoc)
        .if_match(&rev2)
        .run().unwrap();
    match client.delete_document("cats", "emerald", &rev2).run().unwrap_err() {
        couchdb::Error::DocumentConflict { .. } => (),
        e => { panic!("Got unexpected error: {}", e); },
    }
}

#[test]
fn head_design_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert(
        "by_name".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: None,
        });
    let rev1 = client.put_design_document("cats", "my_design", &pddoc)
        .run().unwrap();

    // Verify: Heading an existing design document succeeds.
    client.head_design_document("cats", "my_design")
        .run()
        .unwrap()
        .unwrap();

    // Verify: Heading a non-existing design document fails.
    match client.head_design_document("cats", "does_not_exist")
        .run()
        .unwrap_err()
    {
        couchdb::Error::NotFound { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }

    // Verify: Heading an existing design document with a matching If-None-Match
    // header succeeds.
    assert!(
        client.head_design_document("cats", "my_design")
            .if_none_match(&rev1)
            .run()
            .unwrap()
            .is_none());
}

#[test]
fn get_design_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert(
        "by_name".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: None,
        });
    let rev1 = client.put_design_document("cats", "my_design", &pddoc)
        .run().unwrap();

    // Verify: Getting an existing design document succeeds.
    let ddoc = client.get_design_document("cats", "my_design")
        .run()
        .unwrap()
        .unwrap();
    assert_eq!(ddoc.id, "my_design".to_string());
    assert_eq!(ddoc.revision, rev1);

    // Verify: Getting a non-existing design document fails.
    match client.get_design_document("cats", "my_design")
            .run()
            .unwrap_err()
    {
        couchdb::Error::NotFound { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }

    // Verify: Getting an existing design document with a matching If-None-Match
    // header succeeds.
    assert!(
        client.get_design_document("cats", "my_design")
            .if_none_match(&rev1)
            .run()
            .unwrap()
            .is_none());
}

#[test]
fn put_design_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    // Verify: Putting a non-existing design document succeeds.
    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert(
        "by_name".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: None,
        });
    let rev1 = client.put_design_document("cats", "my_design", &pddoc)
        .run()
        .unwrap();

    // Verify: Putting an existing design document without an If-Match header
    // fails.
    pddoc.views.insert(
        "by_age".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.years_old); }".to_string(),
            reduce: None,
        });
    match client.put_design_document("cats", "my_design", &pddoc)
        .run()
        .unwrap_err()
    {
        couchdb::Error::DocumentConflict { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }

    // Verify: Putting an existing design document with a matching If-Match
    // header fails.
    client.put_design_document("cats", "my_design", &pddoc)
        .if_match(&rev1)
        .run()
        .unwrap();

    // Verify: Putting an existing design document with a non-matching If-Match
    // header fails.
    match client.put_design_document("cats", "my_design", &pddoc)
        .if_match(&rev1)
        .run()
        .unwrap_err()
    {
        couchdb::Error::DocumentConflict { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }
}

#[test]
fn delete_design_document() {

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert(
        "by_name".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name); }".to_string(),
            reduce: None,
        });
    let rev1 = client.put_design_document("cats", "my_design", &pddoc)
        .run()
        .unwrap();

    pddoc.views.insert(
        "by_age".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.years_old); }".to_string(),
            reduce: None,
        });
    let rev2 = client.put_design_document("cats", "my_design", &pddoc)
        .if_match(&rev1)
        .run()
        .unwrap();

    // Verify: Deleting an existing design document with a non-matching revision
    // fails.
    match client.delete_design_document("cats", "my_design", &rev1)
        .run()
        .unwrap_err()
    {
        couchdb::Error::DocumentConflict { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }

    // Verify: Deleting an existing design document with a matching revision
    // succeeds.
    client.delete_design_document("cats", "my_design", &rev2)
        .run()
        .unwrap();

    // Verify: Deleting a non-existing design document fails.
    match client.delete_design_document("cats", "does_not_exist", &rev1)
        .run()
        .unwrap_err()
    {
        couchdb::Error::NotFound { .. } => (),
        e => panic!("Got unexpected error: {}", e),
    }
}

#[test]
fn get_view() {

    use serde_json::Value;
    type Object = std::collections::BTreeMap<String, Value>;
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    client.put_database("cats").run().unwrap();

    let mut pddoc = couchdb::Design::new();
    pddoc.views.insert(
        "name".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name, doc.name); }".to_string(),
            reduce: None,
        });
    pddoc.views.insert(
        "age".to_string(),
        couchdb::ViewFunction {
            map: "function(doc) { emit(doc.name, doc.years_old); }".to_string(),
            reduce: Some("function(keys, values) { return sum(values); }".to_string()),
        });
    client.put_design_document("cats", "my_design", &pddoc).run().unwrap();

    // Verify: Getting an empty view succeeds.
    let result = client.get_view::<String, u32>("cats", "my_design", "name")
        .run()
        .unwrap();
    assert_eq!(0, result.total_rows);
    assert_eq!(0, result.offset);
    assert!(result.rows.is_empty());

    // Populate the database with some documents.

    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "name".to_string(),
        serde_json::to_value(&"Nutmeg"));
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&7));
    client.put_document("cats", "nutmeg", &pdoc).run().unwrap();

    let mut pdoc = Value::Object(Object::new());
    pdoc.as_object_mut().unwrap().insert(
        "name".to_string(),
        serde_json::to_value(&"Emerald"));
    pdoc.as_object_mut().unwrap().insert(
        "years_old".to_string(),
        serde_json::to_value(&6));
    client.put_document("cats", "emerald", &pdoc).run().unwrap();

    // Verify: Getting a nonempty view without 'reduce' succeeds.

    let result = client.get_view::<String, String>("cats", "my_design", "name")
        .run().unwrap();
    assert_eq!(2, result.total_rows);
    assert_eq!(0, result.offset);
    assert_eq!(
        result.rows,
        vec![
            couchdb::ViewRow::<String, String> {
                id: Some("emerald".to_string()),
                key: Some("Emerald".to_string()),
                value: "Emerald".to_string(),
            },
            couchdb::ViewRow::<String, String> {
                id: Some("nutmeg".to_string()),
                key: Some("Nutmeg".to_string()),
                value: "Nutmeg".to_string(),
            },
        ]);

    // Verify: Getting a nonempty view with 'reduce' disabled succeeds.

    let result = client.get_view::<String, u32>("cats", "my_design", "age")
        .reduce(false)
        .run().unwrap();
    assert_eq!(2, result.total_rows);
    assert_eq!(0, result.offset);
    assert_eq!(
        result.rows,
        vec![
            couchdb::ViewRow::<String, u32> {
                id: Some("emerald".to_string()),
                key: Some("Emerald".to_string()),
                value: 6,
            },
            couchdb::ViewRow::<String, u32> {
                id: Some("nutmeg".to_string()),
                key: Some("Nutmeg".to_string()),
                value: 7,
            },
        ]);

    // Verify: Getting a reduced view succeeds.

    let result = client.get_view::<String, u32>("cats", "my_design", "age")
        .run().unwrap();
    assert_eq!(0, result.total_rows);
    assert_eq!(0, result.offset);
    assert_eq!(
        result.rows,
        vec![
            couchdb::ViewRow::<String, u32> {
                id: None,
                key: None,
                value: 13,
            },
        ]);

    // Verify: Getting a view with an explicit start-key succeeds.
    let result = client.get_view::<String, u32>("cats", "my_design", "age")
        .reduce(false)
        .startkey("f".to_string())
        .run().unwrap();
    assert_eq!(2, result.total_rows);
    assert_eq!(1, result.offset);
    assert_eq!(
        result.rows,
        vec![
            couchdb::ViewRow::<String, u32> {
                id: Some("nutmeg".to_string()),
                key: Some("Nutmeg".to_string()),
                value: 7,
            },
        ]);

    // Verify: Getting a view with an explicit end-key succeeds.
    let result = client.get_view::<String, u32>("cats", "my_design", "age")
        .reduce(false)
        .endkey("f".to_string())
        .run().unwrap();
    assert_eq!(2, result.total_rows);
    assert_eq!(0, result.offset);
    assert_eq!(
        result.rows,
        vec![
            couchdb::ViewRow::<String, u32> {
                id: Some("emerald".to_string()),
                key: Some("Emerald".to_string()),
                value: 6,
            },
        ]);
}
