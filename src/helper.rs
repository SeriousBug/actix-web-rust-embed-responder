use crate::parse::parse_accept_encoding_value;
use actix_web::HttpRequest;

pub(crate) fn accepts_gzip(req: &HttpRequest) -> bool {
    req.headers()
        .get("Accept-Encoding")
        .and_then(parse_accept_encoding_value)
        .map(|encodings| encodings.contains(&"gzip"))
        .unwrap_or(false)
}
