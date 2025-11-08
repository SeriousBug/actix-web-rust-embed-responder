use actix_http::body::MessageBody;
use actix_web::test;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    route, web, App,
};
use actix_web_rust_embed_responder::{
    Compress, EmbedResponse, EmbedableFileResponse, IntoResponse,
};

#[derive(rust_embed::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedRE;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedREFW;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
#[gzip = false]
struct EmbedREFWNoGzip;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
#[zstd = false]
struct EmbedREFWNoZstd;

#[route("/re/{compress}/{path:.*}", method = "GET", method = "HEAD")]
async fn re_handler(
    params: web::Path<(String, String)>,
) -> EmbedResponse<rust_embed::EmbeddedFile> {
    let (compress, path) = params.into_inner();
    println!("{} - {}", compress, path);
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    let compress = match compress.as_str() {
        "always" => Compress::Always,
        "ifprecompressed" => Compress::IfPrecompressed,
        "ifwellknown" => Compress::IfWellKnown,
        "never" => Compress::Never,
        _ => panic!("Unknown compression level!"),
    };
    EmbedRE::get(path).into_response().use_compression(compress)
}

#[route("/refw/{compress}/{path:.*}", method = "GET", method = "HEAD")]
async fn refw_handler(params: web::Path<(String, String)>) -> EmbedResponse<EmbedableFileResponse> {
    let (compress, path) = params.into_inner();
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    let compress = match compress.as_str() {
        "always" => Compress::Always,
        "ifprecompressed" => Compress::IfPrecompressed,
        "ifwellknown" => Compress::IfWellKnown,
        "never" => Compress::Never,
        _ => panic!("Unknown compression level!"),
    };
    EmbedREFW::get(path)
        .into_response()
        .use_compression(compress)
}

#[route("/refw-nogz/{compress}/{path:.*}", method = "GET", method = "HEAD")]
async fn refw_nogz_handler(
    params: web::Path<(String, String)>,
) -> EmbedResponse<EmbedableFileResponse> {
    let (compress, path) = params.into_inner();
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    let compress = match compress.as_str() {
        "always" => Compress::Always,
        "ifprecompressed" => Compress::IfPrecompressed,
        "ifwellknown" => Compress::IfWellKnown,
        "never" => Compress::Never,
        _ => panic!("Unknown compression level!"),
    };
    EmbedREFWNoGzip::get(path)
        .into_response()
        .use_compression(compress)
}

#[route("/refw-nozstd/{compress}/{path:.*}", method = "GET", method = "HEAD")]
async fn refw_nozstd_handler(
    params: web::Path<(String, String)>,
) -> EmbedResponse<EmbedableFileResponse> {
    let (compress, path) = params.into_inner();
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    let compress = match compress.as_str() {
        "always" => Compress::Always,
        "ifprecompressed" => Compress::IfPrecompressed,
        "ifwellknown" => Compress::IfWellKnown,
        "never" => Compress::Never,
        _ => panic!("Unknown compression level!"),
    };
    EmbedREFWNoZstd::get(path)
        .into_response()
        .use_compression(compress)
}

async fn make_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .service(refw_handler)
        .service(re_handler)
        .service(refw_nogz_handler)
        .service(refw_nozstd_handler)
}

