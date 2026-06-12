use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TTY error: {0}")]
    Tty(#[from] nix::Error),

    #[error("PAM error (code {code:?}): {message}")]
    Pam {
        code: crate::events::PamErrorCode,
        message: String,
    },

    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("Config not found at {path}")]
    ConfigNotFound { path: String },

    #[error("Framebuffer error: {0}")]
    Framebuffer(String),

    #[error("Session launch error: {0}")]
    SessionLaunch(String),

    #[error("User not found: {username}")]
    UserNotFound { username: String },
}

pub type AuraResult<T> = Result<T, AuraError>;

pub type WidgetId = String;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct AbsolutePosition {
    pub col: u16,
    pub row: u16,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Padding {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}
