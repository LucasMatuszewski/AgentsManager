import { useCallback, useEffect, useRef, useState } from "react";
import type { DebugEntry, RunnerConfig } from "../../../types";
import { listRunners } from "../../../services/tauri";

const isMissingInvoke = (error: unknown) =>
  error instanceof TypeError &&
  (error.message.includes("reading 'invoke'") ||
    error.message.includes("reading \"invoke\""));

type UseRunnersOptions = {
  onDebug?: (entry: DebugEntry) => void;
};

export function useRunners({ onDebug }: UseRunnersOptions = {}) {
  const [runners, setRunners] = useState<RunnerConfig[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inFlightRef = useRef(false);

  const refreshRunners = useCallback(async () => {
    if (inFlightRef.current) {
      return;
    }
    inFlightRef.current = true;
    setIsLoading(true);
    setError(null);
    try {
      const data = await listRunners();
      setRunners(data);
    } catch (err) {
      if (isMissingInvoke(err)) {
        console.warn("Tauri invoke bridge unavailable; returning empty runner list.");
        setRunners([]);
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      onDebug?.({
        id: `${Date.now()}-client-list-runners-error`,
        timestamp: Date.now(),
        source: "error",
        label: "list_runners error",
        payload: message,
      });
    } finally {
      inFlightRef.current = false;
      setIsLoading(false);
    }
  }, [onDebug]);

  useEffect(() => {
    void refreshRunners();
  }, [refreshRunners]);

  return {
    runners,
    isLoading,
    error,
    refreshRunners,
  };
}
