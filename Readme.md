# Rust Embed Responder for Actix Web

An Actix Web responder for serving files embedded into the server.
You can embed files into your server, and then use this responder to serve them out of your server.
For example, you can have a server serve its own assets, html, css, and javascript files, and more.

This crate implements responders for [rust embed](https://docs.rs/rust-embed/latest/rust_embed/index.html),
as well as a more efficient fork [rust-embed-for-web](https://github.com/SeriousBug/rust-embed-for-web).

## Usage

First, add this crate and `rust-embed` or `rust-embed-for-web` into your `Cargo.toml`.

```toml
[dependencies]
# These numbers are not regularly updated, check
# crates.io to make sure you have the latest versions
actix-web = "4.2"
rust-embed = "6.4"
actix-web-rust-embed-responder = "0.1"
```

Then, setup your embed and handler, and add your responder.

```rs
use actix_web::{route, web, App, HttpServer};
use actix_web_rust_embed_responder::EmbedResponse;
use rust_embed::{EmbeddedFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "path/to/assets/"]
struct Embed;

// This responder implements both GET and HEAD
#[route("/{path:.*}", method = "GET", method = "HEAD")]
// The return type is important, that is the type for this responder
async fn serve_assets(path: web::Path<String>) -> EmbedResponse<EmbeddedFile> {
    // This is not required, but is likely what you want if you want this
    // to serve `index.html` as the home page.
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    // There are implementations of `.into()` for both `EmbeddedFile` and `Option<EmbeddedFile>`.
    // With `Option<EmbeddedFile>`, this responder will also handle sending a 404 response for `None`.
    // If you want to customize the `404` response, handle the `None` case yourself and use `.into()`
    // on `RustEmbed`.
    Embed::get(path).into()
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(serve_assets))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

## About the `rust-embed-for-web` fork

The fork pre-computes certain things, like the header values that are used in responses.
It also avoids unnecessary memory copies, and stores compressed version ahead of time.
This can significantly increase the size of your compiled binary, but in exchange improves performance significantly.
Additionally, you can disable the ahead-of-time compression which will minimize the increase (see `rust-embed-for-web` readme for details).
An additional drawback is that you will have to recompile to update files even during development.

In exchange for these limitations, you get massively improved performance.
Based on some basic benchmarks, using the fork is 20% to 3400% faster (more improvement with compression and larger files).
For more detailed information check the [benchmark reports](#).

## Compared to `actix-plus-static-files`

Compared to [actix-plus-static-files](https://crates.io/crates/actix-plus-static-files):

- This crate handles sending `304 Not Modified` responses both with `If-None-Match` and `If-Unmodified-Since` headers, while `actix-plus-static-files` only supports `If-None-Match`.
- This crate supports compression, ahead of time with `rust-embed-for-web` or during transmission with `rust-embed`.
- This crate uses base85 with `rust-embed-for-web` and base64 with `rust-embed` for the `ETag`, which is more space efficient than the hex encoding used by `actix-plus-static-files`.
- This crate is only a responder for the `EmbeddedFile` type that you can add to your handlers, while `actix-plus-static-files` implements a service you can directly add into your app.
- `actix-plus-for-web` implements `If-Any-Match` conditional requests, this crate does not. These are not usually used for `GET` and `HEAD` requests.
