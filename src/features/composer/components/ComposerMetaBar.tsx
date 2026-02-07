import type { CSSProperties } from "react";
import { BrainCog } from "lucide-react";
import type { AccessMode, RunnerConfig, ThreadTokenUsage } from "../../../types";
import { PopoverSelect } from "../../shared/components/PopoverSelect";

type ComposerMetaBarProps = {
  disabled: boolean;
  collaborationModes: { id: string; label: string }[];
  selectedCollaborationModeId: string | null;
  onSelectCollaborationMode: (id: string | null) => void;
  models: { id: string; displayName: string; model: string }[];
  selectedModelId: string | null;
  onSelectModel: (id: string) => void;
  runners: RunnerConfig[];
  selectedRunnerId: string | null;
  onSelectRunner: (id: string | null) => void;
  showRunnerSelect: boolean;
  reasoningOptions: string[];
  selectedEffort: string | null;
  onSelectEffort: (effort: string) => void;
  reasoningSupported: boolean;
  accessMode: AccessMode;
  onSelectAccessMode: (mode: AccessMode) => void;
  contextUsage?: ThreadTokenUsage | null;
};

export function ComposerMetaBar({
  disabled,
  collaborationModes,
  selectedCollaborationModeId,
  onSelectCollaborationMode,
  models,
  selectedModelId,
  onSelectModel,
  runners,
  selectedRunnerId,
  onSelectRunner,
  showRunnerSelect,
  reasoningOptions,
  selectedEffort,
  onSelectEffort,
  reasoningSupported,
  accessMode,
  onSelectAccessMode,
  contextUsage = null,
}: ComposerMetaBarProps) {
  const contextWindow = contextUsage?.modelContextWindow ?? null;
  const lastTokens = contextUsage?.last.totalTokens ?? 0;
  const totalTokens = contextUsage?.total.totalTokens ?? 0;
  const usedTokens = lastTokens > 0 ? lastTokens : totalTokens;
  const contextFreePercent =
    contextWindow && contextWindow > 0 && usedTokens > 0
      ? Math.max(
          0,
          100 -
            Math.min(Math.max((usedTokens / contextWindow) * 100, 0), 100),
        )
      : null;
  const planMode =
    collaborationModes.find((mode) => mode.id === "plan") ?? null;
  const defaultMode =
    collaborationModes.find((mode) => mode.id === "default") ?? null;
  const canUsePlanToggle =
    Boolean(planMode) &&
    collaborationModes.every(
      (mode) => mode.id === "default" || mode.id === "plan",
    );
  const planSelected = selectedCollaborationModeId === (planMode?.id ?? "");

  const collabOptions = collaborationModes.map((m) => ({
    value: m.id,
    label: m.label || m.id,
  }));

  const runnerOptions = runners.length > 0
    ? runners.map((r) => ({ value: r.id, label: r.name }))
    : [{ value: "", label: "No agents" }];

  const modelOptions = models.length > 0
    ? models.map((m) => ({ value: m.id, label: m.displayName || m.model }))
    : [{ value: "", label: "No models" }];

  const effortOptions = reasoningOptions.length > 0
    ? reasoningOptions.map((e) => ({ value: e, label: e }))
    : [{ value: "", label: "Default" }];

  const accessOptions = [
    { value: "read-only", label: "Read only" },
    { value: "current", label: "On-Request" },
    { value: "full-access", label: "Full access" },
  ];

  return (
    <div className="composer-bar">
      <div className="composer-meta">
        {collaborationModes.length > 0 && (
          canUsePlanToggle ? (
            <div className="composer-select-wrap composer-plan-toggle-wrap">
              <label className="composer-plan-toggle" aria-label="Plan mode">
                <input
                  className="composer-plan-toggle-input"
                  type="checkbox"
                  checked={planSelected}
                  disabled={disabled}
                  onChange={(event) =>
                    onSelectCollaborationMode(
                      event.target.checked
                        ? planMode?.id ?? "plan"
                        : (defaultMode?.id ?? null),
                    )
                  }
                />
                <span className="composer-plan-toggle-icon" aria-hidden>
                  <svg viewBox="0 0 24 24" fill="none">
                    <path
                      d="m6.5 7.5 1 1 2-2M6.5 12.5l1 1 2-2M6.5 17.5l1 1 2-2M11 7.5h7M11 12.5h7M11 17.5h7"
                      stroke="currentColor"
                      strokeWidth="1.4"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                </span>
                <span className="composer-plan-toggle-label">
                  {planMode?.label || "Plan"}
                </span>
              </label>
            </div>
          ) : (
            <div className="composer-select-wrap">
              <span className="composer-icon" aria-hidden>
                <svg viewBox="0 0 24 24" fill="none">
                  <path
                    d="m6.5 7.5 1 1 2-2M6.5 12.5l1 1 2-2M6.5 17.5l1 1 2-2M11 7.5h7M11 12.5h7M11 17.5h7"
                    stroke="currentColor"
                    strokeWidth="1.4"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              </span>
              <PopoverSelect
                options={collabOptions}
                value={selectedCollaborationModeId}
                onChange={(v) => onSelectCollaborationMode(v || null)}
                disabled={disabled}
                ariaLabel="Collaboration mode"
                className="composer-select--collab"
                width={78}
              />
            </div>
          )
        )}
        {showRunnerSelect && (
          <div className="composer-select-wrap composer-select-wrap--runner">
            <span className="composer-icon" aria-hidden>
              <svg viewBox="0 0 24 24" fill="none">
                <path
                  d="M6 7.5h12M6 12h8M6 16.5h10"
                  stroke="currentColor"
                  strokeWidth="1.4"
                  strokeLinecap="round"
                />
                <path
                  d="M19 12l2 2-2 2"
                  stroke="currentColor"
                  strokeWidth="1.4"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </span>
            <PopoverSelect
              options={runnerOptions}
              value={selectedRunnerId}
              onChange={(v) => onSelectRunner(v || null)}
              disabled={disabled || runners.length === 0}
              ariaLabel="Agent"
              className="composer-select--runner"
              placeholder="No agents"
              width={110}
            />
          </div>
        )}
        <div className="composer-select-wrap composer-select-wrap--model">
          <span className="composer-icon composer-icon--model" aria-hidden>
            <svg viewBox="0 0 24 24" fill="none">
              <path
                d="M12 4v2"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
              />
              <path
                d="M8 7.5h8a2.5 2.5 0 0 1 2.5 2.5v5a2.5 2.5 0 0 1-2.5 2.5H8A2.5 2.5 0 0 1 5.5 15v-5A2.5 2.5 0 0 1 8 7.5Z"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinejoin="round"
              />
              <circle cx="9.5" cy="12.5" r="1" fill="currentColor" />
              <circle cx="14.5" cy="12.5" r="1" fill="currentColor" />
              <path
                d="M9.5 15.5h5"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
              />
              <path
                d="M5.5 11H4M20 11h-1.5"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
              />
            </svg>
          </span>
          <PopoverSelect
            options={modelOptions}
            value={selectedModelId}
            onChange={onSelectModel}
            disabled={disabled}
            ariaLabel="Model"
            className="composer-select--model"
            placeholder="No models"
            width={110}
          />
        </div>
        <div className="composer-select-wrap composer-select-wrap--effort">
          <span className="composer-icon composer-icon--effort" aria-hidden>
            <BrainCog size={14} strokeWidth={1.8} />
          </span>
          <PopoverSelect
            options={effortOptions}
            value={selectedEffort}
            onChange={onSelectEffort}
            disabled={disabled || !reasoningSupported}
            ariaLabel="Thinking mode"
            className="composer-select--effort"
            width={80}
          />
        </div>
        <div className="composer-select-wrap">
          <span className="composer-icon" aria-hidden>
            <svg viewBox="0 0 24 24" fill="none">
              <path
                d="M12 4l7 3v5c0 4.5-3 7.5-7 8-4-0.5-7-3.5-7-8V7l7-3z"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinejoin="round"
              />
              <path
                d="M9.5 12.5l1.8 1.8 3.7-4"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </span>
          <PopoverSelect
            options={accessOptions}
            value={accessMode}
            onChange={(v) => onSelectAccessMode(v as AccessMode)}
            disabled={disabled}
            ariaLabel="Agent access"
            className="composer-select--approval"
            width={90}
          />
        </div>
      </div>
      <div className="composer-context">
        <div
          className="composer-context-ring"
          data-tooltip={
            contextFreePercent === null
              ? "Context free --"
              : `Context free ${Math.round(contextFreePercent)}%`
          }
          aria-label={
            contextFreePercent === null
              ? "Context free --"
              : `Context free ${Math.round(contextFreePercent)}%`
          }
          style={
            {
              "--context-free": contextFreePercent ?? 0,
            } as CSSProperties
          }
        >
          <span className="composer-context-value">●</span>
        </div>
      </div>
    </div>
  );
}
