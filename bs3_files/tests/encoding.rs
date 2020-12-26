use actix_web::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    test::{self, TestRequest},
    App,
};
use bs3_files::Files;

#[actix_rt::test]
async fn test_utf8_file_contents() {
    // use default ISO-8859-1 encoding
    let mut srv = test::init_service(App::new().service(Files::new("/", "./tests"))).await;

    let req = TestRequest::with_uri("/utf8.txt").to_request();
    let res = test::call_service(&mut srv, req).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/plain")),
    );

    // prefer UTF-8 encoding
    let mut srv =
        test::init_service(App::new().service(Files::new("/", "./tests").prefer_utf8(true))).await;

    let req = TestRequest::with_uri("/utf8.txt").to_request();
    let res = test::call_service(&mut srv, req).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/plain; charset=utf-8")),
    );
}
