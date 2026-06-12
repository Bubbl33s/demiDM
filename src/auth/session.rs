use std::ffi::CString;
use std::path::Path;

use nix::sys::wait::waitpid;
use nix::unistd::{self, User};
use tracing::{error, info};

use crate::errors::{AuraError, AuraResult};

#[allow(dead_code)]
pub struct SessionConfig {
    pub display: String,
    pub xauth_path: Option<String>,
}

impl SessionConfig {
    #[allow(dead_code)]
    pub fn default_config() -> Self {
        Self {
            display: ":0".to_string(),
            xauth_path: None,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

#[allow(dead_code)]
pub fn build_session_env(user: &User, config: &SessionConfig) -> AuraResult<Vec<CString>> {
    let home = user.dir.to_string_lossy().to_string();
    let shell = user.shell.to_string_lossy().to_string();
    let username = user.name.clone();
    let uid = user.uid.as_raw();

    let xdg_runtime_dir = format!("/run/user/{}", uid);

    let xauthority = format!("{}/.Xauthority", home);

    let env_vars = vec![
        format!("HOME={}", home),
        format!("USER={}", username),
        format!("LOGNAME={}", username),
        format!("SHELL={}", shell),
        "PATH=/usr/local/bin:/usr/bin:/bin".to_string(),
        format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir),
        "XDG_SESSION_TYPE=x11".to_string(),
        format!("DISPLAY={}", config.display),
        format!("XAUTHORITY={}", xauthority),
    ];

    env_vars
        .into_iter()
        .map(|s| {
            CString::new(s).map_err(|e| AuraError::SessionLaunch(format!("Invalid env var: {}", e)))
        })
        .collect()
}

#[allow(dead_code)]
pub fn lookup_user(username: &str) -> AuraResult<User> {
    User::from_name(username)
        .map_err(|e| AuraError::SessionLaunch(format!("User lookup error: {}", e)))?
        .ok_or_else(|| AuraError::UserNotFound {
            username: username.to_string(),
        })
}

#[allow(dead_code)]
pub fn launch_session(username: &str, config: &SessionConfig) -> AuraResult<()> {
    let user = lookup_user(username)?;

    info!(
        "Launching session for user {} (uid={}, gid={})",
        user.name,
        user.uid.as_raw(),
        user.gid.as_raw()
    );

    let env = build_session_env(&user, config)?;

    let home = user.dir.to_string_lossy().to_string();
    let xinitrc_path = format!("{}/.xinitrc", home);
    let has_xinitrc = Path::new(&xinitrc_path).exists();

    match unsafe { unistd::fork() } {
        Ok(unistd::ForkResult::Parent { child }) => {
            info!("Session launched as child pid {}", child.as_raw());

            match waitpid(child, None) {
                Ok(status) => {
                    info!("Session ended: {:?}", status);
                }
                Err(e) => {
                    error!("waitpid error: {}", e);
                }
            }

            Ok(())
        }
        Ok(unistd::ForkResult::Child) => {
            info!("Child process starting session");

            unistd::setgid(user.gid)
                .map_err(|e| AuraError::SessionLaunch(format!("setgid failed: {}", e)))?;

            unistd::setuid(user.uid)
                .map_err(|e| AuraError::SessionLaunch(format!("setuid failed: {}", e)))?;

            let startx = CString::new("/usr/bin/startx")
                .map_err(|e| AuraError::SessionLaunch(format!("Invalid path: {}", e)))?;

            let mut args: Vec<CString> = vec![startx.clone()];

            if has_xinitrc {
                let xinitrc_c = CString::new(xinitrc_path.as_str()).map_err(|e| {
                    AuraError::SessionLaunch(format!("Invalid xinitrc path: {}", e))
                })?;
                args.push(xinitrc_c);
            }

            let args_with_null: Vec<&CString> = args.iter().collect();

            unistd::execvpe(&startx, &args_with_null, &env)
                .map_err(|e| AuraError::SessionLaunch(format!("execvpe failed: {}", e)))?;

            unreachable!()
        }
        Err(e) => Err(AuraError::SessionLaunch(format!("fork failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_user() -> User {
        User {
            name: "alice".to_string(),
            passwd: CString::new("x").unwrap(),
            uid: unistd::Uid::from_raw(1000),
            gid: unistd::Gid::from_raw(1000),
            gecos: CString::new("Alice").unwrap(),
            dir: PathBuf::from("/home/alice"),
            shell: PathBuf::from("/bin/bash"),
        }
    }

    #[test]
    fn test_build_session_env_contains_required_vars() {
        let user = test_user();
        let config = SessionConfig::default_config();
        let env = build_session_env(&user, &config).unwrap();

        let env_strings: Vec<String> = env
            .iter()
            .map(|c| c.to_string_lossy().to_string())
            .collect();

        assert!(env_strings.iter().any(|e| e == "HOME=/home/alice"));
        assert!(env_strings.iter().any(|e| e == "USER=alice"));
        assert!(env_strings.iter().any(|e| e == "LOGNAME=alice"));
        assert!(env_strings.iter().any(|e| e == "SHELL=/bin/bash"));
        assert!(env_strings.iter().any(|e| e == "XDG_SESSION_TYPE=x11"));
        assert!(env_strings.iter().any(|e| e == "DISPLAY=:0"));
        assert!(env_strings
            .iter()
            .any(|e| e == "XDG_RUNTIME_DIR=/run/user/1000"));
        assert!(env_strings
            .iter()
            .any(|e| e == "XAUTHORITY=/home/alice/.Xauthority"));
    }

    #[test]
    fn test_build_session_env_path() {
        let user = test_user();
        let config = SessionConfig::default_config();
        let env = build_session_env(&user, &config).unwrap();

        let env_strings: Vec<String> = env
            .iter()
            .map(|c| c.to_string_lossy().to_string())
            .collect();

        assert!(env_strings
            .iter()
            .any(|e| e == "PATH=/usr/local/bin:/usr/bin:/bin"));
    }

    #[test]
    fn test_lookup_user_nonexistent() {
        let result = lookup_user("__nonexistent_user_12345__");
        assert!(result.is_err());
        match result.unwrap_err() {
            AuraError::UserNotFound { username } => {
                assert_eq!(username, "__nonexistent_user_12345__");
            }
            other => panic!("Expected UserNotFound, got: {:?}", other),
        }
    }
}
