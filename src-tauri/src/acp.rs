use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::timeout;
use tracing::{debug, info};

use crate::backend::events::{AppServerEvent, EventSink};
use crate::shared::process_core::tokio_command;
use crate::types::WorkspaceEntry;

const ACP_PROTOCOL_VERSION: u32 = 1;
const ACP_INIT_TIMEOUT: Duration = Duration::from_secs(15);
const ACP_SESSION_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct AcpSession {
    pub(crate) entry: WorkspaceEntry,
    pub(crate) runner_id: String,
    pub(crate) child: Mutex<Child>,
    pub(crate) stdin: Mutex<ChildStdin>,
    pub(crate) pending: Mutex<HashMap<u64, oneshot::Sender<Value>>>,
    pub(crate) next_id: AtomicU64,
    pub(crate) session_id: Mutex<Option<String>>,
    pub(crate) active_turn_id: Mutex<Option<String>>,
    pub(crate) agent_message_buffers: Mutex<HashMap<String, String>>,
    pub(crate) background_thread_callbacks: Mutex<HashMap<String, mpsc::UnboundedSender<Value>>>,
}

impl AcpSession {
    async fn write_message(&self, value: Value) -> Result<(), String> {
        let mut stdin = self.stdin.lock().await;
        let mut line = serde_json::to_string(&value).map_err(|e| e.to_string())?;
        line.push('\n');
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| e.to_string())
    }

    pub(crate) async fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write_message(json!({ "id": id, "method": method, "params": params }))
            .await?;
        rx.await.map_err(|_| "request canceled".to_string())
    }

    pub(crate) async fn send_notification(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), String> {
        let value = if let Some(params) = params {
            json!({ "method": method, "params": params })
        } else {
            json!({ "method": method })
        };
        self.write_message(value).await
    }

    async fn append_agent_message_delta(&self, item_id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let mut buffers = self.agent_message_buffers.lock().await;
        let entry = buffers.entry(item_id.to_string()).or_default();
        entry.push_str(delta);
    }

    async fn take_agent_message_text(&self, item_id: &str) -> String {
        let mut buffers = self.agent_message_buffers.lock().await;
        buffers.remove(item_id).unwrap_or_default()
    }
}

fn build_initialize_params(client_version: &str) -> Value {
    json!({
        "protocolVersion": ACP_PROTOCOL_VERSION,
        "clientCapabilities": {},
        "clientInfo": {
            "name": "agents_manager",
            "title": "Agents Manager",
            "version": client_version
        }
    })
}

fn build_session_new_params(entry: &WorkspaceEntry) -> Value {
    json!({ "cwd": entry.path, "mcpServers": [] })
}

pub(crate) fn build_acp_command(
    runner_id: &str,
    runner_command: Option<String>,
) -> Command {
    let command = runner_command
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| runner_id.to_string());
    tokio_command(command)
}

fn apply_acp_runner_args(command: &mut Command, runner_id: &str) {
    if runner_id == "gemini" {
        command.arg("--experimental-acp");
    }
}

fn emit_app_event(event_sink: &impl EventSink, workspace_id: &str, message: Value) {
    event_sink.emit_app_server_event(AppServerEvent {
        workspace_id: workspace_id.to_string(),
        message,
    });
}

fn map_plan_entries(entries: &[Value]) -> Vec<Value> {
    entries
        .iter()
        .filter_map(|entry| {
            let record = entry.as_object()?;
            let content = record
                .get("content")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim();
            if content.is_empty() {
                return None;
            }
            let status = record
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("pending");
            Some(json!({
                "step": content,
                "status": status
            }))
        })
        .collect()
}

