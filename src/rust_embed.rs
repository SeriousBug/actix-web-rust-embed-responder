use chrono::TimeZone;
use rust_embed::EmbeddedFile;

use crate::embed::{EmbedRespondable, EmbedResponse};

impl From<EmbeddedFile> for EmbedResponse<EmbeddedFile> {
    fn from(file: EmbeddedFile) -> Self {
        EmbedResponse {
            file: Some(file),
            compress: Default::default(),
        }
    }
}

impl From<Option<EmbeddedFile>> for EmbedResponse<EmbeddedFile> {
    fn from(file: Option<EmbeddedFile>) -> Self {
        EmbedResponse {
            file,
            compress: Default::default(),
        }
    }
}

impl EmbedRespondable for EmbeddedFile {
    type Data = Vec<u8>;
    type DataGzip = Vec<u8>;
    type ETag = String;
    type LastModified = String;

    fn data(&self) -> Self::Data {
        self.data.clone().into_owned()
    }

    fn data_gzip(&self) -> Option<Self::DataGzip> {
        None
    }

    fn last_modified(&self) -> Option<Self::LastModified> {
        self.last_modified_timestamp()
            .map(|timestamp| chrono::Utc.timestamp(timestamp, 0).to_rfc2822())
    }

    fn last_modified_timestamp(&self) -> Option<i64> {
        self.metadata
            .last_modified()
            // The last_modified value in rust-embed is u64, but it really
            // should be i64. We'll try a safe conversion here.
            .and_then(|v| v.try_into().ok())
    }

    fn etag(&self) -> Self::ETag {
        format!("\"{}\"", base64::encode(self.metadata.sha256_hash()))
    }

    fn mime_type(&self) -> Option<&str> {
        // rust-embed doesn't include the filename for the embedded file, so we
        // can't guess the mime type. We could add `xdg-mime` to guess based on
        // contents, but it will require the shared mime database to be
        // available at runtime. In any case, it's okay if we just let the
        // browser guess the mime type.
        None
    }
}
