// @vitest-environment jsdom
import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { RunnerConfig, WorkspaceInfo } from "../../../types";
import { useRunnerSelection } from "./useRunnerSelection";

const runners: RunnerConfig[] = [
  {
    id: "codex",
    name: "Codex",
    kind: "codex",
    transport: "app-server",
    defaultCommand: "codex",
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    kind: "gemini",
    transport: "acp",
    defaultCommand: "gemini",
  },
];

const makeWorkspace = (overrides: Partial<WorkspaceInfo> = {}): WorkspaceInfo => ({
  id: "ws-1",
  name: "Project",
  path: "/tmp/project",
  connected: true,
  settings: { sidebarCollapsed: false, runnerId: null },
  ...overrides,
});

describe("useRunnerSelection", () => {
  it("prefers workspace runner settings when available", () => {
    const workspace = makeWorkspace({
      settings: { sidebarCollapsed: false, runnerId: "gemini" },
    });

    const { result } = renderHook(() =>
      useRunnerSelection({ activeWorkspace: workspace, runners }),
    );

    expect(result.current.selectedRunnerId).toBe("gemini");
  });

  it("falls back to the first runner when settings are missing", () => {
    const workspace = makeWorkspace({
      settings: { sidebarCollapsed: false, runnerId: "missing" },
    });

    const { result } = renderHook(() =>
      useRunnerSelection({ activeWorkspace: workspace, runners }),
    );

    expect(result.current.selectedRunnerId).toBe("codex");
  });
});
