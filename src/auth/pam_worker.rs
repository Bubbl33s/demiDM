use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use secrecy::{ExposeSecret, SecretString};
use tokio::sync::mpsc::Sender;
use tracing::{error, info};

use crate::errors::AuraError;
use crate::events::AppEvent;

#[allow(dead_code)]
const PAM_SUCCESS: c_int = 0;
#[allow(dead_code)]
const PAM_PROMPT_ECHO_OFF: c_int = 2;
#[allow(dead_code)]
const PAM_PROMPT_ECHO_ON: c_int = 1;
#[allow(dead_code)]
const PAM_TEXT_INFO: c_int = 3;
#[allow(dead_code)]
const PAM_ERROR_MSG: c_int = 4;
#[allow(dead_code)]
const PAM_AUTHTOK_ERR: c_int = 20;

#[allow(dead_code)]
const PAM_AUTH_ERR: c_int = 7;
#[allow(dead_code)]
const PAM_ACCT_EXPIRED: c_int = 13;
#[allow(dead_code)]
const PAM_CRED_INSUFFICIENT: c_int = 11;
#[allow(dead_code)]
const PAM_SYSTEM_ERR: c_int = 4;
#[allow(dead_code)]
const PAM_BUF_ERR: c_int = 5;
#[allow(dead_code)]
const PAM_CONV_ERR: c_int = 6;
#[allow(dead_code)]
const PAM_PERM_DENIED: c_int = 24;
#[allow(dead_code)]
const PAM_MAXTRIES: c_int = 12;
#[allow(dead_code)]
const PAM_NEW_AUTHTOK_REQD: c_int = 10;
#[allow(dead_code)]
const PAM_ACCT_DISABLED: c_int = 17;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamErrorCode {
    AuthError,
    AccountExpired,
    CredInsufficient,
    SystemError,
    BufEmpty,
    ConvErr,
    PermDenied,
    Maxtries,
    NewTokenRequired,
    AcctDisabled,
    Unknown(i32),
}

