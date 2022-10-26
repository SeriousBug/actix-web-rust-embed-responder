use crate::embed::{EmbedRespondable, EmbedResponse};
use rust_embed_for_web::EmbeddedFile;

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
    type Data = &'static [u8];
    type DataGzip = &'static [u8];
    type ETag = &'static str;
    type LastModified = &'static str;

    fn data(&self) -> Self::Data {
        self.data
    }

    fn data_gzip(&self) -> Option<Self::DataGzip> {
        self.data_gzip
    }

    fn last_modified(&self) -> Option<Self::LastModified> {
        self.metadata.last_modified
    }

    fn last_modified_timestamp(&self) -> Option<i64> {
        self.metadata.last_modified_timestamp
    }

    fn etag(&self) -> Self::ETag {
        self.metadata.etag
    }

    fn mime_type(&self) -> Option<&str> {
        self.metadata.mime_type
    }
}
