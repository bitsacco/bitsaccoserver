use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub groups: Vec<Uuid>,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub user: RwSignal<Option<UserContext>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub notifications: RwSignal<Vec<Notification>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub read: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
            notifications: RwSignal::new(Vec::new()),
        }
    }

    pub fn set_user(&self, user: Option<UserContext>) {
        self.user.set(user);
    }

    pub fn set_loading(&self, loading: bool) {
        self.loading.set(loading);
    }

    pub fn set_error(&self, error: Option<String>) {
        self.error.set(error);
    }

    pub fn add_notification(&self, notification: Notification) {
        self.notifications.update(|notifications| {
            notifications.push(notification);
        });
    }

    pub fn remove_notification(&self, id: Uuid) {
        self.notifications.update(|notifications| {
            notifications.retain(|n| n.id != id);
        });
    }

    pub fn mark_notification_read(&self, id: Uuid) {
        self.notifications.update(|notifications| {
            if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
                notification.read = true;
            }
        });
    }

    pub fn clear_error(&self) {
        self.error.set(None);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn provide_app_state() -> AppState {
    let state = AppState::new();
    provide_context(state.clone());
    state
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>().expect("AppState not provided")
}

pub fn use_user() -> (
    ReadSignal<Option<UserContext>>,
    WriteSignal<Option<UserContext>>,
) {
    let state = use_app_state();
    state.user.split()
}

pub fn use_loading() -> (ReadSignal<bool>, WriteSignal<bool>) {
    let state = use_app_state();
    state.loading.split()
}

pub fn use_error() -> (ReadSignal<Option<String>>, WriteSignal<Option<String>>) {
    let state = use_app_state();
    state.error.split()
}

pub fn use_notifications() -> (
    ReadSignal<Vec<Notification>>,
    WriteSignal<Vec<Notification>>,
) {
    let state = use_app_state();
    state.notifications.split()
}