#[actix_web::test]
async fn always_compress_always_turns_on_compression() {
    let app = test::init_service(make_app().await).await;

    let req = test::TestRequest::get()
        .uri("/re/always/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );

    let req = test::TestRequest::get()
        .uri("/refw/always/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/always/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );
}

#[actix_web::test]
async fn never_compress_always_turns_off_compression() {
    let app = test::init_service(make_app().await).await;

    let req = test::TestRequest::get()
        .uri("/re/never/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());

    let req = test::TestRequest::get()
        .uri("/refw/never/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());

    let req = test::TestRequest::get()
        .uri("/refw-nogz/never/")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());
}

#[actix_web::test]
async fn if_well_known_compresses_html() {
    let app = test::init_service(make_app().await).await;

    // We don't test RE here because it doesn't have mime types :/
    // It should be added if RE adds mime-types to the built-in metadata.

    let req = test::TestRequest::get()
        .uri("/refw/ifwellknown/index.html")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/ifwellknown/index.html")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );
}

#[actix_web::test]
async fn if_pre_compressed_works_as_advertised() {
    let app = test::init_service(make_app().await).await;

    // We don't test RE here because it doesn't support pre-compression anyway,
    // so it would be equivalent to Never.

    let req = test::TestRequest::get()
        .uri("/refw/ifprecompressed/index.html")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "gzip"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/ifprecompressed/index.html")
        .append_header(("Accept-Encoding", "gzip"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.response().headers().get("Content-Encoding").is_none());
}

#[actix_web::test]
async fn always_compress_uses_zstd_when_preferred() {
    let app = test::init_service(make_app().await).await;

    // Test that zstd is preferred over gzip and brotli when all are accepted
    let req = test::TestRequest::get()
        .uri("/re/always/")
        .append_header(("Accept-Encoding", "gzip, br, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    let req = test::TestRequest::get()
        .uri("/refw/always/")
        .append_header(("Accept-Encoding", "gzip, br, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/always/")
        .append_header(("Accept-Encoding", "gzip, br, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );
}

#[actix_web::test]
async fn always_compress_uses_zstd_when_only_zstd_accepted() {
    let app = test::init_service(make_app().await).await;

    let req = test::TestRequest::get()
        .uri("/re/always/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    let req = test::TestRequest::get()
        .uri("/refw/always/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/always/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );
}

#[actix_web::test]
async fn never_compress_disables_zstd() {
    let app = test::init_service(make_app().await).await;

    let req = test::TestRequest::get()
        .uri("/re/never/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());

    let req = test::TestRequest::get()
        .uri("/refw/never/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());

    let req = test::TestRequest::get()
        .uri("/refw-nogz/never/")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().get("Content-Encoding").is_none());
}

#[actix_web::test]
async fn if_well_known_compresses_html_with_zstd() {
    let app = test::init_service(make_app().await).await;

    // We don't test RE here because it doesn't have mime types :/
    // It should be added if RE adds mime-types to the built-in metadata.

    let req = test::TestRequest::get()
        .uri("/refw/ifwellknown/index.html")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    let req = test::TestRequest::get()
        .uri("/refw-nogz/ifwellknown/index.html")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );
}

#[actix_web::test]
async fn if_pre_compressed_works_with_zstd() {
    let app = test::init_service(make_app().await).await;

    // We don't test RE here because it doesn't support pre-compression anyway,
    // so it would be equivalent to Never.

    let req = test::TestRequest::get()
        .uri("/refw/ifprecompressed/index.html")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    // Test with embed that has zstd disabled - should get no compression
    let req = test::TestRequest::get()
        .uri("/refw-nozstd/ifprecompressed/index.html")
        .append_header(("Accept-Encoding", "zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.response().headers().get("Content-Encoding").is_none());
}

#[actix_web::test]
async fn zstd_priority_over_brotli_and_gzip() {
    let app = test::init_service(make_app().await).await;

    // When client accepts all three encodings, zstd should be preferred
    let req = test::TestRequest::get()
        .uri("/refw/always/index.html")
        .append_header(("Accept-Encoding", "gzip, br, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    // When client accepts zstd and brotli, zstd should be preferred
    let req = test::TestRequest::get()
        .uri("/refw/always/index.html")
        .append_header(("Accept-Encoding", "br, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );

    // When client accepts zstd and gzip, zstd should be preferred
    let req = test::TestRequest::get()
        .uri("/refw/always/index.html")
        .append_header(("Accept-Encoding", "gzip, zstd"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.response()
            .headers()
            .get("Content-Encoding")
            .expect("No encoding header"),
        "zstd"
    );
}
