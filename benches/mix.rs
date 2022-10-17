use std::time::Duration;

use actix_http::Method;
use actix_web::{dev::ServiceResponse, test};
use criterion::{criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use tokio::runtime;

mod common;
use common::{prep_service, ETAG_RE, ETAG_REFW, SECS_PER_BENCH};

lazy_static! {
    static ref NOW: String = chrono::Local::now().to_rfc2822();
}

async fn test_re(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let path = "/re/";
    // Make a regular request
    let req = test::TestRequest::get().uri(path).to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(!resp.is_empty());
    // Make a couple cached requests
    let req = test::TestRequest::get()
        .append_header(("If-None-Match", ETAG_RE))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    let req = test::TestRequest::get()
        .append_header(("If-None-Match", ETAG_RE))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    // Make a compressed body request
    let req = test::TestRequest::get()
        .uri(path)
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(!resp.is_empty());
    // Make a cached request again, but this time use "If-Unmodified-Since"
    let req = test::TestRequest::get()
        .append_header(("If-Unmodified-Since", NOW.as_str()))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    // Try a HEAD request
    let req = test::TestRequest::default()
        .method(Method::HEAD)
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    // Get a not-found response
    let req = test::TestRequest::get()
        .uri(&format!("{path}foo-bar-baz"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

async fn test_refw(
    app: impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let path = "/refw/";
    // Make a regular request
    let req = test::TestRequest::get().uri(path).to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(!resp.is_empty());
    // Make a couple cached requests
    let req = test::TestRequest::get()
        .append_header(("If-None-Match", ETAG_REFW))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    let req = test::TestRequest::get()
        .append_header(("If-None-Match", ETAG_REFW))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    // Make a compressed body request
    let req = test::TestRequest::get()
        .uri(path)
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_and_read_body(&app, req).await;
    assert!(!resp.is_empty());
    // Make a cached request again, but this time use "If-Unmodified-Since"
    let req = test::TestRequest::get()
        .append_header(("If-Unmodified-Since", NOW.as_str()))
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 304);
    // Try a HEAD request
    let req = test::TestRequest::default()
        .method(Method::HEAD)
        .uri(path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    // Get a not-found response
    let req = test::TestRequest::get()
        .uri(&format!("{path}foo-bar-baz"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixture of cached and non-cached responses");
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
