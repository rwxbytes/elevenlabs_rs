use bytes::buf::Buf;
use elevenlabs::api::models::build_models_client;
//use http_body::Body;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Bytes};

#[test]
fn client_builders_new_errs_when_env_var_is_not_set() {
    todo!("Test that ClientBuilder::new() errors when ELEVEN_API_KEY is not set");
    // test rest of invalid calls to ClientBuilder::new()
}

#[test]
fn build_models_client_is_returning_a_client_with_models_endpoint_config() {
    let c = build_models_client()
        .expect("build_models_client is returning a client with models config");
    let want = "https://api.elevenlabs.io/v1/models".to_string();
    let parts = c.url.into_parts();
    let got = format!(
        "{}://{}{}",
        parts
            .scheme
            .expect("scheme() is getting the scheme from url"),
        parts
            .authority
            .expect("authority() is getting the authority from url"),
        parts
            .path_and_query
            .expect("path_and_query() is getting the path and query from url")
    );

    assert_eq!(want, got);

    let want = "GET";
    let got = c.method.as_str();
    assert_eq!(want, got);

    assert_eq!(c.headers.get("ACCEPT").unwrap(), "application/json");
}

// Learn how to test this
//#[test]
//fn clients_build_request_is_returning_a_request_for_models_get_endpoint() {
//    let c = build_models_client()
//        .expect("build_models_client is returning a client with models config");
//    let req = c
//        .build_request(Empty::<Bytes>::new())
//        .expect("build_request is returning a request");
//    let b = req.body();
//}

#[test]
fn clients_format_address_is_returning_an_okay_string_with_host_and_port_set_to_443() {
    let c = build_models_client()
        .expect("build_models_client is returning a client with models config");
    let want = "api.elevenlabs.io:443".to_string();
    let got = c.format_address();
    assert_eq!(want, got);
}
