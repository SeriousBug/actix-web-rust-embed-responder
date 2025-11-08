use actix_web::{
    body::{BoxBody, MessageBody},
    http::Method,
    HttpRequest, HttpResponse, Responder,
};

#[cfg(feature = "compression-zstd")]
use crate::compress_data_zstd;
use crate::{
    compress::Compress, compress_data_br, compress_data_gzip, helper::accepts_encoding,
    is_well_known_compressible_mime_type, parse::parse_if_none_match_value,
};

/// A common trait used internally to create HTTP responses.
///
/// This trait is internally implemented for both `rust-embed` and
/// `rust-embed-for-web` types. You could also implement it for your own type if
/// you wish to use the response handling capabilities of this crate without
/// embedded files.
pub trait EmbedRespondable {
    type Data: MessageBody + 'static + AsRef<[u8]>;
    type DataGzip: MessageBody + 'static + AsRef<[u8]>;
    type DataBr: MessageBody + 'static + AsRef<[u8]>;
    type DataZstd: MessageBody + 'static + AsRef<[u8]>;
    type MimeType: AsRef<str>;
    type ETag: AsRef<str>;
    type LastModified: AsRef<str>;

    /// The contents of the embedded file.
    fn data(&self) -> Self::Data;
    /// The contents of the file compressed with gzip.
    ///
    /// `Some` if precompression has been done, `None` if the file was not precompressed.
    fn data_gzip(&self) -> Option<Self::DataGzip>;
    /// The contents of the file compressed with brotli.
    ///
    /// `Some` if precompression has been done, `None` if the file was not precompressed.
    fn data_br(&self) -> Option<Self::DataBr>;
    /// The contents of the file compressed with zstd.
    ///
    /// `Some` if precompression has been done, `None` if the file was not precompressed.
    fn data_zstd(&self) -> Option<Self::DataZstd>;
    /// The UNIX timestamp of when the file was last modified.
    fn last_modified_timestamp(&self) -> Option<i64>;
    /// The rfc2822 encoded last modified date.
    fn last_modified(&self) -> Option<Self::LastModified>;
    /// The ETag value for the file, based on its hash.
    fn etag(&self) -> Self::ETag;
    /// The mime type for the file, if one has been guessed.
    fn mime_type(&self) -> Option<Self::MimeType>;
}

/// An opaque wrapper around the embedded file.
///
/// You don't manually create these objects, you should use `.into_response()`
/// or `.into()` to convert an embedded file into an `EmbedResponse`.
pub struct EmbedResponse<T: EmbedRespondable> {
    pub(crate) file: Option<T>,
    pub(crate) compress: Compress,
}

enum ShouldCompress {
    Zstd,
    Brotli,
    Gzip,
    No,
}

fn should_compress<T: EmbedRespondable>(
    req: &HttpRequest,
    file: &T,
    compress: &Compress,
) -> ShouldCompress {
    let should_compress_for_encoding =
        |is_precompressed_for_encoding: bool, mime_type: Option<T::MimeType>, encoding: &str| {
            accepts_encoding(req, encoding)
                && match compress {
                    Compress::Never => false,
                    Compress::IfPrecompressed => is_precompressed_for_encoding,
                    Compress::IfWellKnown => mime_type
                        .map(|v| is_well_known_compressible_mime_type(v.as_ref()))
                        .unwrap_or(false),
                    Compress::Always => true,
                }
        };

    if should_compress_for_encoding(file.data_zstd().is_some(), file.mime_type(), "zstd") {
        ShouldCompress::Zstd
    } else if should_compress_for_encoding(file.data_br().is_some(), file.mime_type(), "br") {
        ShouldCompress::Brotli
    } else if should_compress_for_encoding(file.data_gzip().is_some(), file.mime_type(), "gzip") {
        ShouldCompress::Gzip
    } else {
        ShouldCompress::No
    }
}

fn send_response<T: EmbedRespondable>(
    req: &HttpRequest,
    file: &T,
    compress: Compress,
) -> HttpResponse {
    let mut resp = HttpResponse::Ok();

    resp.append_header(("ETag", file.etag().as_ref()));
    if let Some(last_modified) = file.last_modified() {
        resp.append_header(("Last-Modified", last_modified.as_ref()));
    }
    if let Some(mime_type) = file.mime_type() {
        resp.append_header(("Content-Type", mime_type.as_ref()));
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
        let encoding_choice = should_compress(req, file, &compress);
        match encoding_choice {
            #[cfg(feature = "compression-zstd")]
            ShouldCompress::Zstd => {
                resp.append_header(("Content-Encoding", "zstd"));
                match file.data_zstd() {
                    Some(data_zstd) => resp.body(data_zstd),
                    None => resp.body(compress_data_zstd(
                        file.etag().as_ref(),
                        file.data().as_ref(),
                    )),
                }
            }
            ShouldCompress::Brotli => {
                resp.append_header(("Content-Encoding", "br"));
                match file.data_br() {
                    Some(data_br) => resp.body(data_br),
                    None => resp.body(compress_data_br(file.etag().as_ref(), file.data().as_ref())),
                }
            }
            ShouldCompress::Gzip => {
                resp.append_header(("Content-Encoding", "gzip"));
                match file.data_gzip() {
                    Some(data_gzip) => resp.body(data_gzip),
                    None => resp.body(compress_data_gzip(
                        file.etag().as_ref(),
                        file.data().as_ref(),
                    )),
                }
            }
            #[cfg(not(feature = "compression-zstd"))]
            ShouldCompress::Zstd => {
                // This should never happen, but if it does, just serve uncompressed
                resp.body(file.data())
            }
            ShouldCompress::No => resp.body(file.data()),
        }
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
                        return send_response(req, &file, self.compress);
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
                            return send_response(req, &file, self.compress);
                        } else {
                            return HttpResponse::NotModified().finish();
                        }
                    }
                }
                // If there was no `If-Unmodified-Since` header either, that
                // means the client does not have this file cached.
                send_response(req, &file, self.compress)
            }
            None => HttpResponse::NotFound().finish(),
        }
    }
}

impl<T: EmbedRespondable> EmbedResponse<T> {
    /// Set the compression option to use for this response. Please see the
    /// Compress type for allowed options.
    pub fn use_compression(mut self, option: Compress) -> Self {
        self.compress = option;
        self
    }
}

/// A specialized version of `Into`, which can help you avoid specifying the type in `Into'.
pub trait IntoResponse<T: EmbedRespondable> {
    /// A specialized version of `Into::into`.
    fn into_response(self) -> EmbedResponse<T>;
}
