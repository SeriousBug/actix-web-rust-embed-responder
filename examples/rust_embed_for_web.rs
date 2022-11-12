use actix_web::{route, web, App, HttpServer};
use actix_web_rust_embed_responder::{EmbedResponse, EmbedableFileResponse, IntoResponse};
use rust_embed_for_web::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Embed;

#[route("/{path:.*}", method = "GET", method = "HEAD")]
async fn greet(path: web::Path<String>) -> EmbedResponse<EmbedableFileResponse> {
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    Embed::get(path).into_response()
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
