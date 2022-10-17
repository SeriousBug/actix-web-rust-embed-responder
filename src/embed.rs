use actix_web::{
    body::{BoxBody, MessageBody},
    http::Method,
    HttpRequest, HttpResponse, Responder,
};

use crate::{helper::accepts_gzip, parse::parse_if_none_match_value};

/// A trait that both rust_embed and rust-embed-for-web implement. We implement
/// the responder in terms of this trait, so the code isn't duplicated for both
/// embed crates.
pub trait EmbedRespondable {
    type Data: MessageBody + 'static;
    type DataGzip: MessageBody + 'static;
    type ETag: AsRef<str>;
    type LastModified: AsRef<str>;

    fn data(&self) -> Self::Data;
    fn data_gzip(&self) -> Option<Self::DataGzip>;
    fn last_modified_timestamp(&self) -> Option<i64>;
    fn last_modified(&self) -> Option<Self::LastModified>;
    fn etag(&self) -> Self::ETag;
    fn mime_type(&self) -> Option<&str>;
}

pub struct EmbedResponse<T: EmbedRespondable> {
    pub(crate) file: Option<T>,
}

fn send_response<T: EmbedRespondable>(req: &HttpRequest, file: &T) -> HttpResponse {
    let mut resp = HttpResponse::Ok();

    resp.append_header(("ETag", file.etag().as_ref()));
    if let Some(last_modified) = file.last_modified() {
        resp.append_header(("Last-Modified", last_modified.as_ref()));
    }
    if let Some(mime_type) = file.mime_type() {
        resp.append_header(("Content-Type", mime_type));
    }

    // This doesn't actually mean "no caching", it means revalidate before
    // using. If we don't add this, web browsers don't try to revalidate assets
    // like attached scripts and images. The users of this crate may or may not
    // be using fingerprinting or versioning on their assets, without this their
    // caching could break.
    resp.append_header(("Cache-Control", "no-cache"));

    if req.method() == Method::HEAD {
        // For HEAD requests, we only need to send the headers and not the data.
        resp.finish()
    } else {
        // For GET requests, we do send the file body. Depending on whether the
        // client accepts compressed files or not, we may send the compressed
        // version.
        if accepts_gzip(req) {
            if let Some(data_gzip) = file.data_gzip() {
                resp.append_header(("Content-Encoding", "gzip"));
                return resp.body(data_gzip);
            }
        }
        resp.body(file.data())
    }
}

impl<T: EmbedRespondable> Responder for EmbedResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.file {
            Some(file) => {
                // This responder can't respond to anything other than GET and HEAD requests.
                if req.method() != Method::GET && req.method() != Method::HEAD {
                    return HttpResponse::NotImplemented().finish();
                }

                // For the ETag we are using the sha256 hash of the file, encoded with
                // base64. We surround it with quotes as per the spec.
                let e = file.etag();
                let etag = e.as_ref();

                let last_modified_timestamp = file.last_modified_timestamp();

                // Handle If-None-Match condition. If the client has the file cached
                // already, it can send back the ETag to ask for the file only if it has
                // changed.
                //
                // We first check If-None-Match because the spec specifies that it gets
                // priority over If-Modified-Since.
                if let Some(req_etags) = req
                    .headers()
                    .get("If-None-Match")
                    .and_then(parse_if_none_match_value)
                {
                    if req_etags.contains(&etag) {
                        return HttpResponse::NotModified().finish();
                    } else {
                        return send_response(req, &file);
                    }
                }
                // If there was no `If-None-Match` condition, check for
                // `If-Unmodified-Since` condition next. As a fallback to ETag,
                // the client can also check if a file has been modified using
                // the last modified time of the file.
                if let Some(last_modified_timestamp) = last_modified_timestamp {
                    if let Some(if_unmodified_since) = req
                        .headers()
                        .get("If-Unmodified-Since")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
                    {
                        // It's been modified since then
                        if last_modified_timestamp > if_unmodified_since.timestamp() {
                            return send_response(req, &file);
                        } else {
                            return HttpResponse::NotModified().finish();
                        }
                    }
                }
                // If there was no `If-Unmodified-Since` header either, that
                // means the client does not have this file cached.
                send_response(req, &file)
            }
            None => HttpResponse::NotFound().finish(),
        }
    }
}