async fn handle_session_update(
    session: &AcpSession,
    workspace_id: &str,
    thread_id: &str,
    update: &Value,
    event_sink: &impl EventSink,
) {
    let update_type = update
        .get("sessionUpdate")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    match update_type {
        "agent_message_chunk" => {
            let content = update.get("content").and_then(|value| value.as_object());
            let delta = content
                .and_then(|record| record.get("text"))
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if delta.is_empty() {
                return;
            }
            let turn_id = session
                .active_turn_id
                .lock()
                .await
                .clone()
                .unwrap_or_else(|| format!("turn-{}", session.next_id.load(Ordering::SeqCst)));
            let item_id = format!("agent-{}", turn_id);
            session.append_agent_message_delta(&item_id, delta).await;
            emit_app_event(
                event_sink,
                workspace_id,
                json!({
                    "method": "item/agentMessage/delta",
                    "params": {
                        "threadId": thread_id,
                        "itemId": item_id,
                        "delta": delta
                    }
                }),
            );
        }
        "plan" => {
            let entries = update
                .get("entries")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            if entries.is_empty() {
                return;
            }
            let turn_id = session
                .active_turn_id
                .lock()
                .await
                .clone()
                .unwrap_or_else(|| format!("turn-{}", session.next_id.load(Ordering::SeqCst)));
            let mapped = map_plan_entries(&entries);
            emit_app_event(
                event_sink,
                workspace_id,
                json!({
                    "method": "turn/plan/updated",
                    "params": {
                        "threadId": thread_id,
                        "turnId": turn_id,
                        "plan": mapped
                    }
                }),
            );
        }
        "tool_call" => {
            debug!(
                thread_id = %thread_id,
                "acp tool_call update received (not yet mapped)"
            );
        }
        _ => {
            debug!(
                thread_id = %thread_id,
                update_type = %update_type,
                "acp update ignored"
            );
        }
    }
}

