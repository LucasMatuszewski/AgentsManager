/**
 * Hardcoded model lists for non-Codex runners.
 *
 * ACP runners (Gemini CLI, Claude Code) don't expose a `model/list` endpoint,
 * so we maintain static model catalogs here. Update when new models ship.
 */

import type { ModelOption } from "../../types";

const geminiModels: ModelOption[] = [
  {
    id: "gemini-2.5-pro",
    model: "gemini-2.5-pro",
    displayName: "Gemini 2.5 Pro",
    description: "Most capable Gemini model with extended thinking",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Fast responses" },
      { reasoningEffort: "medium", description: "Balanced" },
      { reasoningEffort: "high", description: "Deep reasoning" },
    ],
    defaultReasoningEffort: "medium",
    isDefault: true,
  },
  {
    id: "gemini-2.5-flash",
    model: "gemini-2.5-flash",
    displayName: "Gemini 2.5 Flash",
    description: "Fast and efficient with thinking capabilities",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Fastest" },
      { reasoningEffort: "medium", description: "Balanced" },
      { reasoningEffort: "high", description: "More thorough" },
    ],
    defaultReasoningEffort: "low",
    isDefault: false,
  },
  {
    id: "gemini-2.0-flash",
    model: "gemini-2.0-flash",
    displayName: "Gemini 2.0 Flash",
    description: "Previous generation fast model",
    supportedReasoningEfforts: [],
    defaultReasoningEffort: null,
    isDefault: false,
  },
];

const claudeModels: ModelOption[] = [
  {
    id: "claude-sonnet-4-5-20250514",
    model: "claude-sonnet-4-5-20250514",
    displayName: "Claude Sonnet 4.5",
    description: "Best balance of speed and capability",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Quick responses" },
      { reasoningEffort: "medium", description: "Balanced thinking" },
      { reasoningEffort: "high", description: "Extended thinking" },
    ],
    defaultReasoningEffort: "medium",
    isDefault: true,
  },
  {
    id: "claude-opus-4-20250514",
    model: "claude-opus-4-20250514",
    displayName: "Claude Opus 4",
    description: "Most capable Claude model",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Quick responses" },
      { reasoningEffort: "medium", description: "Balanced thinking" },
      { reasoningEffort: "high", description: "Deep reasoning" },
    ],
    defaultReasoningEffort: "medium",
    isDefault: false,
  },
  {
    id: "claude-haiku-3-5-20241022",
    model: "claude-haiku-3-5-20241022",
    displayName: "Claude 3.5 Haiku",
    description: "Fastest Claude model",
    supportedReasoningEfforts: [],
    defaultReasoningEffort: null,
    isDefault: false,
  },
];

const runnerModelMap: Record<string, ModelOption[]> = {
  gemini: geminiModels,
  claude: claudeModels,
};

/**
 * Returns the static model list for a given runner kind.
 * Returns null if models should be fetched from the backend (e.g. Codex).
 */
export function getStaticRunnerModels(
  runnerId: string | null,
): ModelOption[] | null {
  if (!runnerId || runnerId === "codex") {
    return null; // Codex fetches from app-server
  }
  return runnerModelMap[runnerId] ?? null;
}
