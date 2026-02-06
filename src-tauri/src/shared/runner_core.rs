use std::collections::HashMap;

use crate::types::{RunnerConfig, WorkspaceEntry};

pub(crate) struct ResolvedRunnerConfig {
    pub(crate) id: String,
    pub(crate) command: Option<String>,
    pub(crate) env: Option<HashMap<String, String>>,
}

fn pick_default_runner_id(runners: &[RunnerConfig]) -> Option<String> {
    runners
        .iter()
        .find(|runner| runner.id == "codex")
        .map(|runner| runner.id.clone())
        .or_else(|| runners.first().map(|runner| runner.id.clone()))
}

pub(crate) fn resolve_runner_config(
    runner_id: Option<String>,
    entry: &WorkspaceEntry,
    runners: &[RunnerConfig],
) -> ResolvedRunnerConfig {
    let requested = runner_id
        .or_else(|| entry.settings.runner_id.clone())
        .filter(|value| !value.trim().is_empty());

    let resolved_id = match requested {
        Some(id) if runners.iter().any(|runner| runner.id == id) => id,
        Some(id) if runners.is_empty() => id,
        _ => pick_default_runner_id(runners).unwrap_or_else(|| "codex".to_string()),
    };

    let command_from_settings = entry
        .settings
        .runner_command
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let command_from_runner = runners
        .iter()
        .find(|runner| runner.id == resolved_id)
        .map(|runner| runner.default_command.trim().to_string())
        .filter(|value| !value.is_empty());

    ResolvedRunnerConfig {
        id: resolved_id,
        command: command_from_settings.or(command_from_runner),
        env: entry.settings.runner_env.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_runner_config, ResolvedRunnerConfig};
    use crate::types::{RunnerConfig, WorkspaceEntry, WorkspaceKind, WorkspaceSettings};
    use std::collections::HashMap;

    fn runners() -> Vec<RunnerConfig> {
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

    fn entry_with_settings(settings: WorkspaceSettings) -> WorkspaceEntry {
        WorkspaceEntry {
            id: "ws-1".to_string(),
            name: "Workspace".to_string(),
            path: "/tmp".to_string(),
            codex_bin: None,
            kind: WorkspaceKind::Main,
            parent_id: None,
            worktree: None,
            settings,
        }
    }

    fn assert_resolved(config: ResolvedRunnerConfig, id: &str, command: Option<&str>) {
        assert_eq!(config.id, id);
        assert_eq!(config.command.as_deref(), command);
    }

    #[test]
    fn resolves_runner_from_explicit_param() {
        let entry = entry_with_settings(WorkspaceSettings::default());
        let config = resolve_runner_config(Some("gemini".to_string()), &entry, &runners());
        assert_resolved(config, "gemini", Some("gemini"));
    }

    #[test]
    fn resolves_runner_from_workspace_settings() {
        let mut settings = WorkspaceSettings::default();
        settings.runner_id = Some("gemini".to_string());
        let entry = entry_with_settings(settings);
        let config = resolve_runner_config(None, &entry, &runners());
        assert_resolved(config, "gemini", Some("gemini"));
    }

    #[test]
    fn resolves_each_known_runner_setting() {
        for (runner_id, command) in [
            ("codex", "codex"),
            ("gemini", "gemini"),
            ("claude", "claude"),
        ] {
            let mut settings = WorkspaceSettings::default();
            settings.runner_id = Some(runner_id.to_string());
            let entry = entry_with_settings(settings);
            let config = resolve_runner_config(None, &entry, &runners());
            assert_resolved(config, runner_id, Some(command));
        }
    }

    #[test]
    fn falls_back_to_default_runner() {
        let entry = entry_with_settings(WorkspaceSettings::default());
        let config = resolve_runner_config(Some("unknown".to_string()), &entry, &runners());
        assert_resolved(config, "codex", Some("codex"));
    }

    #[test]
    fn uses_requested_runner_when_registry_empty() {
        let entry = entry_with_settings(WorkspaceSettings::default());
        let config = resolve_runner_config(Some("gemini".to_string()), &entry, &[]);
        assert_resolved(config, "gemini", None);
    }

    #[test]
    fn defaults_to_codex_when_registry_empty() {
        let entry = entry_with_settings(WorkspaceSettings::default());
        let config = resolve_runner_config(None, &entry, &[]);
        assert_resolved(config, "codex", None);
    }

    #[test]
    fn prefers_explicit_command_override() {
        let mut settings = WorkspaceSettings::default();
        settings.runner_id = Some("gemini".to_string());
        settings.runner_command = Some("custom-gemini".to_string());
        let entry = entry_with_settings(settings);
        let config = resolve_runner_config(None, &entry, &runners());
        assert_resolved(config, "gemini", Some("custom-gemini"));
    }

    #[test]
    fn preserves_runner_env_overrides() {
        let mut settings = WorkspaceSettings::default();
        settings.runner_env = Some(HashMap::from([(
            "FOO".to_string(),
            "BAR".to_string(),
        )]));
        let entry = entry_with_settings(settings);
        let config = resolve_runner_config(None, &entry, &runners());
        assert_eq!(
            config.env.as_ref().and_then(|env| env.get("FOO")),
            Some(&"BAR".to_string())
        );
    }
}
