use actix_web::{get, web, App, Either, HttpRequest, HttpResponse, HttpServer};
use actix_web_rust_embed_responder::EmbeddedFileResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Embed;

#[get("/{path:.*}")]
async fn greet(
    req: HttpRequest,
    params: web::Path<String>,
) -> Either<EmbeddedFileResponse, HttpResponse> {
    println!("{:?}", params.as_str());

    if let Some(if_modified_since) = req
        .headers()
        .get("If-Unmodified-Since")
        .and_then(|v| v.to_str().ok())
    {
        println!("header: {:?}", if_modified_since);
        if let Some(if_modified_since) =
            chrono::DateTime::parse_from_rfc2822(if_modified_since).ok()
        {
            println!("date: {:?}", if_modified_since.to_rfc2822());
        }
    }

    let path = if params.is_empty() {
        "index.html"
    } else {
        params.as_str()
    };
    let f = Embed::get(path);
    if f.is_none() {
        return Either::Right(HttpResponse::NotFound().finish());
    }
    let embed: EmbeddedFileResponse = f.unwrap().into();
    Either::Left(embed)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
