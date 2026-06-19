use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use tokio::sync::mpsc::Sender;

use crate::errors::WidgetId;
use crate::events::AppEvent;
use crate::widget::{WidgetDef, WidgetSource};

const RUNNER_TICK: Duration = Duration::from_millis(100);

enum ExecutableSource {
    ShellCmd(String),
    StaticText(String),
    LuaFnPlaceholder,
}

pub async fn run_widget_runner(widgets: Arc<RwLock<Vec<WidgetDef>>>, tx: Sender<AppEvent>) {
    let mut last_exec: HashMap<WidgetId, Instant> = HashMap::new();
    let mut executed_once: HashSet<WidgetId> = HashSet::new();

    loop {
        tokio::time::sleep(RUNNER_TICK).await;

        let due = {
            let guard = match widgets.read() {
                Ok(g) => g,
                Err(_) => continue,
            };

            let now = Instant::now();
            let mut due_list = Vec::new();

            for def in guard.iter() {
                if def.refresh == Duration::ZERO {
                    if executed_once.contains(&def.id) {
                        continue;
                    }
                    executed_once.insert(def.id.clone());
                } else if let Some(last) = last_exec.get(&def.id) {
                    if now.duration_since(*last) < def.refresh {
                        continue;
                    }
                }

                let exec_source = match &def.source {
                    WidgetSource::ShellCmd(cmd) => ExecutableSource::ShellCmd(cmd.clone()),
                    WidgetSource::StaticText(text) => ExecutableSource::StaticText(text.clone()),
                    WidgetSource::LuaFn(_) => ExecutableSource::LuaFnPlaceholder,
                };

                due_list.push((def.id.clone(), exec_source));
                last_exec.insert(def.id.clone(), now);
            }

            due_list
        };

        for (id, source) in due {
            let tx = tx.clone();
            tokio::spawn(async move {
                let content = execute_widget_source(&source).await;
                let _ = tx.send(AppEvent::WidgetUpdate { id, content }).await;
            });
        }
    }
}

async fn execute_widget_source(source: &ExecutableSource) -> String {
    match source {
        ExecutableSource::ShellCmd(cmd) => match tokio::process::Command::new("sh")
            .args(["-c", cmd])
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_string();
                if output.status.success() {
                    stdout
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr)
                        .trim_end()
                        .to_string();
                    format!(
                        "[error: {}]",
                        if stderr.is_empty() { &stdout } else { &stderr }
                    )
                }
            }
            Err(e) => format!("[error: {}]", e),
        },
        ExecutableSource::StaticText(text) => text.clone(),
        ExecutableSource::LuaFnPlaceholder => "[lua_fn]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_static_text() {
        let source = ExecutableSource::StaticText("hello world".to_string());
        let result = execute_widget_source(&source).await;
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_execute_shell_cmd_echo() {
        let source = ExecutableSource::ShellCmd("echo hello".to_string());
        let result = execute_widget_source(&source).await;
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_execute_shell_cmd_error() {
        let source = ExecutableSource::ShellCmd("nonexistent_command_xyz".to_string());
        let result = execute_widget_source(&source).await;
        assert!(result.starts_with("[error:"));
    }

    #[tokio::test]
    async fn test_execute_lua_fn_placeholder() {
        let source = ExecutableSource::LuaFnPlaceholder;
        let result = execute_widget_source(&source).await;
        assert_eq!(result, "[lua_fn]");
    }

    fn test_widget_def(id: &str, refresh: Duration, source: WidgetSource) -> WidgetDef {
        use crate::errors::AbsolutePosition;
        use crate::widget::WidgetStyle;

        WidgetDef {
            id: id.to_string(),
            position: AbsolutePosition { col: 0, row: 0 },
            width: 10,
            height: None,
            refresh,
            source,
            style: WidgetStyle::default(),
        }
    }

    #[tokio::test]
    async fn test_failing_widget_does_not_block_sibling() {
        let widgets = Arc::new(RwLock::new(vec![
            test_widget_def(
                "bad",
                Duration::ZERO,
                WidgetSource::ShellCmd("nonexistent_command_xyz".to_string()),
            ),
            test_widget_def(
                "good",
                Duration::ZERO,
                WidgetSource::StaticText("ok".to_string()),
            ),
        ]));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(16);
        tokio::spawn(run_widget_runner(widgets, tx));

        let mut updates: HashMap<WidgetId, String> = HashMap::new();
        for _ in 0..2 {
            let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .expect("timed out waiting for widget update")
                .expect("channel closed unexpectedly");
            if let AppEvent::WidgetUpdate { id, content } = event {
                updates.insert(id, content);
            }
        }

        assert!(updates["bad"].starts_with("[error:"));
        assert_eq!(updates["good"], "ok");
    }

    #[tokio::test]
    async fn test_zero_refresh_widget_executes_exactly_once() {
        let widgets = Arc::new(RwLock::new(vec![test_widget_def(
            "w1",
            Duration::ZERO,
            WidgetSource::StaticText("hello".to_string()),
        )]));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(16);
        tokio::spawn(run_widget_runner(widgets, tx));

        // Wait several runner ticks (RUNNER_TICK = 100ms) to prove the widget
        // doesn't re-execute on subsequent ticks.
        tokio::time::sleep(Duration::from_millis(450)).await;

        let mut update_count = 0;
        while let Ok(AppEvent::WidgetUpdate { .. }) = rx.try_recv() {
            update_count += 1;
        }

        assert_eq!(update_count, 1);
    }
}
