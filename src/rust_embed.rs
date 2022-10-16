use std::io::Write;

use actix_web::{body::BoxBody, http::Method, HttpRequest, HttpResponse, Responder};
use chrono::TimeZone;
use flate2::Compression;

use crate::{helper::accepts_gzip, parse::parse_if_none_match_value};

pub struct EmbeddedFileResponse {
    embedded_file: rust_embed::EmbeddedFile,
}

impl From<rust_embed::EmbeddedFile> for EmbeddedFileResponse {
    fn from(embedded_file: rust_embed::EmbeddedFile) -> Self {
        EmbeddedFileResponse { embedded_file }
    }
}

impl Responder for EmbeddedFileResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        // This responder can't respond to anything other than GET and HEAD requests.
        if req.method() != Method::GET && req.method() != Method::HEAD {
            return HttpResponse::NotImplemented().finish();
        }

        // For the ETag we are using the sha256 hash of the file, encoded with
        // base64. We surround it with quotes as per the spec.
        let etag = format!(
            "\"{}\"",
            base64::encode(self.embedded_file.metadata.sha256_hash())
        );

        let last_modified_date = self
            .embedded_file
            .metadata
            .last_modified()
            .and_then(|timestamp| TryFrom::<u64>::try_from(timestamp).ok())
            .map(|timestamp| chrono::Utc.timestamp(timestamp, 0))
            // TODO
            .unwrap();
        let last_modified = last_modified_date.to_rfc2822();

        // Handle If-None-Match requests. If the client has the file cached
        // already, it can send back the ETag to ask for the file only if it has
        // changed.
        if let Some(req_etags) = req
            .headers()
            .get("If-None-Match")
            .and_then(parse_if_none_match_value)
        {
            if req_etags.contains(&etag.as_str()) {
                return HttpResponse::NotModified().finish();
            } else {
                return respond(req, self, &etag, Some(&last_modified));
            }
        }

        // Handle If-Unmodified-Since requests. As a fallback to ETag, the client
        // can also check if a file has been modified using the last modified
        // timestamp of the file.
        if let Some(if_unmodified_since) = req
            .headers()
            .get("If-Unmodified-Since")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(if_unmodified_since) =
                chrono::DateTime::parse_from_rfc2822(if_unmodified_since).ok()
            {
                // It's been modified since then
                if last_modified_date > if_unmodified_since {
                    return respond(req, self, &etag, Some(&last_modified));
                } else {
                    return HttpResponse::NotModified().finish();
                }
            }
        }

        // Otherwise, the client doesn't have the file cached and we do need to
        // send a response.
        respond(req, self, etag.as_str(), Some(&last_modified))
    }
}

fn respond(
    req: &HttpRequest,
    file: EmbeddedFileResponse,
    etag: &str,
    last_modified: Option<&str>,
) -> HttpResponse {
    let mut resp = HttpResponse::Ok();
    resp.append_header(("ETag", etag));

    if let Some(last_modified) = last_modified {
        resp.append_header(("Last-Modified", last_modified));
    }

    // This doesn't actually mean "no caching", it means revalidate before
    // using. If we don't add this, web browsers don't try to revalidate assets
    // like attached scripts and images. The users of this crate may or may not
    // be using fingerprinting or versioning on their assets, without this their
    // caching could break.
    resp.append_header(("Cache-Control", "no-cache"));

    let file_data = file.embedded_file.data.clone().into_owned();

    // TODO: This about potentially limiting this to only sometimes. Or maybe use a builder pattern to let the user decide if they want compression.
    if accepts_gzip(req) {
        let mut compressed: Vec<u8> = Vec::new();
        flate2::write::GzEncoder::new(&mut compressed, Compression::fast())
            .write_all(file_data.as_ref())
            .unwrap();
        resp.append_header(("Content-Encoding", "gzip"));
        return resp.body(compressed);
    }

    resp.body(file_data)
}
