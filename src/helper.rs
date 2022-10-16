use actix_web::HttpRequest;
use chrono::{DateTime, FixedOffset};

use crate::parse::{parse_accept_encoding_value, parse_if_none_match_value};

pub(crate) fn accepts_gzip(req: &HttpRequest) -> bool {
    req.headers()
        .get("Accept-Encoding")
        .and_then(parse_accept_encoding_value)
        .map(|encodings| encodings.contains(&"gzip"))
        .unwrap_or(false)
}

pub(crate) enum MatchType {
    /// Client has this resource cached, we can respond with 304.
    Cached,
    /// Client does not have this resource cached, we need to send the resource.
    NotCached,
    /// Client may or may not have this resource cached, check other methods.
    NotApplicable,
}
