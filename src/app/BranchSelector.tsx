import { RefreshCw } from "lucide-react";
import { IconButton, Select } from "../design-system";
import { isDesktopRuntime } from "../lib/tauri/client";
import { useBranch } from "./BranchContext";
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
  const { branchId, branchName, branches, loading, error, select, refresh } =
    useBranch();
  return (
    <div className="native-branch-selector" aria-busy={loading}>
      <Select
        label="Branch"
        title={error || "Branches from local SQLite · inventory is branch-scoped"}
        disabled={loading || !!error || !branches.length}
        value={branchId}
        onChange={(e) => select(e.target.value)}
      >
        {loading ? (
          <option value="">Loading local branches…</option>
        ) : error ? (
          <option value="">Branch unavailable</option>
        ) : (
          branches.map((b) => (
            <option key={b.id} value={b.id} disabled={!b.isActive}>
              {b.name}
              {b.isActive ? "" : " (inactive)"}
            </option>
          ))
        )}
      </Select>
      {error && (
        <>
          <span role="alert" className="sr-only">
            {error}
          </span>
          <IconButton
            label="Retry local branch connection"
            title={error}
            onClick={refresh}
          >
            <RefreshCw size={15} />
          </IconButton>
        </>
      )}
      {!error && !loading && branchName && (
        <span className="text-dim" role="status">
          {branchName}
        </span>
      )}
    </div>
  );
}