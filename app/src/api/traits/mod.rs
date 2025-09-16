pub mod auth;
pub mod groups;
pub mod users;
pub mod wallets;

// Re-export all traits
pub use auth::AuthApi;
pub use groups::GroupsApi;
pub use users::UsersApi;
pub use wallets::WalletsApi;
