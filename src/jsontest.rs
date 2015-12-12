use serde_json;
use std;

fn join_json_string<'a, I, J>(head: I, tail: J) -> String
    where I: Iterator<Item = &'a &'a str>,
          J: Iterator<Item = &'a &'a str>
{

    fn append(mut acc: String, item: &&str) -> String {
        if !acc.ends_with("{") {
            acc.push_str(", ");
        }
        acc.push_str(item);
        acc
    }

    let s = head.fold("{".to_string(), append);
    let s = tail.fold(s, append);
    let mut s = s;
    s.push_str("}");
    s
}

pub fn make_complete_json_object(fields: &[&str]) -> String {
    join_json_string(std::iter::empty(), fields.into_iter())
}

pub fn make_json_object_with_missing_field(fields: &[&str], exclude: &str) -> String {
    let exclude = format!(r#""{}""#, exclude);
    let pos = fields.into_iter()
                    .position(|&item| item.starts_with(&exclude))
                    .unwrap();
    join_json_string(fields.into_iter().take(pos),
                     fields.into_iter().skip(pos + 1))
}

// Asserts that an error is a 'missing field' error.
pub fn assert_missing_field(e: &serde_json::Error, _exp_field_name: &str) {

    if let serde_json::Error::SyntaxError(ref code, ref _line, ref _column) = *e {

        // There's a error-reporting bug in serde_json that makes this check
        // impossible. See here: https://github.com/serde-rs/json/issues/22.
        //
        // When this bug is resolved, the following block of code should work
        // when uncommented. Until then, we use the workaround below.

        /* if let serde_json::ErrorCode::MissingField(got_field_name) = *code {
         * assert_eq!(got_field_name, exp_field_name);
         * return;
         * }
         * panic!("Got unexpected error code variant: {:?}", code);
         * */

        if let serde_json::ErrorCode::ExpectedSomeValue = *code {
            return;
        }
        panic!("Got unexpected error code variant: {:?}", code);

    }

    panic!("Got unexpected error variant: {}", e);
}

// Asserts that an error is an `unknown field` error.
pub fn assert_unknown_field(e: &serde_json::Error, exp_field_name: &str) {

    if let serde_json::Error::SyntaxError(ref code, ref _line, ref _column) = *e {
        if let serde_json::ErrorCode::UnknownField(ref got_field_name) = *code {
            assert_eq!(got_field_name, exp_field_name);
            return;
        }
        panic!("Got unexpected error code variant: {:?}", code);
    }

    panic!("Got unexpected error variant: {}", e);
}
