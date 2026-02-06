use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;
use tracing::{debug, error, info, trace, warn};
use tracing_appender::rolling;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

static LOG_GUARD: OnceCell<WorkerGuard> = OnceCell::new();
static LOG_INITIALIZED: OnceCell<()> = OnceCell::new();

pub struct LogSettings {
    pub level: String,
    pub log_to_file: bool,
    pub log_to_stderr: bool,
    pub file_path: Option<PathBuf>,
}

impl LogSettings {
    pub fn new(level: String, log_to_file: bool, log_to_stderr: bool, file_path: Option<PathBuf>) -> Self {
        Self {
            level,
            log_to_file,
            log_to_stderr,
            file_path,
        }
    }
}

fn parse_env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_log_level(default: &str) -> String {
    std::env::var("AGENTSMANAGER_LOG_LEVEL").unwrap_or_else(|_| default.to_string())
}

fn build_filter(level: &str) -> EnvFilter {
    EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"))
}

fn init_logging(settings: LogSettings) {
    if LOG_INITIALIZED.get().is_some() {
        return;
    }

    let filter = build_filter(&settings.level);
    let mut registry = tracing_subscriber::registry();

    if settings.log_to_stderr {
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_filter(filter.clone());
        registry = registry.with(stderr_layer);
    }

    if settings.log_to_file {
        if let Some(file_path) = settings.file_path {
            if let Some(parent) = file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let file_appender = rolling::never(
                file_path
                    .parent()
                    .unwrap_or_else(|| Path::new(".")),
                file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("agentsmanager.log"),
            );
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(filter.clone());
            let _ = LOG_GUARD.set(guard);
            registry = registry.with(file_layer);
        }
    }

    registry.init();
    let _ = LOG_INITIALIZED.set(());
}

pub fn init_app_logging(app: &tauri::AppHandle) {
    let log_dir = app
        .path()
        .app_log_dir()
        .or_else(|_| app.path().app_data_dir())
        .ok();
    let file_path = log_dir.map(|dir| dir.join("agentsmanager.log"));
    let log_to_file = parse_env_bool("AGENTSMANAGER_LOG_FILE", true);
    let log_to_stderr = parse_env_bool("AGENTSMANAGER_LOG_STDERR", cfg!(debug_assertions));
    let level = env_log_level("info");

    init_logging(LogSettings::new(level, log_to_file, log_to_stderr, file_path));
    info!("logging initialized (app)");
}

pub fn init_daemon_logging(data_dir: &Path) {
    let log_dir = data_dir.join("logs");
    let file_path = Some(log_dir.join("daemon.log"));
    let log_to_file = parse_env_bool("AGENTSMANAGER_LOG_FILE", true);
    let log_to_stderr = parse_env_bool("AGENTSMANAGER_LOG_STDERR", true);
    let level = env_log_level("info");

    init_logging(LogSettings::new(level, log_to_file, log_to_stderr, file_path));
    info!("logging initialized (daemon)");
}

#[tauri::command]
pub fn log_client_event(
    level: String,
    label: String,
    payload: Option<serde_json::Value>,
    source: Option<String>,
) {
    let source = source.unwrap_or_else(|| "client".to_string());
    match level.as_str() {
        "error" => error!(source = %source, payload = ?payload, "{label}"),
        "warn" => warn!(source = %source, payload = ?payload, "{label}"),
        "debug" => debug!(source = %source, payload = ?payload, "{label}"),
        "trace" => trace!(source = %source, payload = ?payload, "{label}"),
        _ => info!(source = %source, payload = ?payload, "{label}"),
    }
}
