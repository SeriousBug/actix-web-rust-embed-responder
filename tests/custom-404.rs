use actix_http::body::MessageBody;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    route, web, App, HttpRequest, Responder,
};
use actix_web::{test, HttpResponse};
use actix_web_rust_embed_responder::IntoResponse;

#[derive(rust_embed::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedRE;

#[derive(rust_embed_for_web::RustEmbed)]
#[folder = "examples/assets/"]
struct EmbedREFW;

#[route("/re/{path:.*}", method = "GET", method = "HEAD")]
async fn re_handler(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    match EmbedRE::get(&path) {
        Some(file) => file.into_response().respond_to(&req),
        None => HttpResponse::NotFound().body("File not found!"),
    }
}

#[route("/refw/{path:.*}", method = "GET", method = "HEAD")]
async fn refw_handler(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    match EmbedREFW::get(&path) {
        Some(file) => file.into_response().respond_to(&req),
        None => HttpResponse::NotFound().body("File not found!"),
    }
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
    App::new().service(refw_handler).service(re_handler)
}

#[actix_web::test]
async fn custom_404_response_works() {
    let app = test::init_service(make_app().await).await;

    let req = test::TestRequest::get()
        .uri("/re/does-not-exist.txt")
        .to_request();
    let resp = test::call_and_read_body(&app, req).await;
    let resp_body = String::from_utf8_lossy(resp.as_ref());
    assert_eq!(resp_body, "File not found!");

    let req = test::TestRequest::get()
        .uri("/refw/does-not-exist.txt")
        .to_request();
    let resp = test::call_and_read_body(&app, req).await;
    let resp_body = String::from_utf8_lossy(resp.as_ref());
    assert_eq!(resp_body, "File not found!");
}
