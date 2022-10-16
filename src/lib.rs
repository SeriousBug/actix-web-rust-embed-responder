mod helper;
mod parse;

#[cfg(feature = "support-rust-embed")]
mod rust_embed;
#[cfg(feature = "support-rust-embed")]
pub use crate::rust_embed::EmbeddedFileResponse;

#[cfg(feature = "support-rust-embed-for-web")]
mod rust_embed_for_web;
#[cfg(feature = "support-rust-embed-for-web")]
pub use crate::rust_embed_for_web::EmbeddedForWebFileResponse;
