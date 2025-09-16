use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: u64,
    pub last_activity: u64,
    pub expires_at: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub session_timeout_minutes: u32,
    pub warning_before_expiry_minutes: u32,
    pub max_concurrent_sessions: u32,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 30,
            warning_before_expiry_minutes: 5,
            max_concurrent_sessions: 3,
        }
    }
}

impl SessionManager {
    pub fn new(
        session_timeout_minutes: u32,
        warning_before_expiry_minutes: u32,
        max_concurrent_sessions: u32,
    ) -> Self {
        Self {
            session_timeout_minutes,
            warning_before_expiry_minutes,
            max_concurrent_sessions,
        }
    }

    /// Generate a unique device fingerprint
    pub fn generate_device_id() -> String {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            if let Some(window) = window() {
                let mut fingerprint_parts = Vec::new();

                // Screen resolution
                if let Ok(screen) = window.screen() {
                    fingerprint_parts.push(format!(
                        "{}x{}",
                        screen.width().unwrap_or(0),
                        screen.height().unwrap_or(0)
                    ));
                }

                // Timezone
                let timezone_offset = js_sys::Date::new_0().get_timezone_offset();
                fingerprint_parts.push(timezone_offset.to_string());

                // User agent (partial)
                if let Ok(navigator) = window.navigator() {
                    if let Ok(user_agent) = navigator.user_agent() {
                        // Use a hash of user agent for privacy
                        let hash = Self::simple_hash(&user_agent);
                        fingerprint_parts.push(hash.to_string());
                    }
                }

                // Language
                if let Ok(navigator) = window.navigator() {
                    if let Ok(language) = navigator.language() {
                        fingerprint_parts.push(language);
                    }
                }

                // Combine and hash for final device ID
                let combined = fingerprint_parts.join("|");
                format!("dev_{}", Self::simple_hash(&combined))
            } else {
                format!("dev_{}", js_sys::Date::now() as u64)
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            format!(
                "dev_server_{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )
        }
    }

    fn _simple_hash(input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a session should show expiry warning
    pub fn should_show_expiry_warning(&self, expires_at: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let warning_threshold = expires_at - (self.warning_before_expiry_minutes as u64 * 60);
        current_time >= warning_threshold && current_time < expires_at
    }

    /// Check if a session has expired
    pub fn is_session_expired(&self, expires_at: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_time >= expires_at
    }

    /// Calculate session expiry time
    pub fn calculate_expiry_time(&self) -> u64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_time + (self.session_timeout_minutes as u64 * 60)
    }

    /// Update session activity timestamp
    pub fn update_activity(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get time remaining in session (in seconds)
    pub fn time_remaining(&self, expires_at: u64) -> i64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        expires_at as i64 - current_time as i64
    }

    /// Format time remaining for display
    pub fn format_time_remaining(&self, expires_at: u64) -> String {
        let remaining = self.time_remaining(expires_at);

        if remaining <= 0 {
            "Expired".to_string()
        } else if remaining < 60 {
            format!("{} seconds", remaining)
        } else if remaining < 3600 {
            let minutes = remaining / 60;
            format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" })
        } else {
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            if minutes == 0 {
                format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
            } else {
                format!(
                    "{} hour{} {} minute{}",
                    hours,
                    if hours == 1 { "" } else { "s" },
                    minutes,
                    if minutes == 1 { "" } else { "s" }
                )
            }
        }
    }
}

/// Session timeout warning component
#[component]
pub fn SessionTimeoutWarning(
    session_expires_at: Signal<Option<u64>>,
    on_extend_session: Callback<(), ()>,
    on_logout: Callback<(), ()>,
) -> impl IntoView {
    let session_manager = SessionManager::default();
    let (show_warning, set_show_warning) = signal(false);
    let (time_remaining, set_time_remaining) = signal(String::new());

    // Monitor session expiry
    {
        let session_manager_clone = session_manager.clone();
        Effect::new(move |_| {
            if let Some(expires_at) = session_expires_at.get() {
                let should_warn = session_manager_clone.should_show_expiry_warning(expires_at);
                let is_expired = session_manager_clone.is_session_expired(expires_at);

                if is_expired {
                    tracing::warn!("Session expired, triggering logout");
                    on_logout.run(());
                    return;
                }

                set_show_warning.set(should_warn);

                if should_warn {
                    let remaining = session_manager_clone.format_time_remaining(expires_at);
                    set_time_remaining.set(remaining);
                }
            } else {
                set_show_warning.set(false);
            }
        });
    }

    // Update timer every 10 seconds when warning is shown
    {
        let session_manager_clone = session_manager.clone();
        let session_expires_at_clone = session_expires_at;
        let on_logout_clone = on_logout;
        let set_time_remaining_clone = set_time_remaining;
        let set_show_warning_clone = set_show_warning;

        Effect::new(move |_| {
            if show_warning.get() {
                let session_mgr = session_manager_clone.clone();
                let expires_at_sig = session_expires_at_clone;
                let logout_cb = on_logout_clone;
                let time_setter = set_time_remaining_clone;
                let warning_setter = set_show_warning_clone;

                spawn_local(async move {
                    loop {
                        TimeoutFuture::new(10000).await; // 10 seconds

                        if let Some(expires_at) = expires_at_sig.get() {
                            if session_mgr.is_session_expired(expires_at) {
                                logout_cb.run(());
                                break;
                            }

                            let remaining = session_mgr.format_time_remaining(expires_at);
                            time_setter.set(remaining);

                            if !session_mgr.should_show_expiry_warning(expires_at) {
                                warning_setter.set(false);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                });
            }
        });
    }

    view! {
        <Show when=move || show_warning.get()>
            <div class="fixed top-4 right-4 z-50 max-w-sm">
                <div class="bg-yellow-50 border border-yellow-200 rounded-lg shadow-lg p-4">
                    <div class="flex items-start">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3 flex-1">
                            <h3 class="text-sm font-medium text-yellow-800">
                                "Session Expiring Soon"
                            </h3>
                            <p class="mt-1 text-sm text-yellow-700">
                                "Your session will expire in " {move || time_remaining.get()} "."
                            </p>
                            <div class="mt-3 flex space-x-2">
                                <button
                                    type="button"
                                    class="inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded text-yellow-800 bg-yellow-100 hover:bg-yellow-200 focus:outline-none focus:ring-2 focus:ring-yellow-500 transition-colors"
                                    on:click=move |_| {
                                        on_extend_session.run(());
                                        set_show_warning.set(false);
                                    }
                                >
                                    "Extend Session"
                                </button>
                                <button
                                    type="button"
                                    class="inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded text-gray-700 bg-gray-100 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500 transition-colors"
                                    on:click=move |_| {
                                        on_logout.run(());
                                    }
                                >
                                    "Logout Now"
                                </button>
                            </div>
                        </div>
                        <div class="ml-4 flex-shrink-0 flex">
                            <button
                                type="button"
                                class="bg-yellow-50 rounded-md inline-flex text-yellow-400 hover:text-yellow-500 focus:outline-none focus:ring-2 focus:ring-yellow-500"
                                on:click=move |_| set_show_warning.set(false)
                            >
                                <span class="sr-only">"Close"</span>
                                <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Activity tracker component that monitors user interactions
#[component]
pub fn ActivityTracker(
    on_activity: Callback<(), ()>,
    #[prop(optional)] debounce_ms: Option<u32>,
) -> impl IntoView {
    let debounce_duration = debounce_ms.unwrap_or(5000); // 5 seconds default
    let (last_activity, set_last_activity) = signal(0u64);

    // Track various user activities
    let _track_activity = move || {
        let current_time = js_sys::Date::now() as u64;
        let last_time = last_activity.get();

        // Debounce activity tracking
        if current_time - last_time > debounce_duration as u64 {
            set_last_activity.set(current_time);
            on_activity.run(());
        }
    };

    // Set up global event listeners
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{closure::Closure, JsCast};
            use web_sys::{window, EventTarget};

            if let Some(window) = window() {
                let track_activity_clone = track_activity.clone();
                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    track_activity_clone();
                }) as Box<dyn FnMut(_)>);

                // Track mouse movements
                let _ = window.add_event_listener_with_callback(
                    "mousemove",
                    closure.as_ref().unchecked_ref(),
                );

                // Track keyboard activity
                let track_activity_clone2 = track_activity.clone();
                let closure2 = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    track_activity_clone2();
                }) as Box<dyn FnMut(_)>);
                let _ = window
                    .add_event_listener_with_callback("keydown", closure2.as_ref().unchecked_ref());

                // Track scroll activity
                let track_activity_clone3 = track_activity.clone();
                let closure3 = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    track_activity_clone3();
                }) as Box<dyn FnMut(_)>);
                let _ = window
                    .add_event_listener_with_callback("scroll", closure3.as_ref().unchecked_ref());

                // Track click activity
                let track_activity_clone4 = track_activity.clone();
                let closure4 = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    track_activity_clone4();
                }) as Box<dyn FnMut(_)>);
                let _ = window
                    .add_event_listener_with_callback("click", closure4.as_ref().unchecked_ref());

                // Clean up on component unmount
                // Note: In a real implementation, you'd want to remove these listeners
                // when the component is destroyed to prevent memory leaks
                closure.forget();
                closure2.forget();
                closure3.forget();
                closure4.forget();
            }
        }
    });

    // This component doesn't render anything visible
    view! { <div style="display: none;"></div> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_expiry_calculation() {
        let manager = SessionManager::new(30, 5, 3);
        let expiry = manager.calculate_expiry_time();
        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Should expire in approximately 30 minutes (1800 seconds)
        assert!((expiry - current) >= 1799 && (expiry - current) <= 1801);
    }

    #[test]
    fn test_session_manager_warning_logic() {
        let manager = SessionManager::new(30, 5, 3);
        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Session expires in 3 minutes - should show warning
        let expires_soon = current + 180;
        assert!(manager.should_show_expiry_warning(expires_soon));

        // Session expires in 10 minutes - should not show warning
        let expires_later = current + 600;
        assert!(!manager.should_show_expiry_warning(expires_later));

        // Session already expired - should not show warning (should logout instead)
        let expired = current - 60;
        assert!(!manager.should_show_expiry_warning(expired));
        assert!(manager.is_session_expired(expired));
    }

    #[test]
    fn test_time_remaining_formatting() {
        let manager = SessionManager::default();
        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Test various time remaining scenarios
        assert_eq!(manager.format_time_remaining(current + 30), "30 seconds");
        assert_eq!(manager.format_time_remaining(current + 90), "1 minute");
        assert_eq!(manager.format_time_remaining(current + 150), "2 minutes");
        assert_eq!(manager.format_time_remaining(current + 3600), "1 hour");
        assert_eq!(
            manager.format_time_remaining(current + 3720),
            "1 hour 2 minutes"
        );
        assert_eq!(manager.format_time_remaining(current - 60), "Expired");
    }
}
