import { useCallback, useEffect, useState } from "react";
import { getPortalSnapshot } from "./api";
import type { PortalSnapshot } from "./types";

export function usePortal() {
  const [snapshot, setSnapshot] = useState<PortalSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const refresh = useCallback(async () => {
    setLoading(true); setError("");
    try { setSnapshot(await getPortalSnapshot()); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Portal data could not be loaded."); }
    finally { setLoading(false); }
  }, []);
  useEffect(() => {
    const task = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(task);
  }, [refresh]);
  return { snapshot, setSnapshot, loading, error, refresh };
}
