use serde::{Deserialize, Serialize};

use super::common::Role;
use super::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub pin: String,
    pub phone: Option<String>,
    pub npub: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub pin: String,
    pub phone: Option<String>,
    pub npub: Option<String>,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub phone: Option<String>,
    pub npub: Option<String>,
    pub otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverRequest {
    pub pin: String,
    pub phone: Option<String>,
    pub npub: Option<String>,
    pub otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeTokenRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub authenticated: bool,
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeTokenResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: Option<String>,
}

// NestJS-specific types to handle different response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsPhone {
    pub number: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsNostr {
    pub npub: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsProfile {
    pub name: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsUser {
    pub id: String,
    pub phone: NestJsPhone,
    pub nostr: NestJsNostr,
    pub profile: NestJsProfile,
    pub roles: Vec<u8>, // NestJS returns roles as numbers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsAuthResponse {
    pub user: NestJsUser,
    pub authenticated: bool,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
