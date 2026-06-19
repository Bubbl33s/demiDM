use crossterm::event::KeyEvent;
use secrecy::SecretString;

use crate::errors::WidgetId;
use crate::graphics::handle::FbOverlayHandle;
use crate::lua_runtime::config::LoginBoxConfig;
use crate::renderer::theme::ThemeConfig;
use crate::widget::WidgetDef;

pub use crate::auth::pam_worker::PamErrorCode;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    KeyPress(KeyEvent),
    Resize(u16, u16),
    Tick,
    Shutdown,
    AuthRequested {
        username: String,
        password: SecretString,
    },
    AuthSuccess {
        username: String,
    },
    AuthFailure {
        username: String,
        code: PamErrorCode,
        message: String,
    },
    ConfigLoaded,
    ConfigError(String),
    ConfigUpdate(ThemeConfig),
    LoginBoxConfigUpdate(LoginBoxConfig),
    Notification(String),
    WidgetUpdate {
        id: WidgetId,
        content: String,
    },
    WidgetRegister(WidgetDef),
    WidgetRemove(WidgetId),
    FbImageLoaded {
        handle: FbOverlayHandle,
    },
}
