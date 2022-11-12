use crate::embed::{EmbedRespondable, EmbedResponse, IntoResponse};
use actix_web::body::MessageBody;
use rust_embed_for_web::{DynamicFile, EmbedableFile, EmbeddedFile};

#[cfg(all(debug_assertions, not(feature = "always-embed")))]
pub type EmbedableFileResponse = WebEmbedableFile<DynamicFile>;
#[cfg(any(not(debug_assertions), feature = "always-embed"))]
pub type EmbedableFileResponse = WebEmbedableFile<EmbeddedFile>;

impl From<EmbeddedFile> for EmbedResponse<WebEmbedableFile<EmbeddedFile>> {
    fn from(file: EmbeddedFile) -> Self {
        EmbedResponse {
            file: Some(WebEmbedableFile(file)),
            compress: Default::default(),
        }
    }
}

impl From<Option<EmbeddedFile>> for EmbedResponse<WebEmbedableFile<EmbeddedFile>> {
    fn from(file: Option<EmbeddedFile>) -> Self {
        EmbedResponse {
            file: file.map(|f| WebEmbedableFile(f)),
            compress: Default::default(),
        }
    }
}

impl IntoResponse<WebEmbedableFile<EmbeddedFile>> for EmbeddedFile {
    fn into_response(self) -> EmbedResponse<WebEmbedableFile<EmbeddedFile>> {
        self.into()
    }
}

impl IntoResponse<WebEmbedableFile<EmbeddedFile>> for Option<EmbeddedFile> {
    fn into_response(self) -> EmbedResponse<WebEmbedableFile<EmbeddedFile>> {
        self.into()
    }
}

impl From<DynamicFile> for EmbedResponse<WebEmbedableFile<DynamicFile>> {
    fn from(file: DynamicFile) -> Self {
        EmbedResponse {
            file: Some(WebEmbedableFile(file)),
            compress: Default::default(),
        }
    }
}

impl From<Option<DynamicFile>> for EmbedResponse<WebEmbedableFile<DynamicFile>> {
    fn from(file: Option<DynamicFile>) -> Self {
        EmbedResponse {
            file: file.map(|f| WebEmbedableFile(f)),
            compress: Default::default(),
        }
    }
}

impl IntoResponse<WebEmbedableFile<DynamicFile>> for DynamicFile {
    fn into_response(self) -> EmbedResponse<WebEmbedableFile<DynamicFile>> {
        self.into()
    }
}

impl IntoResponse<WebEmbedableFile<DynamicFile>> for Option<DynamicFile> {
    fn into_response(self) -> EmbedResponse<WebEmbedableFile<DynamicFile>> {
        self.into()
    }
}

pub struct WebEmbedableFile<T: EmbedableFile>(T);

impl<T: EmbedableFile> EmbedRespondable for WebEmbedableFile<T>
where
    T::Data: MessageBody,
{
    type Data = T::Data;
    type DataGzip = T::Data;
    type ETag = T::Meta;
    type LastModified = T::Meta;
    type MimeType = T::Meta;

    fn data(&self) -> Self::Data {
        self.0.data()
    }

    fn data_gzip(&self) -> Option<Self::DataGzip> {
        self.0.data_gzip()
    }

    fn last_modified(&self) -> Option<Self::LastModified> {
        self.0.last_modified()
    }

    fn last_modified_timestamp(&self) -> Option<i64> {
        self.0.last_modified_timestamp()
    }

    fn etag(&self) -> Self::ETag {
        self.0.etag()
    }

    fn mime_type(&self) -> Option<Self::MimeType> {
        self.0.mime_type()
    }
}
