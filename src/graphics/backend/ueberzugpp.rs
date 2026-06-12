use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

use image::RgbaImage;
use serde_json::json;

use super::{BackgroundBackend, RenderOpts};
use crate::errors::{AuraError, AuraResult};

pub struct UeberzugppBackend {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}

impl UeberzugppBackend {
    pub fn new() -> Self {
        let mut process = None;
        let mut stdin = None;

        if let Ok(child) = Command::new("ueberzugpp")
            .args(["layer", "--silent"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            stdin = child.stdin.as_ref().and_then(|_| None);
            process = Some(child);
            if let Some(ref mut p) = process {
                stdin = p.stdin.take();
            }
        }

        Self { process, stdin }
    }
}

impl Default for UeberzugppBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UeberzugppBackend {
    fn drop(&mut self) {
        self.stdin.take();
        if let Some(mut proc) = self.process.take() {
            let _ = proc.kill();
        }
    }
}

impl BackgroundBackend for UeberzugppBackend {
    fn name(&self) -> &'static str {
        "ueberzugpp"
    }

    fn is_available(&self) -> bool {
        Command::new("which")
            .arg("ueberzugpp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn render(&mut self, image: &RgbaImage, opts: &RenderOpts) -> AuraResult<()> {
        let _ = image;

        let tmp_path = std::env::temp_dir().join("demidm_bg.png");
        let cmd = json!({
            "action": "add",
            "identifier": "background",
            "x": 0,
            "y": 0,
            "path": tmp_path.to_string_lossy(),
            "width": opts.screen_cols,
            "height": opts.screen_rows,
        });

        if let Some(ref mut stdin) = self.stdin {
            let line = format!("{}\n", serde_json::to_string(&cmd).unwrap_or_default());
            stdin
                .write_all(line.as_bytes())
                .map_err(|e| AuraError::Framebuffer(format!("ueberzugpp write error: {}", e)))?;
            stdin
                .flush()
                .map_err(|e| AuraError::Framebuffer(format!("ueberzugpp flush error: {}", e)))?;
        }
        Ok(())
    }

    fn clear(&mut self) -> AuraResult<()> {
        let cmd = json!({
            "action": "remove",
            "identifier": "background",
        });

        if let Some(ref mut stdin) = self.stdin {
            let line = format!("{}\n", serde_json::to_string(&cmd).unwrap_or_default());
            stdin
                .write_all(line.as_bytes())
                .map_err(|e| AuraError::Framebuffer(format!("ueberzugpp write error: {}", e)))?;
            stdin
                .flush()
                .map_err(|e| AuraError::Framebuffer(format!("ueberzugpp flush error: {}", e)))?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn format_add_command(path: &str, cols: u16, rows: u16) -> String {
    let cmd = json!({
        "action": "add",
        "identifier": "background",
        "x": 0,
        "y": 0,
        "path": path,
        "width": cols,
        "height": rows,
    });
    serde_json::to_string(&cmd).unwrap_or_default()
}

#[allow(dead_code)]
pub fn format_remove_command() -> String {
    let cmd = json!({
        "action": "remove",
        "identifier": "background",
    });
    serde_json::to_string(&cmd).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_add_command_format() {
        let cmd = format_add_command("/tmp/test.png", 80, 24);
        let parsed: serde_json::Value = serde_json::from_str(&cmd).unwrap();
        assert_eq!(parsed["action"], "add");
        assert_eq!(parsed["identifier"], "background");
        assert_eq!(parsed["x"], 0);
        assert_eq!(parsed["y"], 0);
        assert_eq!(parsed["path"], "/tmp/test.png");
        assert_eq!(parsed["width"], 80);
        assert_eq!(parsed["height"], 24);
    }

    #[test]
    fn test_json_remove_command_format() {
        let cmd = format_remove_command();
        let parsed: serde_json::Value = serde_json::from_str(&cmd).unwrap();
        assert_eq!(parsed["action"], "remove");
        assert_eq!(parsed["identifier"], "background");
    }
}
