use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    // This is a placeholder test that will be expanded later
    // For now, we'll just test that we can create a basic app structure
    let app = Router::new();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // This will return 404 for now, which is expected
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_auth_compat_response_formats() {
    // Test that our auth compatibility endpoints return the correct NestJS format
    // This test verifies the response structure without requiring a full app setup
    
    use app::api::auth_compat::{
        NestJSLoginResponse, NestJSErrorResponse, NestJSRegisterResponse, 
        NestJSVerifyResponse, NestJSRecoverResponse, NestJSAuthenticateResponse
    };
    use app::middleware::auth::UserContext;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Test login response format
    let user = UserContext {
        user_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        given_name: Some("Test".to_string()),
        family_name: Some("User".to_string()),
        roles: vec!["user".to_string()],
        groups: vec![],
        resource_access: HashMap::new(),
    };

    let login_response = NestJSLoginResponse {
        user: user.clone(),
        authenticated: true,
        accessToken: "test-access-token".to_string(),
        refreshToken: "test-refresh-token".to_string(),
    };

    // Serialize to JSON and verify structure
    let json_value = serde_json::to_value(&login_response).unwrap();
    assert!(json_value.get("user").is_some());
    assert!(json_value.get("authenticated").is_some());
    assert!(json_value.get("accessToken").is_some());
    assert!(json_value.get("refreshToken").is_some());
    assert_eq!(json_value["authenticated"], true);

    // Test error response format
    let error_response = NestJSErrorResponse {
        statusCode: 401,
        message: "Authentication failed".to_string(),
        error: "Unauthorized".to_string(),
    };

    let error_json = serde_json::to_value(&error_response).unwrap();
    assert_eq!(error_json["statusCode"], 401);
    assert_eq!(error_json["error"], "Unauthorized");
    assert_eq!(error_json["message"], "Authentication failed");

    // Test register response format
    let register_response = NestJSRegisterResponse {
        userId: "test-user-id".to_string(),
        message: "User registered successfully".to_string(),
        verificationRequired: true,
    };

    let register_json = serde_json::to_value(&register_response).unwrap();
    assert!(register_json.get("userId").is_some());
    assert!(register_json.get("verificationRequired").is_some());
    assert_eq!(register_json["verificationRequired"], true);

    // Test verify response format
    let verify_response = NestJSVerifyResponse {
        verified: true,
        message: "Email verified successfully".to_string(),
    };

    let verify_json = serde_json::to_value(&verify_response).unwrap();
    assert_eq!(verify_json["verified"], true);
    assert!(verify_json.get("message").is_some());

    // Test recover response format
    let recover_response = NestJSRecoverResponse {
        message: "Reset token sent".to_string(),
        resetTokenSent: true,
    };

    let recover_json = serde_json::to_value(&recover_response).unwrap();
    assert!(recover_json.get("resetTokenSent").is_some());
    assert_eq!(recover_json["resetTokenSent"], true);

    // Test authenticate response format
    let auth_response = NestJSAuthenticateResponse {
        authenticated: true,
        user: Some(user),
    };

    let auth_json = serde_json::to_value(&auth_response).unwrap();
    assert_eq!(auth_json["authenticated"], true);
    assert!(auth_json.get("user").is_some());
}

#[tokio::test]
async fn test_cookie_extraction_compatibility() {
    // Test that cookie extraction works as expected for NestJS clients
    use app::middleware::auth_compat::{AuthCompatLayer, Credentials};
    use axum::http::{HeaderMap, HeaderValue};

    fn create_test_request_with_headers(headers: HeaderMap) -> Request<Body> {
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        *request.headers_mut() = headers;
        request
    }

    // Test 1: Cookie takes priority over bearer token
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_static("Authentication=cookie-token; RefreshToken=refresh-token"),
    );
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer bearer-token"),
    );
    
    let request = create_test_request_with_headers(headers);
    let credentials = AuthCompatLayer::extract_credentials(&request).unwrap();
    
    match credentials {
        Credentials::Cookie(token) => assert_eq!(token, "cookie-token"),
        Credentials::Bearer(_) => panic!("Should have extracted cookie token"),
    }

    // Test 2: Falls back to bearer token when no cookies
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer bearer-token"),
    );
    
    let request = create_test_request_with_headers(headers);
    let credentials = AuthCompatLayer::extract_credentials(&request).unwrap();
    
    match credentials {
        Credentials::Bearer(token) => assert_eq!(token, "bearer-token"),
        Credentials::Cookie(_) => panic!("Should have extracted bearer token"),
    }

    // Test 3: Handles malformed cookies gracefully
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_static("malformed_cookie_without_equals"),
    );
    
    let request = create_test_request_with_headers(headers);
    let result = AuthCompatLayer::extract_credentials(&request);
    assert!(result.is_err()); // Should fail gracefully

    // Test 4: Handles empty authentication header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static(""),
    );
    
    let request = create_test_request_with_headers(headers);
    let result = AuthCompatLayer::extract_credentials(&request);
    assert!(result.is_err()); // Should fail when no valid auth found
}