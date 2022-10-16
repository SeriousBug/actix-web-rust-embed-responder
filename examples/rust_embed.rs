use actix_web::{get, web, App, HttpServer};
use actix_web_rust_embed_responder::EmbeddedFileResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Embed;

#[get("/{path:.*}")]
async fn greet(params: web::Path<String>) -> EmbeddedFileResponse {
    let path = if params.is_empty() {
        "index.html"
    } else {
        params.as_str()
    };
    let f = Embed::get(path);
    let embed: EmbeddedFileResponse = f.into();
    embed
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
