use std::time::Duration;

use actix_web::{dev::ServiceResponse, get, test, App};
use actix_web_rust_embed_responder::{EmbeddedFileResponse, EmbeddedForWebFileResponse};
use criterion::{criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use tokio::runtime::{self, Runtime};

#[derive(rust_embed::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedRE;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedREFW;

#[get("/")]
async fn re_handler() -> EmbeddedFileResponse {
    EmbedRE::get("index.html").unwrap().into()
}

#[get("/")]
async fn refw_handler() -> EmbeddedForWebFileResponse {
    EmbedREFW::get("index.html").unwrap().into()
}

fn prep_service(
    runtime: &Runtime,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = ServiceResponse,
    Error = actix_web::Error,
> {
    runtime.block_on(actix_web::test::init_service(
        App::new().service(refw_handler),
    ))
}

async fn test_re(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let req = test::TestRequest::get().to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(resp.starts_with("<!DOCTYPE html>".as_bytes()))
}

async fn test_refw(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let req = test::TestRequest::get().to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(resp.starts_with("<!DOCTYPE html>".as_bytes()))
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Basic homepages");
    group.measurement_time(Duration::from_secs(30));

    let runtime = runtime::Builder::new_current_thread().build().unwrap();
    let app = prep_service(&runtime);

    group.bench_with_input("rust_embed", &app, |b, app| {
        b.to_async(&runtime).iter(|| test_re(app))
    });

    group.bench_with_input("rust_embed_for_web", &app, |b, app| {
        b.to_async(&runtime).iter(|| test_refw(app))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
