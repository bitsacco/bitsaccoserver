use crate::contexts::auth::use_auth_token;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

pub struct ApiClient;

impl ApiClient {
    pub fn get(url: &str) -> RequestBuilder {
        RequestBuilder::new("GET", url)
    }

    pub fn post(url: &str) -> RequestBuilder {
        RequestBuilder::new("POST", url)
    }

    pub fn put(url: &str) -> RequestBuilder {
        RequestBuilder::new("PUT", url)
    }

    pub fn patch(url: &str) -> RequestBuilder {
        RequestBuilder::new("PATCH", url)
    }

    pub fn delete(url: &str) -> RequestBuilder {
        RequestBuilder::new("DELETE", url)
    }
}

pub struct RequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    with_auth: bool,
}

impl RequestBuilder {
    fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: None,
            with_auth: true,
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_string(data)?);
        Ok(self)
    }

    pub fn without_auth(mut self) -> Self {
        self.with_auth = false;
        self
    }

    pub async fn send<T: for<'de> Deserialize<'de>>(self) -> ApiResult<T> {
        let mut request = match self.method.as_str() {
            "GET" => Request::get(&self.url),
            "POST" => Request::post(&self.url),
            "PUT" => Request::put(&self.url),
            "PATCH" => Request::patch(&self.url),
            "DELETE" => Request::delete(&self.url),
            _ => {
                return Err(ApiError {
                    message: "Unsupported HTTP method".to_string(),
                    code: Some("INVALID_METHOD".to_string()),
                })
            }
        };

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // Add authentication header if required
        if self.with_auth {
            if let Some(token) = use_auth_token() {
                request = request.header("Authorization", &format!("Bearer {}", token));
            }
        }

        // Add body if present and build request
        let request = if let Some(body) = &self.body {
            request.body(body.clone()).map_err(|e| ApiError {
                message: format!("Failed to set request body: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
            })?
        } else {
            request.build().map_err(|e| ApiError {
                message: format!("Failed to build request: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
            })?
        };

        // Send request
        let response = request.send().await.map_err(|e| ApiError {
            message: format!("Network error: {}", e),
            code: Some("NETWORK_ERROR".to_string()),
        })?;

        // Handle response
        if response.ok() {
            response.json::<T>().await.map_err(|e| ApiError {
                message: format!("JSON parsing error: {}", e),
                code: Some("PARSE_ERROR".to_string()),
            })
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(ApiError {
                message: format!("HTTP {}: {}", status, error_text),
                code: Some(format!("HTTP_{}", status)),
            })
        }
    }

    pub async fn send_empty(self) -> ApiResult<()> {
        let mut request = match self.method.as_str() {
            "GET" => Request::get(&self.url),
            "POST" => Request::post(&self.url),
            "PUT" => Request::put(&self.url),
            "PATCH" => Request::patch(&self.url),
            "DELETE" => Request::delete(&self.url),
            _ => {
                return Err(ApiError {
                    message: "Unsupported HTTP method".to_string(),
                    code: Some("INVALID_METHOD".to_string()),
                })
            }
        };

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // Add authentication header if required
        if self.with_auth {
            if let Some(token) = use_auth_token() {
                request = request.header("Authorization", &format!("Bearer {}", token));
            }
        }

        // Add body if present and build request
        let request = if let Some(body) = &self.body {
            request.body(body.clone()).map_err(|e| ApiError {
                message: format!("Failed to set request body: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
            })?
        } else {
            request.build().map_err(|e| ApiError {
                message: format!("Failed to build request: {}", e),
                code: Some("REQUEST_BUILD_ERROR".to_string()),
            })?
        };

        // Send request
        let response = request.send().await.map_err(|e| ApiError {
            message: format!("Network error: {}", e),
            code: Some("NETWORK_ERROR".to_string()),
        })?;

        // Handle response
        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(ApiError {
                message: format!("HTTP {}: {}", status, error_text),
                code: Some(format!("HTTP_{}", status)),
            })
        }
    }
}