impl From<i32> for PamErrorCode {
    fn from(code: i32) -> Self {
        match code {
            PAM_AUTH_ERR => PamErrorCode::AuthError,
            PAM_ACCT_EXPIRED => PamErrorCode::AccountExpired,
            PAM_CRED_INSUFFICIENT => PamErrorCode::CredInsufficient,
            PAM_SYSTEM_ERR => PamErrorCode::SystemError,
            PAM_BUF_ERR => PamErrorCode::BufEmpty,
            PAM_CONV_ERR => PamErrorCode::ConvErr,
            PAM_PERM_DENIED => PamErrorCode::PermDenied,
            PAM_MAXTRIES => PamErrorCode::Maxtries,
            PAM_NEW_AUTHTOK_REQD => PamErrorCode::NewTokenRequired,
            PAM_ACCT_DISABLED => PamErrorCode::AcctDisabled,
            other => PamErrorCode::Unknown(other),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PamRequest {
    pub username: String,
    pub password: SecretString,
}

impl PamRequest {
    #[allow(dead_code)]
    pub fn new(username: String, password: SecretString) -> Self {
        Self { username, password }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PamResult {
    Success { username: String },
    Failure { code: PamErrorCode, message: String },
}

#[derive(Debug)]
pub struct PamError {
    pub code: PamErrorCode,
    pub message: String,
}

impl std::fmt::Display for PamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PAM error ({:?}): {}", self.code, self.message)
    }
}

impl std::error::Error for PamError {}

impl From<PamError> for AuraError {
    fn from(err: PamError) -> Self {
        AuraError::Pam {
            code: err.code,
            message: err.message,
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[allow(dead_code)]
#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

#[allow(dead_code)]
#[repr(C)]
struct PamConv {
    conv: Option<
        unsafe extern "C" fn(
            num_msg: c_int,
            msg: *mut *const PamMessage,
            resp: *mut *mut PamResponse,
            appdata_ptr: *mut c_void,
        ) -> c_int,
    >,
    appdata_ptr: *mut c_void,
}

#[link(name = "pam")]
extern "C" {
    #[allow(dead_code)]
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conv: *const PamConv,
        pamh: *mut *mut c_void,
    ) -> c_int;
    #[allow(dead_code)]
    fn pam_authenticate(pamh: *mut c_void, flags: c_int) -> c_int;
    #[allow(dead_code)]
    fn pam_acct_mgmt(pamh: *mut c_void, flags: c_int) -> c_int;
    #[allow(dead_code)]
    fn pam_open_session(pamh: *mut c_void, flags: c_int) -> c_int;
    #[allow(dead_code)]
    fn pam_close_session(pamh: *mut c_void, flags: c_int) -> c_int;
    #[allow(dead_code)]
    fn pam_end(pamh: *mut c_void, pam_status: c_int) -> c_int;
    #[allow(dead_code)]
    fn pam_strerror(pamh: *mut c_void, errnum: c_int) -> *const c_char;
}

#[allow(dead_code)]
unsafe extern "C" fn pam_conv_fn(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 || msg.is_null() || resp.is_null() {
        return PAM_CONV_ERR;
    }

    let password = if appdata_ptr.is_null() {
        return PAM_CONV_ERR;
    } else {
        unsafe { CStr::from_ptr(appdata_ptr as *const c_char) }
    };

    let responses =
        libc::calloc(num_msg as usize, std::mem::size_of::<PamResponse>()) as *mut PamResponse;
    if responses.is_null() {
        return PAM_BUF_ERR;
    }

    for i in 0..num_msg as usize {
        let msg_ptr = unsafe { *msg.add(i) };
        if msg_ptr.is_null() {
            zero_and_free_responses(responses, i);
            return PAM_CONV_ERR;
        }

        let msg_ref = unsafe { &*msg_ptr };
        let msg_text = if !msg_ref.msg.is_null() {
            unsafe { CStr::from_ptr(msg_ref.msg) }.to_string_lossy()
        } else {
            std::borrow::Cow::Borrowed("(null)")
        };

        info!(
            "PAM conversation message {}: style={}, text={}",
            i, msg_ref.msg_style, msg_text
        );

        let c_pwd = libc::strdup(password.as_ptr());
        if c_pwd.is_null() {
            zero_and_free_responses(responses, i);
            return PAM_BUF_ERR;
        }
        unsafe {
            (*responses.add(i)).resp = c_pwd;
            (*responses.add(i)).resp_retcode = 0;
        }
    }

    unsafe {
        // Success path: PAM takes ownership of responses and is responsible for the eventual free.
        *resp = responses;
    }

    PAM_SUCCESS
}

unsafe fn zero_and_free_responses(responses: *mut PamResponse, count: usize) {
    for j in 0..count {
        let r = unsafe { &mut *responses.add(j) };
        if !r.resp.is_null() {
            let len = unsafe { libc::strlen(r.resp) };
            unsafe { libc::explicit_bzero(r.resp as *mut c_void, len) };
            unsafe { libc::free(r.resp as *mut c_void) };
            r.resp = ptr::null_mut();
        }
    }
    unsafe { libc::free(responses as *mut c_void) };
}

#[allow(dead_code)]
pub fn pam_authenticate_blocking(req: &PamRequest) -> Result<String, PamError> {
    let service = CString::new("demidm").map_err(|e| PamError {
        code: PamErrorCode::SystemError,
        message: format!("Invalid service name: {}", e),
    })?;

    let username_c = CString::new(req.username.as_str()).map_err(|e| PamError {
        code: PamErrorCode::SystemError,
        message: format!("Invalid username: {}", e),
    })?;

    let pwd = req.password.expose_secret();
    let password_c = CString::new(pwd.as_str()).map_err(|e| PamError {
        code: PamErrorCode::SystemError,
        message: format!("Invalid password: {}", e),
    })?;
    info!(
        "PAM: password length={}, is_empty={}",
        pwd.len(),
        pwd.is_empty()
    );

    let conv = PamConv {
        conv: Some(pam_conv_fn),
        appdata_ptr: password_c.as_ptr() as *mut c_void,
    };

    let mut pamh: *mut c_void = ptr::null_mut();

    let status = unsafe { pam_start(service.as_ptr(), username_c.as_ptr(), &conv, &mut pamh) };
    info!("pam_start returned status={}", status);

    if status != PAM_SUCCESS {
        let err_msg = unsafe {
            CStr::from_ptr(pam_strerror(pamh, status))
                .to_string_lossy()
                .into_owned()
        };
        unsafe { pam_end(pamh, status) };
        return Err(PamError {
            code: PamErrorCode::from(status),
            message: format!("pam_start failed: {}", err_msg),
        });
    }

    let status = unsafe { pam_authenticate(pamh, 0) };
    info!("pam_authenticate returned status={}", status);

    if status != PAM_SUCCESS {
        let err_msg = unsafe {
            CStr::from_ptr(pam_strerror(pamh, status))
                .to_string_lossy()
                .into_owned()
        };
        unsafe { pam_end(pamh, status) };
        return Err(PamError {
            code: PamErrorCode::from(status),
            message: format!("Authentication failed: {}", err_msg),
        });
    }

    let status = unsafe { pam_acct_mgmt(pamh, 0) };
    info!("pam_acct_mgmt returned status={}", status);

    if status != PAM_SUCCESS {
        let err_msg = unsafe {
            CStr::from_ptr(pam_strerror(pamh, status))
                .to_string_lossy()
                .into_owned()
        };
        unsafe { pam_end(pamh, status) };
        return Err(PamError {
            code: PamErrorCode::from(status),
            message: format!("Account validation failed: {}", err_msg),
        });
    }

    let status = unsafe { pam_open_session(pamh, 0) };
    info!("pam_open_session returned status={}", status);

    if status != PAM_SUCCESS {
        let err_msg = unsafe {
            CStr::from_ptr(pam_strerror(pamh, status))
                .to_string_lossy()
                .into_owned()
        };
        unsafe { pam_end(pamh, status) };
        return Err(PamError {
            code: PamErrorCode::from(status),
            message: format!("Open session failed: {}", err_msg),
        });
    }

    let result = Ok(req.username.clone());

    unsafe { pam_close_session(pamh, 0) };
    info!("pam_close_session called");
    unsafe { pam_end(pamh, PAM_SUCCESS) };

    result
}

#[allow(dead_code)]
pub fn authenticate(req: PamRequest, tx: Sender<AppEvent>) {
    let username = req.username.clone();

    let result = pam_authenticate_blocking(&req);

    drop(req);

    match result {
        Ok(user) => {
            info!("PAM auth success for user: {}", user);
            let _ = tx.blocking_send(AppEvent::AuthSuccess { username: user });
        }
        Err(err) => {
            error!("PAM auth failed for user {}: {}", username, err);
            let _ = tx.blocking_send(AppEvent::AuthFailure {
                username: username.clone(),
                code: err.code,
                message: err.message,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pam_request_creation() {
        let req = PamRequest::new(
            "alice".to_string(),
            SecretString::from("secret123".to_string()),
        );
        assert_eq!(req.username, "alice");
    }

    #[test]
    fn test_password_not_in_debug() {
        let req = PamRequest::new(
            "alice".to_string(),
            SecretString::from("supersecret".to_string()),
        );
        let debug_str = format!("{:?}", req);
        assert!(!debug_str.contains("supersecret"));
    }

    #[test]
    fn test_pam_error_code_from_i32() {
        assert_eq!(PamErrorCode::from(PAM_AUTH_ERR), PamErrorCode::AuthError);
        assert_eq!(
            PamErrorCode::from(PAM_ACCT_EXPIRED),
            PamErrorCode::AccountExpired
        );
        assert_eq!(
            PamErrorCode::from(PAM_PERM_DENIED),
            PamErrorCode::PermDenied
        );
        assert_eq!(PamErrorCode::from(99), PamErrorCode::Unknown(99));
    }

    #[test]
    fn test_pam_close_session_is_linked() {
        let _fn_ptr: unsafe extern "C" fn(*mut c_void, c_int) -> c_int = pam_close_session;
    }
}
