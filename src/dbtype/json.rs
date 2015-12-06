#[cfg(test)]
use std;

#[cfg(test)]
fn join_json_string<'a, I, J>(head: I, tail: J) -> String
    where I: Iterator<Item = &'a &'a str>,
          J: Iterator<Item = &'a &'a str>
{

    fn append(mut acc: String, item: & &str) -> String {
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

#[cfg(test)]
pub fn make_complete_json_object(fields: &[&str]) -> String {
    join_json_string(std::iter::empty(), fields.into_iter())
}

#[cfg(test)]
pub fn make_json_object_with_missing_field(fields: &[&str], exclude: &str) -> String {
    let exclude = format!(r#""{}""#, exclude);
    let pos = fields.into_iter().position(|&item| {
        item.starts_with(&exclude)
    }).unwrap();
    join_json_string(
        fields.into_iter().take(pos),
        fields.into_iter().skip(pos+1))
}
