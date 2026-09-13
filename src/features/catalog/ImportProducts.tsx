import { useEffect, useRef, useState } from "react";
import {
  Alert,
  Button,
  Card,
  ConfirmationModal,
  LinkButton,
  PageHeader,
  SummaryStrip,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import {
  applyCatalogImport,
  dryRunCatalogImport,
  getCatalogImportReport,
  getCatalogImportStatus,
  profileCatalogSource,
  type ImportProfile,
  type ImportReport,
} from "../../lib/tauri/catalog-import";
const labels: Record<string, string> = {
  ready: "Ready",
  exact_duplicate_source: "Exact duplicates",
  possible_duplicate: "Possible duplicates",
  already_imported: "Already imported",
  missing_scientific_name: "Missing scientific names",
  missing_manufacturer: "Missing manufacturers",
  missing_class: "Missing classes",
  unknown_route: "UNKNOWN routes",
  ambiguous_price: "Ambiguous prices",
  invalid_price: "Invalid prices",
  invalid_required_data: "Invalid required data",
  review_required: "Review required",
};
export function ImportProducts() {
  const [profile, setProfile] = useState<ImportProfile | null>(null),
    [report, setReport] = useState<ImportReport | null>(null),
    [busy, setBusy] = useState("Loading source"),
    [error, setError] = useState(""),
    [confirm, setConfirm] = useState(false),
    [canApply, setCanApply] = useState(false);
  const gate = useRef(false);
  const desktop = isDesktopRuntime();
  useEffect(() => {
    if (!desktop) return;
    let live = true;
    Promise.all([profileCatalogSource(), getCatalogImportReport()])
      .then(([p, r]) => {
        if (live) {
          setProfile(p);
          setReport(r);
        }
      })
      .catch((e) => {
        if (live) setError(e.message);
      })
      .finally(() => {
        if (live) setBusy("");
      });
    return () => {
      live = false;
    };
  }, [desktop]);
  useEffect(() => {
    if (report?.status !== "applying") return;
    let live = true;
    let polling = false;
    const timer = setInterval(async () => {
      if (polling) return;
      polling = true;
      try {
        const r = await getCatalogImportStatus();
        if (live && r) {
          setReport(r);
          if (r.status !== "applying") {
            setBusy("");
            gate.current = false;
          }
        }
      } catch (e) {
        if (live)
          setError(
            e instanceof Error ? e.message : "Cannot check import status.",
          );
      } finally {
        polling = false;
      }
    }, 500);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [report?.status]);
  async function dry() {
    if (gate.current) return;
    gate.current = true;
    setCanApply(false);
    setError("");
    setBusy("Validating all source rows");
    try {
      const r = await dryRunCatalogImport();
      setReport(r);
      setProfile(r.profile);
      setCanApply(r.status === "validated" && (r.counts.ready ?? 0) > 0);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Dry run failed.");
    } finally {
      gate.current = false;
      setBusy("");
    }
  }
  async function apply() {
    if (gate.current || !canApply || !report) return;
    gate.current = true;
    setCanApply(false);
    setError("");
    setBusy("Importing");
    try {
      setReport(await applyCatalogImport(report.id));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Apply failed.");
      setBusy("");
      gate.current = false;
    }
  }
  return (
    <div className="feature-page">
      <PageHeader
        title="Import Products"
        description="Review the Egyptian Drug Catalog before adding it to your local Catalog."
        actions={<LinkButton to="/catalog">Back to Products</LinkButton>}
      />
      {!desktop ? (
        <Alert title="Desktop app required" tone="warning">
          Open M² Health desktop to import the canonical catalog.
        </Alert>
      ) : (
        <>
          {error && (
            <Alert title="Import could not continue" tone="danger">
              {error}
            </Alert>
          )}
          <Card>
            <h2>Egyptian Drug Catalog</h2>
            {profile ? (
              <div className="stack">
                <p>
                  {profile.totalRows.toLocaleString()} source rows ·{" "}
                  {(profile.size / 1024 / 1024).toFixed(2)} MB
                </p>
                <p>
                  Last profile:{" "}
                  {new Date(Number(profile.profiledAt)).toLocaleString()}
                </p>
                <p style={{ overflowWrap: "anywhere" }}>
                  Source: {profile.path}
                  <br />
                  SHA-256: {profile.hash}
                </p>
              </div>
            ) : (
              <p>{busy || "Source profile unavailable. Retry the dry run."}</p>
            )}
            <p>
              Scientific text is preserved. Package sizes and ingredient links
              remain unspecified. Review-required rows are skipped; no barcodes
              are generated.
            </p>
            <div className="row">
              <Button
                onClick={dry}
                disabled={!!busy}
                loading={busy === "Validating all source rows"}
              >
                Dry Run
              </Button>
              <Button
                variant="primary"
                disabled={!canApply || !!busy}
                onClick={() => setConfirm(true)}
              >
                Apply Import
              </Button>
            </div>
          </Card>
          {report && (
            <>
              <div role="status" aria-live="polite">
                Import state: {report.status}
                {report.status === "applying" && (
                  <>
                    {" "}
                    · {report.processed.toLocaleString()} imported of{" "}
                    {(report.counts.ready ?? 0).toLocaleString()} ready
                    <progress
                      aria-label="Import progress"
                      value={report.processed}
                      max={report.counts.ready || 1}
                    />
                  </>
                )}
              </div>
              {report.error && (
                <Alert title="Import needs attention" tone="danger">
                  {report.error}
                </Alert>
              )}
              <SummaryStrip
                items={Object.entries(labels).map(([key, label]) => ({
                  label,
                  value: report.counts[key] ?? 0,
                }))}
              />
              {report.mode === "apply" && (
                <Alert
                  title={
                    report.status === "complete"
                      ? "Import complete"
                      : "Import progress"
                  }
                  tone={report.status === "complete" ? "success" : "info"}
                >
                  {report.imported.toLocaleString()} products, packages and
                  initial prices committed.{" "}
                  {report.durationMs > 0 &&
                    `${(report.durationMs / 1000).toFixed(1)} seconds.`}
                </Alert>
              )}
              <Card>
                <h2>Import report</h2>
                <p>
                  Full row dispositions and original source values are saved in
                  the JSON report.
                </p>
                <p style={{ overflowWrap: "anywhere" }}>{report.reportPath}</p>
                {report.backupPath && (
                  <p style={{ overflowWrap: "anywhere" }}>
                    Backup: {report.backupPath}
                  </p>
                )}
                <p>
                  New lookups:{" "}
                  {Object.entries(report.lookupCreated)
                    .map(([k, v]) => `${v} ${k}`)
                    .join(" · ")}
                </p>
                {report.rows.length > 0 && (
                  <details>
                    <summary>
                      Review and skipped rows (first {report.rows.length})
                    </summary>
                    <ul>
                      {report.rows.map((r) => (
                        <li key={r.number}>
                          Row {r.number}: {r.fields[0] || r.fields[1]} —{" "}
                          {r.disposition.replaceAll("_", " ")}
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
              </Card>
            </>
          )}
          <ConfirmationModal
            open={confirm}
            onOpenChange={setConfirm}
            title="Apply catalog import?"
            description={`Import ${(report?.counts.ready ?? 0).toLocaleString()} reviewed rows. A SQLite backup will be created first. Review-required records will be skipped. Keep the app open until the import finishes.`}
            onConfirm={() => void apply()}
          />
        </>
      )}
    </div>
  );
}
