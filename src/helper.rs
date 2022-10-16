use actix_web::HttpRequest;

use crate::parse::parse_accept_encoding_value;

pub(crate) fn accepts_gzip(req: &HttpRequest) -> bool {
    req.headers()
        .get("Accept-Encoding")
        .map(parse_accept_encoding_value)
        .map(|encodings| encodings.contains(&"gzip"))
        .unwrap_or(false)
}
