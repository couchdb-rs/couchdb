extern crate couchdb;
extern crate serde_json;

#[test]
#[allow(dead_code)]
fn main() {

    use std::collections::{BTreeMap, HashSet};
    type M = BTreeMap<String, serde_json::Value>;

    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();

    let is_server_clean = || {
        let all = client.get_all_databases().run().unwrap();
        let mut our = all.iter().filter(|x| {
            !x.starts_with("_")
        });
        our.next().is_none()
    };

    assert!(is_server_clean());

    // = PUT and DELETE database =

    {
        // Verify: Putting a database succeeds.
        let pre: HashSet<_> = client.get_all_databases().run()
            .unwrap()
            .into_iter().collect();
        client.put_database("cats").run().unwrap();
        let post: HashSet<_> = client.get_all_databases().run()
            .unwrap()
            .into_iter().collect();
        assert_eq!(0 as usize, pre.difference(&post).count());
        assert_eq!(vec!["cats"], post.difference(&pre).map(|x| x.clone()).collect::<Vec<String>>());

        // Verify: Putting an existing database fails.
        let x = client.put_database("cats").run().unwrap_err();
        match x {
            couchdb::Error::DatabaseExists { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Putting a database with an invalid name fails.
        let x = client.put_database("_valid_names_cannot_start_with_an_underscore").run()
            .unwrap_err();
        match x {
            couchdb::Error::InvalidDatabaseName { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        /*

        The following two tests fail. The proximate problem is the CouchDB
        server sends a 404 Not Found instead of a 400 Bad Request. However, the
        ultimate problem is that Hyper doesn't percent-encode paths, so our
        client sends a PUT document request ("cats/moochie"), not a PUT database
        request ("cats%2Fmoochie").
        
        See this bug report for more details:
        https://github.com/hyperium/hyper/issues/638
        
        // Verify: Putting a database with a slash ('/') in the name fails as an invalid database
        // name.
        let x = client.put_database("cats/moochie").run().unwrap_err();
        match x {
            couchdb::Error::InvalidDatabaseName { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Putting a database with a slash ('/') in the name fails as an invalid database
        // name.
        let x = client.put_database("dogs/fido").run().unwrap_err();
        match x {
            couchdb::Error::InvalidDatabaseName { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        */

        // Verify: Deleting an existing database succeeds.
        client.delete_database("cats").run().unwrap(); 
        let post: HashSet<_> = client.get_all_databases().run().unwrap().into_iter().collect();
        assert_eq!(pre, post);

        // Verify: Deleting an non-existing database fails.
        let x = client.delete_database("cats").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Deleting a database with an invalid name fails.
        let x = client.delete_database("_invalid_name").run().unwrap_err();
        match x {
            couchdb::Error::InvalidDatabaseName { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }
    }

    assert!(is_server_clean());

    // = HEAD and GET database =

    {
        // Verify: Heading a non-existing database fails.
        let x = client.head_database("cats").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Heading an existing database succeeds.
        client.put_database("cats").run().unwrap();
        client.head_database("cats").run().unwrap();

        // Verify: Getting an existing database succeeds.
        client.get_database("cats").run().unwrap();

        // Verify: Getting a non-existing database fails.
        let x = client.get_database("dogs").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        client.delete_database("cats").run().unwrap();
    }

    assert!(is_server_clean());

    // = PUT and DELETE document =

    {
        client.put_database("cats").run().unwrap();

        // Verify: Putting a non-existing document succeeds.
        let mut doc = serde_json::Value::Object(M::new());
        doc.as_object_mut().unwrap().insert("name".to_string(), serde_json::to_value(&"Nutmeg"));
        doc.as_object_mut().unwrap().insert("years_old".to_string(), serde_json::to_value(&7));
        let rev = client.put_document("cats", "nutmeg", &doc).run().unwrap();

        // Verify: Putting an existing document without a revision fails.
        doc.as_object_mut().unwrap().insert("color".to_string(), serde_json::to_value(&"orange"));
        let x = client.put_document("cats", "nutmeg", &doc).run().unwrap_err();
        match x {
            couchdb::Error::DocumentConflict { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Putting an existing document with a matching revision succeeds.
        doc.as_object_mut().unwrap().insert("color".to_string(), serde_json::to_value(&"orange"));
        let rev2 = client.put_document("cats", "nutmeg", &doc)
            .if_match(&rev)
            .run().unwrap();

        // Verify: Putting an existing document with a non-matching revision fails.
        doc.as_object_mut().unwrap().insert("color".to_string(), serde_json::to_value(&"orange"));
        let x = client.put_document("cats", "nutmeg", &doc)
            .if_match(&rev)
            .run().unwrap_err();
        match x {
            couchdb::Error::DocumentConflict { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Deleting an existing document with match revision succeeds.
        client.delete_document("cats", "nutmeg", &rev2).run().unwrap();
        match client.get_document::<serde_json::Value>("cats", "nutmeg").run().unwrap_err() {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Deleting a non-existing document fails.
        match client.delete_document("cats", "nutmeg", &rev2).run().unwrap_err() {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Deleting a document with a stale revision fails.
        let mut doc = serde_json::Value::Object(M::new());
        doc.as_object_mut().unwrap().insert("name".to_string(), serde_json::to_value(&"Emerald"));
        doc.as_object_mut().unwrap().insert("years_old".to_string(), serde_json::to_value(&6));
        let rev = client.put_document("cats", "emerald", &doc).run().unwrap();
        doc.as_object_mut().unwrap().insert("color".to_string(), serde_json::to_value(&"brown"));
        let rev2 = client.put_document("cats", "emerald", &doc)
            .if_match(&rev)
            .run().unwrap();
        match client.delete_document("cats", "emerald", &rev).run().unwrap_err() {
            couchdb::Error::DocumentConflict { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }
        client.delete_document("cats", "emerald", &rev2).run().unwrap();

        client.delete_database("cats").run().unwrap();
    }

    assert!(is_server_clean());

    // = HEAD and GET document =

    {
        client.put_database("cats").run().unwrap();

        // Verify: Getting an existing document succeeds.
        let mut doc = serde_json::Value::Object(M::new());
        doc.as_object_mut().unwrap().insert("name".to_string(), serde_json::to_value(&"Emerald"));
        doc.as_object_mut().unwrap().insert("years_old".to_string(), serde_json::to_value(&6));
        client.put_document("cats", "emerald", &doc).run().unwrap();
        let doc = client.get_document::<serde_json::Value>("cats", "emerald")
            .run().unwrap().unwrap();
        assert!(doc.id == "emerald");
        let rev = doc.revision;

        // Verify: Getting a non-existing document fails.
        let x = client.get_document::<serde_json::Value>("cats", "nutmeg")
            .run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => { panic!("Got error: {}", x); },
        }

        // Verify: Getting an existing document with an If-None-Match header succeeds with no
        // document returned.

        match client.get_document::<serde_json::Value>("cats", "emerald")
            .if_none_match(&rev)
            .run()
            .unwrap()
        {
            Some(_) => { panic!("Got document, expected none"); },
            None => (),
        }

        // Verify: Heading an existing document succeeds.
        client.head_document("cats", "emerald").run().unwrap().unwrap();

        // Verify: Heading an existing document with an If-None-Match header
        // succeeds.
        let x = client.head_document("cats", "emerald")
            .if_none_match(&rev)
            .run()
            .unwrap();
        assert!(x.is_none());

        // Verify: Heading a non-existing document fails.
        let x = client.head_document("cats", "nutmeg").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => panic!("Got error: {}", x),
        }

        client.delete_database("cats").run().unwrap();
    }

    assert!(is_server_clean());

    // = HEAD, GET, PUT, and DELETE design document =

    {
        client.put_database("cats").run().unwrap();

        // Verify: Heading a non-existing design document fails.
        let x = client.head_design_document("cats", "my_design").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => panic!("Got error: {}", x),
        }

        // Verify: Getting a non-existing design document fails.
        let x = client.get_design_document("cats", "my_design").run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => panic!("Got error: {}", x),
        }

        // Verify: Putting a non-existing design document succeeds.
        let mut ddoc = couchdb::DesignDocument::new();
        ddoc.views.insert(
            "names".to_string(),
            couchdb::ViewFunction {
                map: "function(doc) { emit(doc.name); }".to_string(),
                reduce: None
            });
        let rev = client.put_design_document("cats", "my_design", &ddoc).run().unwrap();

        // Verify: Heading an existing design document succeeds.
        client.head_design_document("cats", "my_design").run().unwrap().unwrap();

        // Verify: Heading an existing design document with explicit revision
        // succeeds.
        let x = client.head_design_document("cats", "my_design")
            .if_none_match(&rev)
            .run()
            .unwrap();
        assert!(x.is_none());

        // Verify: Getting an existing document succeeds.
        let got = client.get_design_document("cats", "my_design").run().unwrap().unwrap();
        assert_eq!(rev, got.revision);
        assert_eq!(ddoc, got.content);

        // Verify: Deleting an existing document succeeds.
        client.delete_design_document("cats", "my_design", &rev).run().unwrap();

        // Verify: Deleting a non-existing document fails.
        let x = client.delete_design_document("cats", "my_design", &rev).run().unwrap_err();
        match x {
            couchdb::Error::NotFound { .. } => (),
            _ => panic!("Got error: {}", x),
        }

        client.delete_database("cats").run().unwrap();
    }

    assert!(is_server_clean());

    // = GET view =

    {
        client.put_database("cats").run().unwrap();

        let mut ddoc = couchdb::DesignDocument::new();
        ddoc.views.insert(
            "names".to_string(),
            couchdb::ViewFunction {
                map: "function(doc) { emit(doc.name, doc.name.length); }".to_string(),
                reduce: Some("function(keys, values) { return sum(values); }".to_string()),
            });
        client.put_design_document("cats", "my_design", &ddoc).run().unwrap();

        // Verify: Getting an empty view succeeds.
        let result = client.get_view::<String, u32>("cats", "my_design", "names").run().unwrap();
        assert_eq!(0, result.total_rows);
        assert_eq!(0, result.offset);
        assert!(result.rows.is_empty());

        let mut doc = serde_json::Value::Object(M::new());
        doc.as_object_mut().unwrap().insert("name".to_string(), serde_json::to_value(&"Emerald"));
        doc.as_object_mut().unwrap().insert("years_old".to_string(), serde_json::to_value(&6));
        client.put_document("cats", "emerald", &doc).run().unwrap();

        let mut doc = serde_json::Value::Object(M::new());
        doc.as_object_mut().unwrap().insert("name".to_string(), serde_json::to_value(&"Nutmeg"));
        doc.as_object_mut().unwrap().insert("years_old".to_string(), serde_json::to_value(&7));
        client.put_document("cats", "nutmeg", &doc).run().unwrap();

        // Verify: Getting a nonempty view succeeds.
        let result = client.get_view::<String, u32>("cats", "my_design", "names")
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
                    value: 7,
                },
                couchdb::ViewRow::<String, u32> {
                    id: Some("nutmeg".to_string()),
                    key: Some("Nutmeg".to_string()),
                    value: 6
                },
            ]);

        // Verify: Getting a view with an explicit start-key succeeds.
        let result = client.get_view::<String, u32>("cats", "my_design", "names")
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
                    value: 6
                },
            ]);

        // Verify: Getting a view with an explicit end-key succeeds.
        let result = client.get_view::<String, u32>("cats", "my_design", "names")
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
                    value: 7,
                },
            ]);

        // Verify: Getting a reduced view succeeds.
        let result = client.get_view::<String, u32>("cats", "my_design", "names")
            .reduce(true)
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

        client.delete_database("cats").run().unwrap();
    }

    assert!(is_server_clean());
}
