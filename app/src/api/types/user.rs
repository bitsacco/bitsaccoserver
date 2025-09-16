use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Nostr, Phone, Profile, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub phone: Option<Phone>,
    pub nostr: Option<Nostr>,
    pub profile: Option<Profile>,
    pub roles: Vec<Role>,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdates {
    pub phone: Option<Phone>,
    pub nostr: Option<Nostr>,
    pub profile: Option<Profile>,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindUserRequest {
    pub id: Option<Uuid>,
    pub phone: Option<String>,
    pub npub: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    pub updates: UserUpdates,
}
