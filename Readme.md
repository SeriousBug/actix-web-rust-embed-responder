# [Rust Embed Responder for Actix Web](https://github.com/SeriousBug/actix-web-rust-embed-responder)

[![Crates.io](https://img.shields.io/crates/v/actix-web-rust-embed-responder)](https://crates.io/crates/actix-web-rust-embed-responder) [![docs.rs](https://img.shields.io/docsrs/actix-web-rust-embed-responder)]() [![tests](https://img.shields.io/github/workflow/status/SeriousBug/actix-web-rust-embed-responder/tests?label=tests)](https://github.com/SeriousBug/actix-web-rust-embed-responder/actions/workflows/test.yml) [![Test coverage report](https://img.shields.io/codecov/c/github/SeriousBug/actix-web-rust-embed-responder)](https://codecov.io/gh/SeriousBug/actix-web-rust-embed-responder) [![lint checks](https://img.shields.io/github/workflow/status/SeriousBug/actix-web-rust-embed-responder/lint%20checks?label=lint)](https://github.com/SeriousBug/actix-web-rust-embed-responder/actions/workflows/lint.yml) [![MIT license](https://img.shields.io/github/license/SeriousBug/actix-web-rust-embed-responder)](https://github.com/SeriousBug/actix-web-rust-embed-responder/blob/main/LICENSE.txt)

An Actix Web responder for serving files embedded into the server.
You can embed files into your server, and then use this responder to serve them out of your server.
For example you can have a web app serve its own assets, html, css, javascript files, and more.

This crate implements responders for [rust embed](https://docs.rs/rust-embed/latest/rust_embed/index.html),
as well as a more efficient fork [rust-embed-for-web](https://github.com/SeriousBug/rust-embed-for-web).

## Usage

First, add this crate and `rust-embed` or `rust-embed-for-web` into your `Cargo.toml`.

```toml
[dependencies]
actix-web = "4.2"
rust-embed = "6.4" # or rust-embed-for-web = "11.1"
actix-web-rust-embed-responder = "2.1.0"
```

Then, setup your embed and handler, and add your responder.

```rs
use actix_web::{route, web, App, HttpServer};
use actix_web_rust_embed_responder::{EmbedResponse, IntoResponse};
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
    // There are implementations of `.into_response()` for both `EmbeddedFile` and `Option<EmbeddedFile>`.
    // With `Option<EmbeddedFile>`, this responder will also handle sending a 404 response for `None`.
    // If you want to customize the `404` response, you can handle the `None` case yourself: see the
    // `custom-404.rs` test for an example.
    Embed::get(path).into_response().
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
You can disable the pre-compression which will minimize the increase (see `rust-embed-for-web` readme for details).
An additional drawback is that you will have to recompile to update files even during development.

In exchange for these limitations, you get massively improved performance.
Based on some benchmarks, using the fork is more than 20% faster, with more improvement on larger files or when enabling compression.
For more detailed information check the [benchmark reports](https://seriousbug.github.io/actix-web-rust-embed-responder/reports/).

## Compression

With `rust-embed-for-web`, this crate will serve compressed responses to clients
that support them if compression is enabled for the embed (you didn't add
`#[gzip = "false"]`) and the file being served actually benefits from compression.

With `rust-embed`, compressed responses are not served by default. However you
can set `.use_compression(Compress::Always)` to turn it on. If you do, the files
will be compressed on the fly and cached. This will always compress files, even
for files like image files that are unlikely to benefit from compression.

```rs
Embed::get(path).into_response().use_compression(Compress::Always)
```

For `rust-embed-for-web`, if you disabled pre-compression with `#[gzip = false]` and `#[br = false]`,
you can also enable on-the-fly compression with `Compress::Always`.
Alternatively, you can use `Compress::IfWellKnown` which will only compress files
known to be compressible such as html, css, and javascript.
You can also disable compression entirely with `Compress::Never`.

## Customizing responses

Actix-web has a built-in response customization feature you can use.

```rs
#[route("/{path:.*}", method = "GET", method = "HEAD")]
async fn handler(
    path: web::Path<String>,
) -> CustomizeResponder<EmbedResponse<EmbeddedFile>> {
    EmbedRE::get(path)
        .into_response()
        .customize()
        .insert_header(("X-My-Header", "My Header Value"))
}
```

## Examples

There are examples for both `rust-embed` and `rust-embed-for-web` in the [examples folder](https://github.com/SeriousBug/actix-web-rust-embed-responder/tree/main/examples).
You can run these examples by using `cargo run --example rust_embed --release` or `cargo run --example rust_embed_for_web --release`, then visiting `localhost:8080` in your browser.

## Features

By default, this crate enables support for both `rust-embed` and
`rust-embed-for-web`. You can disable support for the one you're not using:

```toml
# If you are using `rust-embed`:
actix-web-rust-embed-responder = { version = "2.1.0", default-features = false, features = ["support-rust-embed"] }
# If you are using `rust-embed-for-web`:
actix-web-rust-embed-responder = { version = "2.1.0", default-features = false, features = ["support-rust-embed-for-web"] }
```

There's also a feature flag `always-embed` which is disabled by default. This is only useful for testing, you can ignore this feature.

## Compared to `actix-plus-static-files`

Compared to [actix-plus-static-files](https://crates.io/crates/actix-plus-static-files):

- This crate handles sending `304 Not Modified` responses both with `If-None-Match` and `If-Unmodified-Since` headers, while `actix-plus-static-files` only supports `If-None-Match`.
- This crate supports compression, ahead of time with `rust-embed-for-web` or during transmission with `rust-embed`.
- This crate uses base85 with `rust-embed-for-web` and base64 with `rust-embed` for the `ETag`, which is more space efficient than the hex encoding used by `actix-plus-static-files`.
- This crate is only a responder for the `EmbeddedFile` type that you can add to your handlers, while `actix-plus-static-files` implements a service you can directly add into your app.
- `actix-plus-for-web` implements `If-Any-Match` conditional requests, this crate does not. These are not usually used for `GET` and `HEAD` requests.
