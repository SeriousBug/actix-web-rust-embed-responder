mod helper;
mod parse;

mod compress;
pub use compress::*;
mod embed;

#[cfg(feature = "support-rust-embed")]
mod rust_embed;
#[cfg(feature = "support-rust-embed")]
pub use crate::rust_embed::*;

#[cfg(feature = "support-rust-embed-for-web")]
mod rust_embed_for_web;
#[cfg(feature = "support-rust-embed-for-web")]
pub use crate::rust_embed_for_web::*;

pub use embed::{EmbedRespondable, EmbedResponse, IntoResponse};
