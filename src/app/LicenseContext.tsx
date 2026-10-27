import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { isDesktopRuntime } from "../lib/tauri/client";
import {
  activateManualLicense,
  getLicenseStatus,
  revalidateManualLicense,
  startLicenseTrial,
  type LicenseStatus,
} from "../lib/tauri/licensing";

interface LicenseContextValue {
  status: LicenseStatus | null;
  loading: boolean;
  error: string;
  startTrial(): Promise<void>;
  activate(code: string): Promise<void>;
  revalidate(code: string): Promise<void>;
  refresh(): Promise<void>;
}

const Context = createContext<LicenseContextValue | null>(null);
const errorText = (error: unknown) =>
  error instanceof Error ? error.message : "License verification could not be completed.";

const previewStatus: LicenseStatus = {
  state: "ACTIVE",
  deviceId: "browser-preview",
  activatedAt: null,
  expiresAt: null,
  lastValidatedAt: null,
  nextRevalidationAt: null,
  offlineGraceEndsAt: null,
  daysRemaining: null,
  canOperate: true,
  readOnly: false,
  requiresActivation: false,
  reason: "Development browser preview.",
};

export function LicenseProvider({ children }: { children: ReactNode }) {
  const desktop = isDesktopRuntime();
  const [status, setStatus] = useState<LicenseStatus | null>(
    desktop ? null : previewStatus,
  );
  const [loading, setLoading] = useState(desktop);
  const [error, setError] = useState("");
  const refresh = useCallback(async () => {
    if (!desktop) return;
    setLoading(true);
    setError("");
    try {
      setStatus(await getLicenseStatus());
    } catch (reason) {
      setError(errorText(reason));
    } finally {
      setLoading(false);
    }
  }, [desktop]);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    getLicenseStatus()
      .then((value) => {
        if (active) setStatus(value);
      })
      .catch((reason) => {
        if (active) setError(errorText(reason));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [desktop]);
  const action = useCallback(
    async (work: () => Promise<LicenseStatus>) => {
      setLoading(true);
      setError("");
      try {
        setStatus(await work());
      } catch (reason) {
        setError(errorText(reason));
        throw reason;
      } finally {
        setLoading(false);
      }
    },
    [],
  );
  const value = useMemo<LicenseContextValue>(
    () => ({
      status,
      loading,
      error,
      refresh,
      startTrial: () => action(startLicenseTrial),
      activate: (code) => action(() => activateManualLicense(code)),
      revalidate: (code) => action(() => revalidateManualLicense(code)),
    }),
    [status, loading, error, refresh, action],
  );
  return <Context.Provider value={value}>{children}</Context.Provider>;
}

export function useLicense() {
  const value = useContext(Context);
  if (!value) throw new Error("LicenseProvider is required");
  return value;
}
