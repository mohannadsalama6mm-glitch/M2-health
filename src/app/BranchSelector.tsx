import { useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import { IconButton, Select } from "../design-system";
import { isDesktopRuntime } from "../lib/tauri/client";
import {
  ensureDefaultBranch,
  listBranches,
  type LocalBranch,
} from "../lib/tauri/branches";
import { useDemo } from "./DemoContext";
export function BranchSelector() {
  return isDesktopRuntime() ? <LocalBranchSelector /> : <DemoBranchSelector />;
}
function DemoBranchSelector() {
  const { branch, setBranch, branches, notify } = useDemo();
  return (
    <Select
      label="Branch"
      value={branch}
      onChange={(e) => {
        setBranch(e.target.value);
        notify(
          `Demo branch changed to ${e.target.value}. Sample figures are illustrative.`,
        );
      }}
    >
      {branches.map((b) => (
        <option key={b.id}>{b.name}</option>
      ))}
    </Select>
  );
}
function LocalBranchSelector() {
  const [state, setState] = useState<{
    loading: boolean;
    branches: LocalBranch[];
    selected: string;
    error: string;
  }>({ loading: true, branches: [], selected: "", error: "" });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    async function load() {
      try {
        const initial = await ensureDefaultBranch();
        const branches = await listBranches();
        if (active)
          setState({
            loading: false,
            branches,
            selected: initial.id,
            error: "",
          });
      } catch (error: unknown) {
        if (active)
          setState({
            loading: false,
            branches: [],
            selected: "",
            error:
              error instanceof Error
                ? error.message
                : "Local branches could not be loaded.",
          });
      }
    }
    void load();
    return () => {
      active = false;
    };
  }, [attempt]);
  return (
    <div className="native-branch-selector" aria-busy={state.loading}>
      <Select
        label="Branch"
        title={
          state.error ||
          "Branches from local SQLite · other screens still use demo data"
        }
        disabled={state.loading || !!state.error || !state.branches.length}
        value={state.selected}
        onChange={(e) => setState({ ...state, selected: e.target.value })}
      >
        {state.loading ? (
          <option value="">Loading local branches…</option>
        ) : state.error ? (
          <option value="">Branch unavailable</option>
        ) : (
          state.branches.map((b) => (
            <option key={b.id} value={b.id} disabled={!b.isActive}>
              {b.name}
              {b.isActive ? "" : " (inactive)"}
            </option>
          ))
        )}
      </Select>
      {state.error && (
        <>
          <span role="alert" className="sr-only">
            {state.error}
          </span>
          <IconButton
            label="Retry local branch connection"
            title={state.error}
            onClick={() => {
              setState({
                loading: true,
                branches: [],
                selected: "",
                error: "",
              });
              setAttempt((v) => v + 1);
            }}
          >
            <RefreshCw size={15} />
          </IconButton>
        </>
      )}
    </div>
  );
}
