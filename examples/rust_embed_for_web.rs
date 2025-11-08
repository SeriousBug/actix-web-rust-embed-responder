use actix_web::{route, web, App, HttpServer};
use actix_web_rust_embed_responder::{EmbedResponse, EmbedableFileResponse, IntoResponse};
use rust_embed_for_web::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
#[cfg_attr(feature = "compression-zstd", zstd = "true")]
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
    println!("Starting server at http://127.0.0.1:8080");
    #[cfg(feature = "compression-zstd")]
    println!("Zstd compression: ENABLED");
    #[cfg(not(feature = "compression-zstd"))]
    println!("Zstd compression: DISABLED");

    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
