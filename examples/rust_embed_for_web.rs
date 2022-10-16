use actix_web::{get, web, App, Either, HttpResponse, HttpServer};
use actix_web_rust_embed_responder::EmbeddedForWebFileResponse;
use rust_embed_for_web::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Embed;

#[get("/{path:.*}")]
async fn greet(params: web::Path<String>) -> Either<EmbeddedForWebFileResponse, HttpResponse> {
    println!("{:?}", params.as_str());
    let path = if params.is_empty() {
        "index.html"
    } else {
        params.as_str()
    };
    let f = Embed::get(path);
    if f.is_none() {
        return Either::Right(HttpResponse::NotFound().finish());
    }
    let embed: EmbeddedForWebFileResponse = f.unwrap().into();
    Either::Left(embed)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
