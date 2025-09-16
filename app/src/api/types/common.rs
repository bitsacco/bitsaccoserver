use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    #[serde(rename = "member")]
    Member = 0,
    #[serde(rename = "admin")]
    Admin = 1,
    #[serde(rename = "super_admin")]
    SuperAdmin = 2,
}

impl Role {
    /// Convert role to string format expected by frontend auth guards
    pub fn to_auth_string(&self) -> String {
        match self {
            Role::Member => "member".to_string(),
            Role::Admin => "admin".to_string(),
            Role::SuperAdmin => "superadmin".to_string(), // Note: no underscore for frontend
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phone {
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nostr {
    pub npub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub filters: Option<std::collections::HashMap<String, String>>,
}
