import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import {
  listBranches,
  type LocalBranch,
} from "../lib/tauri/branches";
import { isDesktopRuntime, setNativeBranch } from "../lib/tauri/client";
import { useAuth } from "./AuthContext";

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

function loadBranches() {
  return listBranches();
}

export function BranchProvider({ children }: { children: ReactNode }) {
  const auth = useAuth();
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
      .then((branches) => {
        if (cancelled) return;
        const primary = auth.user?.branches.find((branch) => branch.isPrimary)?.branchId;
        setState({ loading: false, branches, selected: primary ?? branches[0]?.id ?? "", error: "" });
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
  }, [attempt, auth.user]);
  const select = useCallback((branchId: string) => {
    setState((prev) => ({ ...prev, selected: branchId }));
  }, []);
  const refresh = useCallback(() => setAttempt((v) => v + 1), []);
  const branchId = state.branches.some((b) => b.id === state.selected)
    ? state.selected
    : state.branches.find((b) => b.isActive)?.id ?? "";
  const selected = state.branches.find((b) => b.id === branchId) ?? fallback;
  useEffect(() => {
    setNativeBranch(state.branches.some((branch) => branch.id === branchId) ? branchId : "");
  }, [branchId, state.branches]);
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
