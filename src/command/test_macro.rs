macro_rules! expect_couchdb_error {
    ($result:expr, $expected_error:ident) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                use Error;
                match *e {
                    Error::$expected_error(..) => (),
                    _ => {
                        panic!("Unexpected error variant: Expected {}, got: {}",
                               stringify!($expected_error),
                               e);
                    }
                }
            }
        }
    }
}

macro_rules! expect_request_method {
    ($request:expr, $expected_method:expr) => {
        assert_eq!(*$request.method(), $expected_method);
    }
}

macro_rules! expect_request_uri {
    ($request:expr, $expected_uri:expr) => {
        assert_eq!(*$request.uri(), hyper::Url::parse($expected_uri).unwrap());
    }
}

macro_rules! expect_request_accept_application_json {
    ($request:expr) => {
        match $request.headers().get::<hyper::header::Accept>() {
            None => {
                panic!("Missing header to Accept application/json");
            }
            Some(ref qitems) => {
                // No need to check for quality part. Any quality is okay.
                use hyper::mime::{Mime, TopLevel, SubLevel};
                let expected_mime = Mime(TopLevel::Application, SubLevel::Json, vec![]);
                let has = qitems.iter().any(|x| x.item == expected_mime);
                if !has {
                    panic!("Missing header to Accept application/json, got {:?}", qitems);
                }
            }
        }
    }
}

macro_rules! expect_request_body {
    ($request:expr, $expected_body:expr) => {
        assert_eq!($request.body, $expected_body);
    }
}

macro_rules! expect_request_content_type_application_json {
    ($request:expr) => {
        match $request.headers().get::<hyper::header::ContentType>() {
            None => {
                panic!("Missing header to Accept application/json");
            }
            Some(&hyper::header::ContentType(ref got_mime)) => {
                // No need to check for quality part. Any quality is okay.
                use hyper::mime::{Mime, TopLevel, SubLevel};
                let expected_mime = Mime(TopLevel::Application, SubLevel::Json, vec![]);
                assert_eq!(*got_mime, expected_mime);
            }
        }
    }
}

macro_rules! expect_request_if_match_revision {
    ($request:expr, $expected_tag:expr) => {
        match $request.headers().get::<hyper::header::IfMatch>() {
            None => {
                panic!("Missing If-Match header");
            }
            Some(header) => {
                match *header {
                    hyper::header::IfMatch::Any => {
                        panic!("Wrong If-Match header: Expected revision, got Any");
                    }
                    hyper::header::IfMatch::Items(ref tags)=> {
                        use Revision;
                        let rev = Revision::parse($expected_tag).unwrap();
                        let tag = hyper::header::EntityTag::new(false, rev.to_string());
                        let expected_tags = vec![tag];
                        assert_eq!(expected_tags, *tags);
                    }
                }
            }
        }
    }
}

macro_rules! expect_request_if_none_match_revision {
    ($request:expr, $expected_tag:expr) => {
        match $request.headers().get::<hyper::header::IfNoneMatch>() {
            None => {
                panic!("Missing If-None-Match header");
            }
            Some(header) => {
                match *header {
                    hyper::header::IfNoneMatch::Any => {
                        panic!("Wrong If-None-Match header: Expected revision, got Any");
                    }
                    hyper::header::IfNoneMatch::Items(ref tags)=> {
                        let rev = Revision::parse($expected_tag).unwrap();
                        let tag = hyper::header::EntityTag::new(false, rev.to_string());
                        let expected_tags = vec![tag];
                        assert_eq!(expected_tags, *tags);
                    }
                }
            }
        }
    }
}