pub(crate) async fn spawn_acp_session(
    entry: WorkspaceEntry,
    runner_id: String,
    runner_command: Option<String>,
    runner_env: Option<HashMap<String, String>>,
    client_version: String,
    event_sink: impl EventSink,
) -> Result<Arc<AcpSession>, String> {
    let mut command = build_acp_command(&runner_id, runner_command.clone());
    apply_acp_runner_args(&mut command, &runner_id);
    if let Some(env) = runner_env {
        for (key, value) in env {
            command.env(key, value);
        }
    }
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    info!(
        workspace_id = %entry.id,
        runner_id = %runner_id,
        runner_command = ?runner_command,
        "spawning ACP runner"
    );

    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let stdin = child.stdin.take().ok_or("missing stdin")?;
    let stdout = child.stdout.take().ok_or("missing stdout")?;
    let stderr = child.stderr.take().ok_or("missing stderr")?;

    let session = Arc::new(AcpSession {
        entry: entry.clone(),
        runner_id: runner_id.clone(),
        child: Mutex::new(child),
        stdin: Mutex::new(stdin),
        pending: Mutex::new(HashMap::new()),
        next_id: AtomicU64::new(1),
        session_id: Mutex::new(None),
        active_turn_id: Mutex::new(None),
        agent_message_buffers: Mutex::new(HashMap::new()),
        background_thread_callbacks: Mutex::new(HashMap::new()),
    });

    let session_clone = Arc::clone(&session);
    let workspace_id = entry.id.clone();
    let event_sink_clone = event_sink.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            let value: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(err) => {
                    emit_app_event(
                        &event_sink_clone,
                        &workspace_id,
                        json!({
                            "method": "acp/parseError",
                            "params": { "error": err.to_string(), "raw": line },
                        }),
                    );
                    continue;
                }
            };

            let maybe_id = value.get("id").and_then(|id| id.as_u64());
            let has_method = value.get("method").is_some();
            let has_result_or_error = value.get("result").is_some() || value.get("error").is_some();

            if let Some(id) = maybe_id {
                if has_result_or_error {
                    if let Some(tx) = session_clone.pending.lock().await.remove(&id) {
                        let _ = tx.send(value);
                    }
                } else if has_method {
                    let payload = AppServerEvent {
                        workspace_id: workspace_id.clone(),
                        message: value,
                    };
                    event_sink_clone.emit_app_server_event(payload);
                } else if let Some(tx) = session_clone.pending.lock().await.remove(&id) {
                    let _ = tx.send(value);
                }
            } else if has_method {
                if value
                    .get("method")
                    .and_then(|method| method.as_str())
                    == Some("session/update")
                {
                    let params = value.get("params").cloned().unwrap_or(json!({}));
                    let thread_id = params
                        .get("sessionId")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let update = params.get("update").cloned().unwrap_or(json!({}));
                    handle_session_update(&session_clone, &workspace_id, thread_id, &update, &event_sink_clone)
                        .await;
                } else {
                    let payload = AppServerEvent {
                        workspace_id: workspace_id.clone(),
                        message: value,
                    };
                    event_sink_clone.emit_app_server_event(payload);
                }
            }
        }
    });

    let workspace_id = entry.id.clone();
    let event_sink_clone = event_sink.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            emit_app_event(
                &event_sink_clone,
                &workspace_id,
                json!({
                    "method": "acp/stderr",
                    "params": { "message": line },
                }),
            );
        }
    });

    let init_params = build_initialize_params(&client_version);
    let init_result = timeout(
        ACP_INIT_TIMEOUT,
        session.send_request("initialize", init_params),
    )
    .await;
    let init_response = match init_result {
        Ok(response) => response,
        Err(_) => {
            let mut child = session.child.lock().await;
            let _ = child.kill().await;
            return Err(format!(
                "ACP runner '{}' did not respond to initialize within {}s. \
                 Ensure the runner binary is installed and supports ACP protocol. \
                 For Claude Code, install: npm install -g @zed-industries/claude-code-acp. \
                 For Gemini CLI, ensure 'gemini' is in PATH. \
                 Check app logs for details.",
                runner_id,
                ACP_INIT_TIMEOUT.as_secs()
            ));
        }
    };
    init_response?;

    let session_result = timeout(
        ACP_SESSION_TIMEOUT,
        session.send_request("session/new", build_session_new_params(&entry)),
    )
    .await
    .map_err(|_| "ACP runner did not respond to session/new".to_string())??;
    let session_id = session_result
        .get("result")
        .and_then(|value| value.get("sessionId"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| "ACP session/new missing sessionId".to_string())?
        .to_string();

    {
        let mut session_lock = session.session_id.lock().await;
        *session_lock = Some(session_id.clone());
    }

    emit_app_event(
        &event_sink,
        &entry.id,
        json!({
            "method": "thread/started",
            "params": {
                "thread": {
                    "id": session_id,
                    "cwd": entry.path,
                    "preview": "New ACP session",
                    "createdAt": chrono::Utc::now().timestamp_millis()
                }
            }
        }),
    );

    Ok(session)
}

