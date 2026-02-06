import { useCallback, useEffect, useMemo, useState } from "react";
import type { RunnerConfig, WorkspaceInfo } from "../../../types";

const resolveDefaultRunnerId = (
  workspace: WorkspaceInfo | null,
  runners: RunnerConfig[],
): string | null => {
  if (!workspace) {
    return null;
  }
  const fromSettings = workspace.settings.runnerId ?? null;
  if (fromSettings && runners.some((runner) => runner.id === fromSettings)) {
    return fromSettings;
  }
  return runners[0]?.id ?? null;
};

type UseRunnerSelectionOptions = {
  activeWorkspace: WorkspaceInfo | null;
  runners: RunnerConfig[];
};

export function useRunnerSelection({
  activeWorkspace,
  runners,
}: UseRunnerSelectionOptions) {
  const [runnerIdByWorkspace, setRunnerIdByWorkspace] = useState<
    Record<string, string | null>
  >({});
  const activeWorkspaceId = activeWorkspace?.id ?? null;

  const defaultRunnerId = useMemo(
    () => resolveDefaultRunnerId(activeWorkspace, runners),
    [activeWorkspace, runners],
  );

  const selectedRunnerId = useMemo(() => {
    if (!activeWorkspaceId) {
      return null;
    }
    return runnerIdByWorkspace[activeWorkspaceId] ?? null;
  }, [activeWorkspaceId, runnerIdByWorkspace]);

  useEffect(() => {
    if (!activeWorkspaceId) {
      return;
    }
    setRunnerIdByWorkspace((prev) => {
      const current = prev[activeWorkspaceId];
      const hasCurrent =
        current !== null &&
        current !== undefined &&
        runners.some((runner) => runner.id === current);
      if (current !== undefined && (hasCurrent || runners.length === 0)) {
        return prev;
      }
      const next = defaultRunnerId;
      if (current === next) {
        return prev;
      }
      return {
        ...prev,
        [activeWorkspaceId]: next,
      };
    });
  }, [activeWorkspaceId, defaultRunnerId, runners]);

  const setSelectedRunnerId = useCallback(
    (next: string | null) => {
      if (!activeWorkspaceId) {
        return;
      }
      setRunnerIdByWorkspace((prev) => ({
        ...prev,
        [activeWorkspaceId]: next,
      }));
    },
    [activeWorkspaceId],
  );

  return {
    selectedRunnerId: selectedRunnerId ?? defaultRunnerId,
    setSelectedRunnerId,
  };
}
