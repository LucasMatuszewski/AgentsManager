use crate::types::RunnerConfig;

#[tauri::command]
pub(crate) fn list_runners() -> Vec<RunnerConfig> {
    vec![
        RunnerConfig {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            kind: "codex".to_string(),
            transport: "app-server".to_string(),
            default_command: "codex".to_string(),
        },
        RunnerConfig {
            id: "gemini".to_string(),
            name: "Gemini CLI".to_string(),
            kind: "gemini".to_string(),
            transport: "acp".to_string(),
            default_command: "gemini".to_string(),
        },
        RunnerConfig {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            kind: "claude".to_string(),
            transport: "acp".to_string(),
            default_command: "claude".to_string(),
        },
    ]
}