pub(crate) async fn start_prompt_turn(
    session: &Arc<AcpSession>,
    event_sink: impl EventSink,
    text: String,
    images: Option<Vec<String>>,
) -> Result<Value, String> {
    let session_id = session
        .session_id
        .lock()
        .await
        .clone()
        .ok_or_else(|| "ACP session not initialized".to_string())?;

    let turn_id = format!("turn-{}", uuid::Uuid::new_v4());
    {
        let mut turn_lock = session.active_turn_id.lock().await;
        *turn_lock = Some(turn_id.clone());
    }

    emit_app_event(
        &event_sink,
        &session.entry.id,
        json!({
            "method": "turn/started",
            "params": {
                "threadId": session_id,
                "turn": { "id": turn_id, "threadId": session_id }
            }
        }),
    );

    let mut content_blocks = vec![json!({ "type": "text", "text": text.clone() })];
    if let Some(paths) = images {
        for path in paths {
            content_blocks.push(json!({
                "type": "resource",
                "resource": {
                    "uri": format!("file://{}", path),
                    "mimeType": "image/png"
                }
            }));
        }
    }

    let user_item_id = format!("user-{}", turn_id);
    emit_app_event(
        &event_sink,
        &session.entry.id,
        json!({
            "method": "item/started",
            "params": {
                "threadId": session_id,
                "item": {
                    "id": user_item_id,
                    "type": "userMessage",
                    "content": content_blocks
                }
            }
        }),
    );

    emit_app_event(
        &event_sink,
        &session.entry.id,
        json!({
            "method": "item/completed",
            "params": {
                "threadId": session_id,
                "item": {
                    "id": user_item_id,
                    "type": "userMessage",
                    "content": content_blocks
                }
            }
        }),
    );

    let params = json!({
        "sessionId": session_id,
        "prompt": content_blocks
    });

    let response = session.send_request("session/prompt", params).await?;

    let agent_item_id = format!("agent-{}", turn_id);
    let agent_text = session.take_agent_message_text(&agent_item_id).await;
    emit_app_event(
        &event_sink,
        &session.entry.id,
        json!({
            "method": "item/completed",
            "params": {
                "threadId": session_id,
                "item": {
                    "id": agent_item_id,
                    "type": "agentMessage",
                    "text": agent_text
                }
            }
        }),
    );

    emit_app_event(
        &event_sink,
        &session.entry.id,
        json!({
            "method": "turn/completed",
            "params": {
                "threadId": session_id,
                "turn": { "id": turn_id, "threadId": session_id }
            }
        }),
    );

    Ok(response)
}

pub(crate) async fn list_acp_threads(
    thread_index: &Mutex<HashMap<String, Vec<Value>>>,
    workspace_id: &str,
) -> Result<Value, String> {
    let map = thread_index.lock().await;
    let list = map.get(workspace_id).cloned().unwrap_or_default();
    Ok(json!({ "data": list, "nextCursor": null }))
}

pub(crate) async fn cancel_prompt_turn(session: &Arc<AcpSession>) -> Result<(), String> {
    let session_id = session
        .session_id
        .lock()
        .await
        .clone()
        .ok_or_else(|| "ACP session not initialized".to_string())?;
    session
        .send_notification("session/cancel", Some(json!({ "sessionId": session_id })))
        .await
}

pub(crate) async fn register_acp_thread(
    thread_index: &Mutex<HashMap<String, Vec<Value>>>,
    workspace_id: &str,
    thread: Value,
) {
    let mut map = thread_index.lock().await;
    let entry = map.entry(workspace_id.to_string()).or_default();
    entry.push(thread);
}

pub(crate) async fn remove_acp_thread(
    thread_index: &Mutex<HashMap<String, Vec<Value>>>,
    workspace_id: &str,
    thread_id: &str,
) {
    let mut map = thread_index.lock().await;
    if let Some(list) = map.get_mut(workspace_id) {
        list.retain(|thread| {
            thread
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                != thread_id
        });
    }
}

