use std::collections::HashMap;
use regex::Regex;

lazy_static! {
    static ref PAT_QUERY_STRING: Regex = Regex::new(r#"([a-zA-Z]+)=([^&]*)"#)
        .expect("Failed to compile query string regex");
}

pub fn parse_query(queryst: &str) -> HashMap<&str, &str> {
    let mut result = HashMap::new();
    for group in PAT_QUERY_STRING.captures_iter(queryst) {
        let key = group.get(1).unwrap().as_str();
        let value = group.get(2).unwrap().as_str();

        result.insert(key, value);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queryst() {
        assert_eq!(
            parse_query("q=foo&,searchProperty=baz"),
            hashmap![
                "q" => "foo",
                "searchProperty" => "baz"]);
    }
}
