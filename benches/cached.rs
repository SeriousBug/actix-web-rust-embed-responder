use std::time::Duration;

use actix_web::{dev::ServiceResponse, test};
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime;

mod common;
use common::{prep_service, ETAG_RE, ETAG_REFW, SECS_PER_BENCH};

async fn test_re(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let req = test::TestRequest::get()
        .append_header(("If-None-Match", ETAG_RE))
        .uri("/re")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
}

async fn test_refw(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let req = test::TestRequest::get()
        .uri("/refw")
        .append_header(("If-None-Match", ETAG_REFW))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached responses only");
    group.measurement_time(Duration::from_secs(SECS_PER_BENCH));

    let runtime = runtime::Builder::new_current_thread().build().unwrap();
    let app = prep_service(&runtime);

    group.bench_with_input("rust_embed", &app, |b, app| {
        b.to_async(&runtime).iter(|| test_re(app))
    });

    group.bench_with_input("rust_embed_for_web", &app, |b, app| {
        b.to_async(&runtime).iter(|| test_refw(app))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