pub(crate) async fn update_acp_thread_name(
    thread_index: &Mutex<HashMap<String, Vec<Value>>>,
    workspace_id: &str,
    thread_id: &str,
    name: &str,
) {
    let mut map = thread_index.lock().await;
    if let Some(list) = map.get_mut(workspace_id) {
        for thread in list.iter_mut() {
            if thread
                .get("id")
                .and_then(|value| value.as_str())
                == Some(thread_id)
            {
                if let Some(thread_obj) = thread.as_object_mut() {
                    thread_obj.insert("preview".to_string(), json!(name));
                    thread_obj.insert(
                        "updatedAt".to_string(),
                        json!(chrono::Utc::now().timestamp_millis()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        list_acp_threads, register_acp_thread, remove_acp_thread, spawn_acp_session,
        update_acp_thread_name,
    };
    use crate::backend::events::{AppServerEvent, EventSink, TerminalExit, TerminalOutput};
    use crate::types::{WorkspaceEntry, WorkspaceKind, WorkspaceSettings};
    use serde_json::json;
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex;

    #[derive(Clone, Default)]
    struct NoopEventSink;

    impl EventSink for NoopEventSink {
        fn emit_app_server_event(&self, _event: AppServerEvent) {}

        fn emit_terminal_output(&self, _event: TerminalOutput) {}

        fn emit_terminal_exit(&self, _event: TerminalExit) {}
    }

    #[test]
    fn register_and_list_threads() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let store: Mutex<HashMap<String, Vec<serde_json::Value>>> =
                Mutex::new(HashMap::new());
            register_acp_thread(
                &store,
                "ws-1",
                json!({ "id": "thread-1", "preview": "Hello" }),
            )
            .await;

            let result = list_acp_threads(&store, "ws-1").await.unwrap();
            let data = result.get("data").and_then(|value| value.as_array()).unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(
                data[0].get("id").and_then(|v| v.as_str()),
                Some("thread-1")
            );
        });
    }

    #[test]
    fn update_thread_name() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let store: Mutex<HashMap<String, Vec<serde_json::Value>>> =
                Mutex::new(HashMap::new());
            register_acp_thread(
                &store,
                "ws-1",
                json!({ "id": "thread-1", "preview": "Hello" }),
            )
            .await;

            update_acp_thread_name(&store, "ws-1", "thread-1", "New name").await;

            let result = list_acp_threads(&store, "ws-1").await.unwrap();
            let data = result.get("data").and_then(|value| value.as_array()).unwrap();
            assert_eq!(
                data[0].get("preview").and_then(|v| v.as_str()),
                Some("New name")
            );
        });
    }

    #[test]
    fn remove_thread() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let store: Mutex<HashMap<String, Vec<serde_json::Value>>> =
                Mutex::new(HashMap::new());
            register_acp_thread(
                &store,
                "ws-1",
                json!({ "id": "thread-1", "preview": "Hello" }),
            )
            .await;
            remove_acp_thread(&store, "ws-1", "thread-1").await;

            let result = list_acp_threads(&store, "ws-1").await.unwrap();
            let data = result.get("data").and_then(|value| value.as_array()).unwrap();
            assert!(data.is_empty());
        });
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn spawn_acp_session_with_mock_runner() {
        use std::os::unix::fs::PermissionsExt;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let script_path = env::temp_dir().join(format!(
            "acp_mock_runner_{}_{}.py",
            std::process::id(),
            now
        ));
        let script = r#"#!/usr/bin/env python3
import sys
import json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        message = json.loads(line)
    except Exception:
        continue
    if "id" not in message:
        continue
    method = message.get("method", "")
    if method == "initialize":
        result = {}
    elif method == "session/new":
        result = {"sessionId": "test-session"}
    else:
        result = {}
    response = {"id": message["id"], "result": result}
    sys.stdout.write(json.dumps(response) + "\n")
    sys.stdout.flush()
"#;
        fs::write(&script_path, script).expect("write script");
        let mut permissions = fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&script_path, permissions).expect("chmod");

        let entry = WorkspaceEntry {
            id: "ws-1".to_string(),
            name: "Workspace".to_string(),
            path: env::temp_dir().to_string_lossy().to_string(),
            codex_bin: None,
            kind: WorkspaceKind::Main,
            parent_id: None,
            worktree: None,
            settings: WorkspaceSettings {
                runner_id: Some("gemini".to_string()),
                runner_command: Some(script_path.to_string_lossy().to_string()),
                ..WorkspaceSettings::default()
            },
        };

        let session = spawn_acp_session(
            entry,
            "gemini".to_string(),
            Some(script_path.to_string_lossy().to_string()),
            None,
            "test-client".to_string(),
            NoopEventSink::default(),
        )
        .await
        .expect("spawn session");

        let session_id = session.session_id.lock().await.clone();
        assert_eq!(session_id.as_deref(), Some("test-session"));

        let mut child = session.child.lock().await;
        let _ = child.kill().await;
        let _ = child.wait().await;

        fs::remove_file(&script_path).expect("cleanup script");
    }
}
