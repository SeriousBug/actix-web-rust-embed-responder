use actix_web::{dev::ServiceResponse, get, App};
use actix_web_rust_embed_responder::{EmbeddedFileResponse, EmbeddedForWebFileResponse};
use tokio::runtime::Runtime;

#[derive(rust_embed::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedRE;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedREFW;

#[get("/re")]
async fn re_handler() -> EmbeddedFileResponse {
    EmbedRE::get("index.html").unwrap().into()
}

#[get("/refw")]
async fn refw_handler() -> EmbeddedForWebFileResponse {
    EmbedREFW::get("index.html").unwrap().into()
}

#[get("/re/image")]
async fn re_image_handler() -> EmbeddedFileResponse {
    EmbedRE::get("pexels-david-yu-10075042.jpg").unwrap().into()
}

#[get("/refw/image")]
async fn refw_image_handler() -> EmbeddedForWebFileResponse {
    EmbedREFW::get("pexels-david-yu-10075042.jpg")
        .unwrap()
        .into()
}

pub fn prep_service(
    runtime: &Runtime,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = ServiceResponse,
    Error = actix_web::Error,
> {
    runtime.block_on(actix_web::test::init_service(
        App::new()
            .service(refw_handler)
            .service(re_handler)
            .service(re_image_handler)
            .service(refw_image_handler),
    ))
}

// These aren't actually dead, but it looks like rust can't tell that.
#[allow(dead_code)]
pub static ETAG_RE: &'static str = r#""0HmEESpoRuXjI9o47wPpRmueMqePF3leJjWufwSYkNs=""#;
#[allow(dead_code)]
pub static ETAG_REFW: &'static str = r#""(0POrDriRK<0INQ?*r*ZYo0Qvj~97fCN-{q1elQ9""#;
#[allow(dead_code)]
/// The number of seconds to run each benchmark for.
pub static SECS_PER_BENCH: u64 = 60;
