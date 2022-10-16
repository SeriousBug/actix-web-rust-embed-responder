use actix_web::http::header::HeaderValue;
use lazy_static::lazy_static;
use regex::Regex;

pub(crate) fn parse_if_none_match_value(value: &HeaderValue) -> Option<Vec<&str>> {
    parse_comma_seperated_list(value, parse_single_etag_value)
}

pub(crate) fn parse_accept_encoding_value(value: &HeaderValue) -> Option<Vec<&str>> {
    parse_comma_seperated_list(value, parse_single_encoding_value)
}

fn parse_comma_seperated_list<'h>(
    value: &'h HeaderValue,
    parse_item: fn(&str) -> Option<&str>,
) -> Option<Vec<&'h str>> {
    value.to_str().ok().map(|v| {
        v.split(',')
            .into_iter()
            .filter_map(parse_item)
            .collect::<Vec<&'h str>>()
    })
}

fn parse_single_etag_value(value: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^ *(W/)?(?P<value>"[^"]+") *$"#).unwrap();
    }

    RE.captures(value)
        .and_then(|v| v.name("value"))
        .map(|v| v.as_str())
}

fn parse_single_encoding_value(value: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^ *(?P<value>[^ ;,]+) *$"#).unwrap();
    }

    RE.captures(value)
        .and_then(|v| v.name("value"))
        .map(|v| v.as_str())
}
