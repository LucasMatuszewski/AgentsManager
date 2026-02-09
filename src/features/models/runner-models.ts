/**
 * Hardcoded model lists for non-Codex runners.
 *
 * ACP runners (Gemini CLI, Claude Code) don't expose a `model/list` endpoint,
 * so we maintain static model catalogs here. Update when new models ship.
 */

import type { ModelOption } from "../../types";

const geminiModels: ModelOption[] = [
  {
    id: "gemini-3-pro-preview",
    model: "gemini-3-pro-preview",
    displayName: "Gemini 3 Pro",
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
    id: "gemini-3-flash-preview",
    model: "gemini-3-flash-preview",
    displayName: "Gemini 3 Flash",
    description: "Fast and efficient with thinking capabilities",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Fastest" },
      { reasoningEffort: "medium", description: "Balanced" },
      { reasoningEffort: "high", description: "More thorough" },
    ],
    defaultReasoningEffort: "low",
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
    id: "claude-opus-4-6-20250514",
    model: "claude-opus-4-6-20250514",
    displayName: "Claude Opus 4.6",
    description: "Latest and most capable Claude model",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Quick responses" },
      { reasoningEffort: "medium", description: "Balanced thinking" },
      { reasoningEffort: "high", description: "Deep reasoning" },
    ],
    defaultReasoningEffort: "medium",
    isDefault: false,
  },
  {
    id: "claude-opus-4-5-20250514",
    model: "claude-opus-4-5-20250514",
    displayName: "Claude Opus 4.5",
    description: "Highly capable Claude model",
    supportedReasoningEfforts: [
      { reasoningEffort: "low", description: "Quick responses" },
      { reasoningEffort: "medium", description: "Balanced thinking" },
      { reasoningEffort: "high", description: "Deep reasoning" },
    ],
    defaultReasoningEffort: "medium",
    isDefault: false,
  },
  {
    id: "claude-haiku-4-5-20250514",
    model: "claude-haiku-4-5-20250514",
    displayName: "Claude Haiku 4.5",
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
