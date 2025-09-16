// NestJS API endpoint constants
pub const AUTH_BASE: &str = "/auth";
pub const USERS_BASE: &str = "/users";
pub const GROUPS_BASE: &str = "/groups";
pub const WALLETS_BASE: &str = "/wallets";

// Auth endpoints
pub const LOGIN: &str = "/auth/login";
pub const REGISTER: &str = "/auth/register";
pub const VERIFY: &str = "/auth/verify";
pub const AUTHENTICATE: &str = "/auth/authenticate";
pub const RECOVER: &str = "/auth/recover";
pub const REFRESH: &str = "/auth/refresh";
pub const LOGOUT: &str = "/auth/logout";

// User endpoints
pub const GET_USER: &str = "/users";
pub const FIND_USER: &str = "/users/find";
pub const UPDATE_USER: &str = "/users";
pub const DELETE_USER: &str = "/users";

// Group endpoints
pub const GET_GROUPS: &str = "/groups";
pub const CREATE_GROUP: &str = "/groups";
pub const UPDATE_GROUP: &str = "/groups";
pub const DELETE_GROUP: &str = "/groups";

// Wallet endpoints
pub const GET_WALLETS: &str = "/wallets";
pub const CREATE_WALLET: &str = "/wallets";
pub const DELETE_WALLET: &str = "/wallets";
pub const GET_WALLET_TRANSACTIONS: &str = "/wallets";
pub const GET_WALLET_BALANCE: &str = "/wallets";
