use super::{delegate_login_request, parse_delegate_response, LoginDelegate};
use crate::PushError;
use plist::Dictionary;
use reqwest::StatusCode;

const V2_URL: &str = "https://setup.icloud.com/setup/signin/v2/login";

#[test]
fn remote_profile_pairs_the_legacy_endpoint_and_body() {
    let (url, body) = delegate_login_request(true, V2_URL, "synthetic@example.invalid", "synthetic-pet", &[LoginDelegate::MobileMe]).unwrap();
    assert_eq!(url, "https://setup.icloud.com/setup/iosbuddy/loginDelegates");
    let body: Dictionary = plist::from_bytes(body.as_bytes()).unwrap();
    assert_eq!(body["apple-id"].as_string(), Some("synthetic@example.invalid"));
    assert_eq!(body["password"].as_string(), Some("synthetic-pet"));
    assert!(body["delegates"].as_dictionary().unwrap().contains_key("com.apple.mobileme"));
    assert!(!body.contains_key("protocolVersion"));
}

#[test]
fn native_profile_retains_the_v2_endpoint_and_body() {
    let (url, body) = delegate_login_request(false, V2_URL, "synthetic@example.invalid", "synthetic-pet", &[LoginDelegate::IDS, LoginDelegate::MobileMe]).unwrap();
    assert_eq!(url, V2_URL);
    let body: Dictionary = plist::from_bytes(body.as_bytes()).unwrap();
    assert_eq!(body["protocolVersion"].as_string(), Some("1.0"));
    assert!(body.contains_key("userInfo"));
    assert!(!body.contains_key("apple-id"));
    assert!(!body.contains_key("password"));
    assert_eq!(body["delegates"].as_dictionary().unwrap().len(), 2);
}

#[test]
fn failed_or_empty_responses_have_safe_diagnostics() {
    assert!(matches!(parse_delegate_response(StatusCode::UNAUTHORIZED, b"private data"), Err(PushError::StatusError(StatusCode::UNAUTHORIZED))));
    for body in [b"".as_slice(), b"<html>private data</html>".as_slice()] {
        let error = parse_delegate_response(StatusCode::OK, body).unwrap_err();
        assert!(matches!(error, PushError::InvalidDelegateResponse { status: 200, .. }));
        assert!(!error.to_string().contains("private data"));
    }
}
