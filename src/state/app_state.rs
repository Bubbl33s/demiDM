use secrecy::SecretString;
use tokio::sync::mpsc::Sender;

use crate::events::AppEvent;
use crate::graphics::handle::FbOverlayHandle;
use crate::lua_runtime::config::AuraConfig;
use crate::state::input_field::InputField;
use crate::widget::WidgetInstance;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AppPhase {
    Startup,
    Idle,
    Authenticating,
    AuthSuccess,
    AuthFailure { message: String },
    LaunchingSession,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Username,
    Password,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AuthStatus {
    Idle,
    Authenticating,
    Success,
    Failure { message: String },
}

pub struct AppState {
    pub phase: AppPhase,
    pub username_field: InputField,
    pub password_field: InputField,
    pub active_field: FocusTarget,
    #[allow(dead_code)]
    pub auth_status: AuthStatus,
    pub config: AuraConfig,
    pub notification: Option<String>,
    pub widgets: Vec<WidgetInstance>,
    pub fb_overlay: Option<FbOverlayHandle>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            phase: AppPhase::Idle,
            username_field: InputField::new(),
            password_field: InputField::new(),
            active_field: FocusTarget::Username,
            auth_status: AuthStatus::Idle,
            config: AuraConfig::default(),
            notification: None,
            widgets: Vec::new(),
            fb_overlay: None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_event(state: &mut AppState, event: AppEvent, tx: &Sender<AppEvent>) {
    match event {
        AppEvent::KeyPress(key_event) => {
            use crossterm::event::KeyCode;

            if let AppPhase::AuthFailure { .. } = &state.phase {
                state.phase = AppPhase::Idle;
                state.password_field.clear();
                state.auth_status = AuthStatus::Idle;
                return;
            }

            match key_event.code {
                KeyCode::Enter => {
                    if state.active_field == FocusTarget::Password {
                        let username = state.username_field.display_value().to_string();
                        let password = state.password_field.display_value().to_string();

                        if !username.is_empty() && !password.is_empty() {
                            state.phase = AppPhase::Authenticating;
                            state.auth_status = AuthStatus::Authenticating;
                            state.password_field.clear();

                            let _ = tx.try_send(AppEvent::AuthRequested {
                                username,
                                password: SecretString::from(password),
                            });
                        }
                    }
                }
                KeyCode::Tab => {
                    state.active_field = match state.active_field {
                        FocusTarget::Username => FocusTarget::Password,
                        FocusTarget::Password => FocusTarget::Username,
                    };
                }
                KeyCode::BackTab => {
                    state.active_field = match state.active_field {
                        FocusTarget::Username => FocusTarget::Password,
                        FocusTarget::Password => FocusTarget::Username,
                    };
                }
                KeyCode::Backspace => match state.active_field {
                    FocusTarget::Username => state.username_field.pop_char(),
                    FocusTarget::Password => state.password_field.pop_char(),
                },
                KeyCode::Char(c) => match state.active_field {
                    FocusTarget::Username => state.username_field.push_char(c),
                    FocusTarget::Password => state.password_field.push_char(c),
                },
                _ => {}
            }
        }
        AppEvent::Resize(_, _) => {}
        AppEvent::Tick => {}
        AppEvent::Shutdown => {
            state.phase = AppPhase::Shutdown;
        }
        AppEvent::AuthRequested { .. } => {
            state.phase = AppPhase::Authenticating;
            state.auth_status = AuthStatus::Authenticating;
            state.password_field.clear();
        }
        AppEvent::AuthSuccess { username } => {
            state.phase = AppPhase::AuthSuccess;
            state.auth_status = AuthStatus::Success;
            let _ = username;
        }
        AppEvent::AuthFailure { code: _, message } => {
            state.phase = AppPhase::AuthFailure {
                message: message.clone(),
            };
            state.auth_status = AuthStatus::Failure { message };
            state.password_field.clear();
        }
        AppEvent::ConfigLoaded => {
            tracing::info!("Config loaded event received");
        }
        AppEvent::ConfigError(msg) => {
            tracing::error!("Config error: {}", msg);
            state.notification = Some(format!("Config error: {}", msg));
        }
        AppEvent::ConfigUpdate(theme) => {
            state.config.theme = theme;
        }
        AppEvent::LoginBoxConfigUpdate(login_box) => {
            state.config.login_box = login_box;
        }
        AppEvent::Notification(msg) => {
            state.notification = Some(msg);
        }
        AppEvent::WidgetUpdate { id, content } => {
            if let Some(widget) = state.widgets.iter_mut().find(|w| w.def.id == id) {
                widget.content = content;
                widget.dirty = true;
            }
        }
        AppEvent::WidgetRegister(def) => {
            if !state.widgets.iter().any(|w| w.def.id == def.id) {
                state.widgets.push(WidgetInstance::new(def));
            }
        }
        AppEvent::WidgetRemove(id) => {
            state.widgets.retain(|w| w.def.id != id);
        }
        AppEvent::FbImageLoaded { handle } => {
            state.fb_overlay = Some(handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::PamErrorCode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn test_sender() -> Sender<AppEvent> {
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        tx
    }

    #[test]
    fn test_keypress_appends_to_username_field() {
        let mut state = AppState::new();
        state.active_field = FocusTarget::Username;
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.username_field.display_value(), "a");
    }

    #[test]
    fn test_keypress_appends_to_password_field() {
        let mut state = AppState::new();
        state.active_field = FocusTarget::Password;
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.password_field.display_value(), "x");
    }

    #[test]
    fn test_tab_switches_focus() {
        let mut state = AppState::new();
        let tx = test_sender();
        assert_eq!(state.active_field, FocusTarget::Username);

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.active_field, FocusTarget::Password);
    }

    #[test]
    fn test_backspace_removes_char() {
        let mut state = AppState::new();
        state.username_field.push_char('a');
        state.username_field.push_char('b');
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.username_field.display_value(), "a");
    }

    #[test]
    fn test_shutdown_event() {
        let mut state = AppState::new();
        let tx = test_sender();
        apply_event(&mut state, AppEvent::Shutdown, &tx);
        assert_eq!(state.phase, AppPhase::Shutdown);
    }

    #[test]
    fn test_enter_triggers_auth_when_password_focused() {
        let mut state = AppState::new();
        state.username_field.push_char('a');
        state.username_field.push_char('l');
        state.username_field.push_char('i');
        state.username_field.push_char('c');
        state.username_field.push_char('e');
        state.active_field = FocusTarget::Password;
        state.password_field.push_char('p');
        state.password_field.push_char('a');
        state.password_field.push_char('s');
        state.password_field.push_char('s');
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.phase, AppPhase::Authenticating);
        assert_eq!(state.password_field.display_value(), "");
    }

    #[test]
    fn test_enter_does_not_trigger_auth_when_username_focused() {
        let mut state = AppState::new();
        state.username_field.push_char('a');
        state.active_field = FocusTarget::Username;
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.phase, AppPhase::Idle);
    }

    #[test]
    fn test_auth_success_transition() {
        let mut state = AppState::new();
        state.phase = AppPhase::Authenticating;
        let tx = test_sender();

        apply_event(
            &mut state,
            AppEvent::AuthSuccess {
                username: "alice".to_string(),
            },
            &tx,
        );

        assert_eq!(state.phase, AppPhase::AuthSuccess);
    }

    #[test]
    fn test_auth_failure_transition() {
        let mut state = AppState::new();
        state.phase = AppPhase::Authenticating;
        let tx = test_sender();

        apply_event(
            &mut state,
            AppEvent::AuthFailure {
                code: PamErrorCode::AuthError,
                message: "Invalid password".to_string(),
            },
            &tx,
        );

        assert_eq!(
            state.phase,
            AppPhase::AuthFailure {
                message: "Invalid password".to_string()
            }
        );
        assert_eq!(state.password_field.display_value(), "");
    }

    #[test]
    fn test_recovery_from_auth_failure() {
        let mut state = AppState::new();
        state.phase = AppPhase::AuthFailure {
            message: "error".to_string(),
        };
        state.password_field.push_char('x');
        let tx = test_sender();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        apply_event(&mut state, AppEvent::KeyPress(key_event), &tx);

        assert_eq!(state.phase, AppPhase::Idle);
        assert_eq!(state.password_field.display_value(), "");
    }

    #[test]
    fn test_auth_requested_clears_password() {
        let mut state = AppState::new();
        state.password_field.push_char('s');
        state.password_field.push_char('e');
        state.password_field.push_char('c');
        let tx = test_sender();

        apply_event(
            &mut state,
            AppEvent::AuthRequested {
                username: "alice".to_string(),
                password: SecretString::from("secret".to_string()),
            },
            &tx,
        );

        assert_eq!(state.phase, AppPhase::Authenticating);
        assert_eq!(state.password_field.display_value(), "");
    }

    #[test]
    fn test_widget_update_applies_to_correct_widget() {
        use crate::errors::AbsolutePosition;
        use crate::widget::{WidgetDef, WidgetInstance, WidgetSource, WidgetStyle};
        use std::time::Duration;

        let mut state = AppState::new();
        let tx = test_sender();

        let widget_a = WidgetInstance::new(WidgetDef {
            id: "widget_a".to_string(),
            position: AbsolutePosition { col: 0, row: 0 },
            width: 20,
            height: None,
            refresh: Duration::from_secs(5),
            source: WidgetSource::StaticText("initial_a".to_string()),
            style: WidgetStyle::default(),
        });
        let widget_b = WidgetInstance::new(WidgetDef {
            id: "widget_b".to_string(),
            position: AbsolutePosition { col: 0, row: 5 },
            width: 20,
            height: None,
            refresh: Duration::from_secs(5),
            source: WidgetSource::StaticText("initial_b".to_string()),
            style: WidgetStyle::default(),
        });
        state.widgets.push(widget_a);
        state.widgets.push(widget_b);

        apply_event(
            &mut state,
            AppEvent::WidgetUpdate {
                id: "widget_b".to_string(),
                content: "updated_b".to_string(),
            },
            &tx,
        );

        assert_eq!(state.widgets[0].content, "");
        assert!(!state.widgets[0].dirty);
        assert_eq!(state.widgets[1].content, "updated_b");
        assert!(state.widgets[1].dirty);
    }
}
