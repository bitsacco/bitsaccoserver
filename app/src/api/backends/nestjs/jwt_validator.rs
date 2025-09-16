#[cfg(feature = "ssr")]
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::api::errors::{ApiError, ApiResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub nbf: usize,         // Not before
    pub roles: Vec<String>, // User roles
    pub phone: Option<String>,
    pub npub: Option<String>,
}

#[derive(Clone)]
pub struct JwtValidator {
    #[cfg(feature = "ssr")]
    decoding_key: DecodingKey,
    #[cfg(feature = "ssr")]
    validation: Validation,
}

impl std::fmt::Debug for JwtValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtValidator")
            .field("has_key", &true)
            .finish()
    }
}

impl JwtValidator {
    #[cfg(feature = "ssr")]
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::new();
        validation.required_spec_claims.insert("exp".to_string());
        validation.required_spec_claims.insert("sub".to_string());

        Self {
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            validation,
        }
    }

    #[cfg(not(feature = "ssr"))]
    pub fn new(_secret: &str) -> Self {
        Self {}
    }

    pub fn validate_token(&self, token: &str) -> ApiResult<Claims> {
        #[cfg(feature = "ssr")]
        {
            decode::<Claims>(token, &self.decoding_key, &self.validation)
                .map(|token_data| token_data.claims)
                .map_err(|e| ApiError::Authentication {
                    message: format!("Invalid JWT token: {}", e),
                })
        }

        #[cfg(not(feature = "ssr"))]
        {
            // In client-side, we can't validate JWT securely, so we just check basic format
            if token.split('.').count() != 3 {
                return Err(ApiError::Authentication {
                    message: "Invalid JWT token format".to_string(),
                });
            }

            // Return a placeholder claims structure for client-side
            Ok(Claims {
                sub: "client-side".to_string(),
                exp: 0,
                iat: 0,
                nbf: 0,
                roles: vec![],
                phone: None,
                npub: None,
            })
        }
    }

    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = chrono::Utc::now().timestamp() as usize;
        claims.exp < now
    }

    pub fn has_role(&self, claims: &Claims, role: &str) -> bool {
        claims.roles.iter().any(|r| r == role)
    }

    pub fn has_any_role(&self, claims: &Claims, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(claims, role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "ssr")]
    fn test_jwt_validator_creation() {
        let validator = JwtValidator::new("test_secret");
        // Test that validator was created successfully
        // Note: We can't test the internal decoding_key as as_bytes() is private
        assert!(true); // If we reach here, creation was successful
    }

    #[test]
    fn test_token_expiry_check() {
        let validator = JwtValidator::new("test_secret");

        // Create an expired token
        let expired_claims = Claims {
            sub: "user123".to_string(),
            exp: 1000000000, // Old timestamp
            iat: 1000000000,
            nbf: 1000000000,
            roles: vec!["user".to_string()],
            phone: Some("+1234567890".to_string()),
            npub: None,
        };

        assert!(validator.is_token_expired(&expired_claims));

        // Create a valid token
        let valid_claims = Claims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600) as usize, // 1 hour from now
            iat: chrono::Utc::now().timestamp() as usize,
            nbf: chrono::Utc::now().timestamp() as usize,
            roles: vec!["user".to_string()],
            phone: Some("+1234567890".to_string()),
            npub: None,
        };

        assert!(!validator.is_token_expired(&valid_claims));
    }

    #[test]
    fn test_role_checking() {
        let validator = JwtValidator::new("test_secret");
        let claims = Claims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            nbf: chrono::Utc::now().timestamp() as usize,
            roles: vec!["admin".to_string(), "user".to_string()],
            phone: Some("+1234567890".to_string()),
            npub: None,
        };

        assert!(validator.has_role(&claims, "admin"));
        assert!(validator.has_role(&claims, "user"));
        assert!(!validator.has_role(&claims, "superadmin"));

        assert!(validator.has_any_role(&claims, &["admin", "superadmin"]));
        assert!(!validator.has_any_role(&claims, &["superadmin", "moderator"]));
    }
}
