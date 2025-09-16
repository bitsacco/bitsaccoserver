use crate::contexts::app_state::{use_app_state, Notification, NotificationType};
use gloo_timers::callback::Timeout;
use leptos::prelude::*;

#[component]
pub fn ToastContainer() -> impl IntoView {
    let app_state = use_app_state();
    let notifications = app_state.notifications;

    view! {
        <div class="fixed inset-0 flex items-end justify-center px-4 py-6 pointer-events-none sm:p-6 sm:items-start sm:justify-end z-50">
            <div class="w-full flex flex-col items-center space-y-4 sm:items-end">
                <For
                    each=move || notifications.get()
                    key=|notification| notification.id
                    children=move |notification| {
                        view! {
                            <ToastItem notification=notification />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn ToastItem(notification: Notification) -> impl IntoView {
    let app_state = use_app_state();
    let (visible, set_visible) = signal(true);
    let id = notification.id;

    // Auto-dismiss after 5 seconds
    let app_state_clone = app_state.clone();
    let set_visible_clone = set_visible;
    let timeout = Timeout::new(5_000, move || {
        set_visible_clone.set(false);
        // Remove from notifications after animation
        let app_state_clone2 = app_state_clone.clone();
        let timeout = Timeout::new(300, move || {
            app_state_clone2.remove_notification(id);
        });
        timeout.forget();
    });
    timeout.forget();

    let (bg_color, icon_color, icon) = match notification.notification_type {
        NotificationType::Success => ("bg-green-50", "text-green-400", "✓"),
        NotificationType::Error => ("bg-red-50", "text-red-400", "✕"),
        NotificationType::Warning => ("bg-yellow-50", "text-yellow-400", "⚠"),
        NotificationType::Info => ("bg-blue-50", "text-blue-400", "ℹ"),
    };

    let text_color = match notification.notification_type {
        NotificationType::Success => "text-green-800",
        NotificationType::Error => "text-red-800",
        NotificationType::Warning => "text-yellow-800",
        NotificationType::Info => "text-blue-800",
    };

    view! {
        <div
            class=format!(
                "max-w-sm w-full {} shadow-lg rounded-lg pointer-events-auto ring-1 ring-black ring-opacity-5 overflow-hidden transition-all duration-300 ease-in-out transform {}",
                bg_color,
                if visible.get() { "translate-y-0 opacity-100" } else { "translate-y-2 opacity-0" }
            )
        >
            <div class="p-4">
                <div class="flex items-start">
                    <div class="flex-shrink-0">
                        <div class=format!("w-5 h-5 {} flex items-center justify-center text-sm font-bold", icon_color)>
                            {icon}
                        </div>
                    </div>
                    <div class="ml-3 w-0 flex-1 pt-0.5">
                        <p class=format!("text-sm font-medium {}", text_color)>
                            {notification.title}
                        </p>
                        <p class=format!("mt-1 text-sm {}", text_color.replace("800", "700"))>
                            {notification.message}
                        </p>
                    </div>
                    <div class="ml-4 flex-shrink-0 flex">
                        <button
                            class=format!("rounded-md inline-flex {} hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500", text_color.replace("800", "400"))
                            on:click={
                                let app_state_clone = app_state.clone();
                                move |_| {
                                    set_visible.set(false);
                                    let app_state_clone2 = app_state_clone.clone();
                                    let timeout = Timeout::new(300, move || {
                                        app_state_clone2.remove_notification(id);
                                    });
                                    timeout.forget();
                                }
                            }
                        >
                            <span class="sr-only">"Close"</span>
                            <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// Toast utility functions
pub fn show_success(title: &str, message: &str) {
    let app_state = use_app_state();
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        message: message.to_string(),
        notification_type: NotificationType::Success,
        timestamp: chrono::Utc::now(),
        read: false,
    };
    app_state.add_notification(notification);
}

pub fn show_error(title: &str, message: &str) {
    let app_state = use_app_state();
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        message: message.to_string(),
        notification_type: NotificationType::Error,
        timestamp: chrono::Utc::now(),
        read: false,
    };
    app_state.add_notification(notification);
}

pub fn show_warning(title: &str, message: &str) {
    let app_state = use_app_state();
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        message: message.to_string(),
        notification_type: NotificationType::Warning,
        timestamp: chrono::Utc::now(),
        read: false,
    };
    app_state.add_notification(notification);
}

pub fn show_info(title: &str, message: &str) {
    let app_state = use_app_state();
    let notification = Notification {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        message: message.to_string(),
        notification_type: NotificationType::Info,
        timestamp: chrono::Utc::now(),
        read: false,
    };
    app_state.add_notification(notification);
}
