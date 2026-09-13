import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import {
  ensureDefaultBranch,
  listBranches,
  type LocalBranch,
} from "../lib/tauri/branches";
import { isDesktopRuntime } from "../lib/tauri/client";

export interface BranchState {
  branchId: string;
  branchName: string;
  branches: LocalBranch[];
  loading: boolean;
  error: string;
  select: (branchId: string) => void;
  refresh: () => void;
}

const BranchContext = createContext<BranchState | null>(null);

const fallback: LocalBranch = {
  id: "main",
  code: "MAIN",
  name: "Main Branch",
  phone: "",
  address: "",
  isActive: true,
  createdAt: "",
  updatedAt: "",
};

let inFlight: Promise<{ default_: LocalBranch; branches: LocalBranch[] }> | null =
  null;
function loadBranches() {
  if (!inFlight) {
    inFlight = Promise.all([ensureDefaultBranch(), listBranches()]).then(
      ([default_, branches]) => ({ default_, branches }),
    );
    inFlight.catch(() => {
      inFlight = null;
    });
  }
  return inFlight;
}

export function BranchProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState({
    loading: true,
    branches: [] as LocalBranch[],
    selected: "",
    error: "",
  });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!isDesktopRuntime()) return;
    let cancelled = false;
    loadBranches()
      .then(({ default_, branches }) => {
        if (cancelled) return;
        setState({ loading: false, branches, selected: default_.id, error: "" });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setState({
          loading: false,
          branches: [],
          selected: "",
          error:
            err instanceof Error
              ? err.message
              : "Local branches could not be loaded.",
        });
      });
    return () => {
      cancelled = true;
    };
  }, [attempt]);
  const select = useCallback((branchId: string) => {
    setState((prev) => ({ ...prev, selected: branchId }));
  }, []);
  const refresh = useCallback(() => setAttempt((v) => v + 1), []);
  const branchId =
    state.selected ||
    (state.branches.some((b) => b.id === state.selected)
      ? state.selected
      : state.branches.find((b) => b.isActive)?.id ?? fallback.id);
  const selected = state.branches.find((b) => b.id === branchId) ?? fallback;
  return (
    <BranchContext.Provider
      value={{
        branchId,
        branchName: selected.name,
        branches: state.branches,
        loading: state.loading,
        error: state.error,
        select,
        refresh,
      }}
    >
      {children}
    </BranchContext.Provider>
  );
}

export function useBranch(): BranchState {
  const ctx = useContext(BranchContext);
  if (!ctx) {
    throw new Error("useBranch must be used within a BranchProvider");
  }
  return ctx;
}