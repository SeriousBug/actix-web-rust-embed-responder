use actix_web::{dev::ServiceResponse, route, web, App};
use actix_web_rust_embed_responder::{EmbedResponse, EmbedableFileResponse};
use tokio::runtime::Runtime;

#[derive(rust_embed::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedRE;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedREFW;

#[route("/re/{path:.*}", method = "GET", method = "HEAD")]
async fn re_handler(path: web::Path<String>) -> EmbedResponse<rust_embed::EmbeddedFile> {
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    EmbedRE::get(path).into()
}

#[route("/refw/{path:.*}", method = "GET", method = "HEAD")]
async fn refw_handler(path: web::Path<String>) -> EmbedResponse<EmbedableFileResponse> {
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    EmbedREFW::get(path).into()
}

pub fn prep_service(
    runtime: &Runtime,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = ServiceResponse,
    Error = actix_web::Error,
> {
    runtime.block_on(actix_web::test::init_service(
        App::new().service(refw_handler).service(re_handler),
    ))
}

// These aren't actually dead, but it looks like rust can't tell that.
#[allow(dead_code)]
pub static ETAG_RE: &'static str = r#""0HmEESpoRuXjI9o47wPpRmueMqePF3leJjWufwSYkNs""#;
#[allow(dead_code)]
pub static ETAG_REFW: &'static str = r#""(0POrDriRK<0INQ?*r*ZYo0Qvj~97fCN-{q1elQ9""#;
#[allow(dead_code)]
/// The number of seconds to run each benchmark for.
pub static SECS_PER_BENCH: u64 = 60;
