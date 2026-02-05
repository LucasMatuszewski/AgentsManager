use std::collections::HashMap;

use crate::types::{WorkspaceEntry, WorkspaceSettings};
#[cfg(test)]
use crate::types::WorkspaceInfo;

#[cfg(test)]
pub(crate) fn sort_workspaces(list: &mut Vec<WorkspaceInfo>) {
    list.sort_by(|a, b| {
        let a_order = a.settings.sort_order.unwrap_or(u32::MAX);
        let b_order = b.settings.sort_order.unwrap_or(u32::MAX);
        a_order
            .cmp(&b_order)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.cmp(&b.id))
    });
}

pub(crate) fn apply_workspace_settings_update(
    workspaces: &mut HashMap<String, WorkspaceEntry>,
    id: &str,
    settings: WorkspaceSettings,
) -> Result<WorkspaceEntry, String> {
    match workspaces.get_mut(id) {
        Some(entry) => {
            let mut next = settings.clone();
            if next.runner_id.is_none() {
                next.runner_id = entry.settings.runner_id.clone();
            }
            if next.runner_command.is_none() {
                next.runner_command = entry.settings.runner_command.clone();
            }
            if next.runner_env.is_none() {
                next.runner_env = entry.settings.runner_env.clone();
            }
            entry.settings = next;
            Ok(entry.clone())
        }
        None => Err("workspace not found".to_string()),
    }
}
