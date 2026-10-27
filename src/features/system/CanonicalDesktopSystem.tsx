/* eslint-disable react-hooks/set-state-in-effect */
import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import {
  Alert,
  Badge,
  Button,
  Card,
  DetailList,
  Drawer,
  EmptyState,
  Input,
  PageHeader,
  Select,
  Status,
  SummaryStrip,
  Table,
  Textarea,
  WorkflowDialog,
} from "../../design-system";
import { useBranch } from "../../app/BranchContext";
import { useAuth } from "../../app/AuthContext";
import { useLicense } from "../../app/LicenseContext";
import {
  evaluateAlerts,
  getAlertSettings,
  listAlerts,
  saveAlertSettings,
  type AlertSettings,
  type InventoryAlert,
  type Severity,
} from "../../lib/tauri/inventory-alerts";
import { authStatus } from "../../lib/tauri/auth";
import { getCloudFoundationStatus } from "../../lib/tauri/cloud";
import {
  createBackup,
  getBackupStatus,
  getLocalSettings,
  listAuditEvents,
  restoreBackup,
  updateLocalSettings,
  validateBackup,
  type AuditEvent,
  type BackupRecord,
  type BackupStatus,
  type LocalSettings,
} from "../../lib/tauri/desktop";
import {
  exportDiagnostics,
  getReleaseInfo,
  type ReleaseInfo,
} from "../../lib/tauri/licensing";
import { useLocalization } from "../../localization/LocalizationContext";

const errorText = (e: unknown) =>
  e instanceof Error
    ? e.message
    : "The desktop service could not complete this request.";
const alertPath = (row: InventoryAlert) =>
  row.alertType.startsWith("BATCH_")
    ? "/inventory/expiry"
    : "/inventory/low-stock";
export function CanonicalNotifications() {
  const { branchId, branchName } = useBranch(),
    auth = useAuth();
  const [rows, setRows] = useState<InventoryAlert[]>([]),
    [severity, setSeverity] = useState<Severity | "">(""),
    [loading, setLoading] = useState(true),
    [error, setError] = useState(""),
    [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!branchId) return;
    let active = true;
    setLoading(true);
    listAlerts({
      branchId,
      alertType: null,
      severity: severity || null,
      status: "ACTIVE",
      fromDate: null,
      toDate: null,
      limit: 100,
      offset: 0,
    })
      .then((v) => active && setRows(v.items))
      .catch((e) => active && setError(errorText(e)))
      .finally(() => active && setLoading(false));
    return () => {
      active = false;
    };
  }, [branchId, severity, attempt]);
  const refresh = async () => {
    setError("");
    try {
      if (auth.hasPermission("inventory.alert.manage"))
        await evaluateAlerts(branchId);
      setAttempt((v) => v + 1);
    } catch (e) {
      setError(errorText(e));
    }
  };
  return (
    <div className="stack">
      <PageHeader
        title="Notifications"
        description={`Active inventory and expiry conditions for ${branchName}.`}
        actions={<Button onClick={() => void refresh()}>Refresh</Button>}
      />
      <div className="toolbar">
        <Select
          label="Severity"
          value={severity}
          onChange={(e) => setSeverity(e.target.value as Severity | "")}
        >
          <option value="">All severities</option>
          <option>CRITICAL</option>
          <option>HIGH</option>
          <option>MEDIUM</option>
        </Select>
        <Badge>{rows.length} active</Badge>
      </div>
      {error && (
        <Alert title="Notifications unavailable" tone="danger">
          {error}
        </Alert>
      )}
      {loading ? (
        <p role="status">Loading notifications…</p>
      ) : rows.length ? (
        <Card>
          {rows.map((r) => (
            <article className="notification-item unread" key={r.id}>
              <div>
                <h3>
                  {r.productName || "Catalog product"} · {r.packageLabel}
                </h3>
                <p className="muted">
                  {r.alertType.replaceAll("_", " ")} · last detected{" "}
                  {r.lastDetectedAt}
                </p>
                <div className="row">
                  <Status value={r.severity} />
                  <Badge>{branchName}</Badge>
                </div>
              </div>
              <Link className="button secondary sm" to={alertPath(r)}>
                Review
              </Link>
            </article>
          ))}
        </Card>
      ) : (
        <EmptyState
          title="No active notifications"
          description="No active stock or expiry condition matches this branch."
        />
      )}
    </div>
  );
}

const eventTypes = [
  "USER_CREATED",
  "USER_UPDATED",
  "USER_ACTIVATED",
  "USER_DEACTIVATED",
  "PASSWORD_RESET",
  "ROLE_ASSIGNED",
  "ROLE_REMOVED",
  "BRANCH_MEMBERSHIP_CHANGED",
];
export function CanonicalAuditLog() {
  const [rows, setRows] = useState<AuditEvent[]>([]),
    [total, setTotal] = useState(0),
    [search, setSearch] = useState(""),
    [type, setType] = useState(""),
    [from, setFrom] = useState(""),
    [to, setTo] = useState(""),
    [page, setPage] = useState(0),
    [loading, setLoading] = useState(true),
    [error, setError] = useState(""),
    [detail, setDetail] = useState<AuditEvent | null>(null),
    [attempt, setAttempt] = useState(0);
  const query = useMemo(
    () => ({
      search: search || null,
      eventType: type || null,
      fromDate: from || null,
      toDate: to || null,
      limit: 50,
      offset: page * 50,
    }),
    [search, type, from, to, page],
  );
  useEffect(() => {
    let active = true;
    setLoading(true);
    setError("");
    listAuditEvents(query)
      .then((v) => {
        if (active) {
          setRows(v.items);
          setTotal(v.total);
        }
      })
      .catch((e) => active && setError(errorText(e)))
      .finally(() => active && setLoading(false));
    return () => {
      active = false;
    };
  }, [query, attempt]);
  return (
    <div className="stack">
      <PageHeader
        title="Security audit log"
        description="Immutable local history of user, role, password, and branch-membership administration."
      />
      <Card>
        <div className="toolbar">
          <Input
            label="Search audit"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0);
            }}
          />
          <Select
            label="Event type"
            value={type}
            onChange={(e) => {
              setType(e.target.value);
              setPage(0);
            }}
          >
            <option value="">All event types</option>
            {eventTypes.map((v) => (
              <option key={v}>{v}</option>
            ))}
          </Select>
          <Input
            label="From"
            type="date"
            value={from}
            onChange={(e) => setFrom(e.target.value)}
          />
          <Input
            label="Through"
            type="date"
            value={to}
            onChange={(e) => setTo(e.target.value)}
          />
        </div>
      </Card>
      {error && (
        <Alert title="Audit unavailable" tone="danger">
          {error}{" "}
          <Button onClick={() => setAttempt((v) => v + 1)}>Retry</Button>
        </Alert>
      )}
      {loading ? (
        <p role="status">Loading audit events…</p>
      ) : rows.length ? (
        <Card>
          <Table
            label="Security audit events"
            rows={rows}
            rowKey={(r) => r.id}
            columns={[
              { key: "time", header: "Time", render: (r) => r.createdAt },
              { key: "actor", header: "Actor", render: (r) => r.actorName },
              { key: "event", header: "Event", render: (r) => r.eventType },
              {
                key: "subject",
                header: "Subject",
                render: (r) => r.subjectName || "—",
              },
              {
                key: "detail",
                header: "Details",
                render: (r) => (
                  <Button size="sm" onClick={() => setDetail(r)}>
                    Inspect
                  </Button>
                ),
              },
            ]}
          />
          <div className="grid-footer">
            <Button disabled={page === 0} onClick={() => setPage((v) => v - 1)}>
              Previous
            </Button>
            <span>
              {total} events · page {page + 1}
            </span>
            <Button
              disabled={(page + 1) * 50 >= total}
              onClick={() => setPage((v) => v + 1)}
            >
              Next
            </Button>
          </div>
        </Card>
      ) : (
        <EmptyState
          title="No audit events"
          description="No security administration event matches the current filters."
        />
      )}
      <Drawer
        open={!!detail}
        onOpenChange={(o) => !o && setDetail(null)}
        title={detail?.eventType || "Event details"}
        description="Persisted immutable security event"
      >
        <DetailList
          items={
            detail
              ? Object.entries(detail).map(([label, value]) => ({
                  label,
                  value: value ?? "—",
                }))
              : []
          }
        />
      </Drawer>
    </div>
  );
}

const size = (bytes: number) => `${(bytes / 1024 / 1024).toFixed(2)} MB`;
export function CanonicalBackup() {
  const [status, setStatus] = useState<BackupStatus | null>(null),
    [selected, setSelected] = useState<BackupRecord | null>(null),
    [loading, setLoading] = useState(true),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState("");
  const load = () => {
    setLoading(true);
    return getBackupStatus()
      .then(setStatus)
      .catch((e) => setError(errorText(e)))
      .finally(() => setLoading(false));
  };
  useEffect(() => {
    void load();
  }, []);
  const create = async () => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const row = await createBackup();
      setNotice(`Backup ${row.id} passed its integrity check.`);
      await load();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const restore = async (values: Record<string, string>) => {
    if (!selected) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await validateBackup(selected.id);
      const result = await restoreBackup(
        selected.id,
        values.confirmation,
        values.reason,
      );
      setNotice(
        `Restore completed. Safety backup: ${result.safetyBackupId}. Restart M² Health before continuing.`,
      );
      await load();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="stack">
      <PageHeader
        title="Backup and restore"
        description="Verified local SQLite backups in the application-managed recovery folder."
        actions={
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => void create()}
          >
            {busy ? "Working…" : "Create verified backup"}
          </Button>
        }
      />
      {error && (
        <Alert title="Backup operation failed" tone="danger">
          {error}
        </Alert>
      )}
      {notice && (
        <Alert title="Backup operation completed" tone="success">
          {notice}
        </Alert>
      )}
      <SummaryStrip
        items={[
          {
            label: "Managed location",
            value: status?.managedLocation || "Loading…",
          },
          { label: "Verified backups", value: status?.backups.length ?? 0 },
          { label: "Restore policy", value: "Safety backup first" },
        ]}
      />
      {loading ? (
        <p role="status">Loading backups…</p>
      ) : status?.backups.length ? (
        <Card>
          <Table
            label="Verified local backups"
            rows={status.backups}
            rowKey={(r) => r.id}
            columns={[
              { key: "created", header: "Created", render: (r) => r.createdAt },
              { key: "size", header: "Size", render: (r) => size(r.sizeBytes) },
              {
                key: "schema",
                header: "Schema",
                render: (r) => `v${r.schemaVersion}`,
              },
              {
                key: "integrity",
                header: "Integrity",
                render: (r) => (
                  <Status
                    value={r.integrity === "ok" ? "Verified" : "Failed"}
                  />
                ),
              },
              {
                key: "action",
                header: "Action",
                render: (r) => (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setSelected(r)}
                  >
                    Restore
                  </Button>
                ),
              },
            ]}
          />
        </Card>
      ) : (
        <EmptyState
          title="No backups yet"
          description="Create a verified backup before a risky local operation."
        />
      )}
      <Alert title="Restore safety">
        Restore accepts only a verified M² Health backup from this managed
        folder. The application creates and retains a safety backup before
        replacing local data.
      </Alert>
      <WorkflowDialog
        key={selected?.id}
        open={!!selected}
        onClose={() => setSelected(null)}
        title="Restore local database"
        description={`Replace local data with ${selected?.id || "the selected backup"}.`}
        danger
        submitLabel="Validate and restore"
        fields={[
          {
            name: "confirmation",
            label: "Type RESTORE",
            required: true,
            pattern: "RESTORE",
          },
          {
            name: "reason",
            label: "Restore reason",
            type: "textarea",
            required: true,
          },
        ]}
        onSubmit={(v) => void restore(v)}
      />
    </div>
  );
}

export function CanonicalSettings() {
  const { direction, locale, setLocale, t } = useLocalization();
  const { branchId, branchName } = useBranch();
  const license = useLicense();
  const [value, setValue] = useState<LocalSettings | null>(null),
    [alerts, setAlerts] = useState<AlertSettings | null>(null),
    [runtime, setRuntime] = useState<{
      session: number;
      device: string;
      transport: boolean;
      release: ReleaseInfo;
    } | null>(null),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState("");
  useEffect(() => {
    let active = true;
    Promise.all([
      getLocalSettings(),
      getAlertSettings(branchId),
      authStatus(),
      getCloudFoundationStatus(),
      getReleaseInfo(),
    ])
      .then(([settings, alertSettings, auth, cloud, releaseInfo]) => {
        if (active) {
          setValue(settings);
          setAlerts(alertSettings);
          setRuntime({
            session: auth.sessionLifetimeSeconds,
            device: cloud.deviceId,
            transport: cloud.transportEnabled,
            release: releaseInfo,
          });
        }
      })
      .catch((e) => active && setError(errorText(e)));
    return () => {
      active = false;
    };
  }, [branchId]);
  const run = async (work: () => Promise<unknown>, noticeText: string) => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await work();
      setNotice(noticeText);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const saveProfile = (e: FormEvent) => {
    e.preventDefault();
    if (value)
      void run(
        async () => setValue(await updateLocalSettings(value)),
        "Local pharmacy settings were saved.",
      );
  };
  return (
    <div className="stack">
      <PageHeader
        title={t("settings.title")}
        description={t("settings.description")}
      />
      <Card>
        <h2>{t("settings.languageTitle")}</h2>
        <p className="muted">{t("settings.languageCopy")}</p>
        <div className="form-grid">
          <Select
            label={t("language.label")}
            showLabel
            value={locale}
            onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}
          >
            <option value="en">{t("language.english")}</option>
            <option value="ar">{t("language.arabic")}</option>
          </Select>
          <Input label={t("language.direction")} disabled value={direction.toUpperCase()} />
        </div>
        <p className="field-help">{t("language.saved")}</p>
      </Card>
      {error && (
        <Alert title="Settings unavailable" tone="danger">
          {error}
        </Alert>
      )}
      {notice && (
        <Alert title="Settings saved" tone="success">
          {notice}
        </Alert>
      )}
      {!value || !alerts ? (
        <p role="status">Loading settings…</p>
      ) : (
        <>
          <Card>
            <h2>{t("settings.pharmacyProfile")}</h2>
            <form className="form-grid" onSubmit={saveProfile}>
              <Input
                label="Pharmacy name"
                required
                value={value.pharmacyName}
                onChange={(e) =>
                  setValue({ ...value, pharmacyName: e.target.value })
                }
              />
              <Input
                label="Phone"
                value={value.pharmacyPhone}
                onChange={(e) =>
                  setValue({ ...value, pharmacyPhone: e.target.value })
                }
              />
              <Textarea
                label="Address"
                value={value.pharmacyAddress}
                onChange={(e) =>
                  setValue({ ...value, pharmacyAddress: e.target.value })
                }
              />
              <Input
                label="Registration number"
                value={value.registrationNumber}
                onChange={(e) =>
                  setValue({ ...value, registrationNumber: e.target.value })
                }
              />
              <Select
                label="Time zone"
                showLabel
                value={value.timezone}
                onChange={(e) =>
                  setValue({
                    ...value,
                    timezone: e.target.value as LocalSettings["timezone"],
                  })
                }
              >
                <option>Africa/Cairo</option>
                <option>UTC</option>
              </Select>
              <Input
                label="Receipt header"
                required
                value={value.receiptHeader}
                onChange={(e) =>
                  setValue({ ...value, receiptHeader: e.target.value })
                }
              />
              <Textarea
                label="Receipt footer"
                value={value.receiptFooter}
                onChange={(e) =>
                  setValue({ ...value, receiptFooter: e.target.value })
                }
              />
              <Button type="submit" variant="primary" disabled={busy}>
                Save pharmacy settings
              </Button>
            </form>
          </Card>
          <Card>
            <h2>Inventory monitoring · {branchName}</h2>
            <form
              className="form-grid"
              onSubmit={(e) => {
                e.preventDefault();
                void run(
                  async () => setAlerts(await saveAlertSettings(alerts)),
                  "Branch inventory monitoring settings were saved.",
                );
              }}
            >
              <Input
                label="Expiry warning days"
                type="number"
                min={0}
                value={alerts.expiryWarningDays}
                onChange={(e) =>
                  setAlerts({
                    ...alerts,
                    expiryWarningDays: Number(e.target.value),
                  })
                }
              />
              <Input
                label="Critical expiry days"
                type="number"
                min={0}
                value={alerts.criticalExpiryDays}
                onChange={(e) =>
                  setAlerts({
                    ...alerts,
                    criticalExpiryDays: Number(e.target.value),
                  })
                }
              />
              <label>
                <input
                  type="checkbox"
                  checked={alerts.expiryEnabled}
                  onChange={(e) =>
                    setAlerts({ ...alerts, expiryEnabled: e.target.checked })
                  }
                />{" "}
                Expiry alerts enabled
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={alerts.lowStockEnabled}
                  onChange={(e) =>
                    setAlerts({ ...alerts, lowStockEnabled: e.target.checked })
                  }
                />{" "}
                Low-stock alerts enabled
              </label>
              <Button type="submit" variant="primary" disabled={busy}>
                Save monitoring settings
              </Button>
            </form>
          </Card>
        </>
      )}
      <Card>
        <h2>Runtime security status</h2>
        <DetailList
          items={[
            {
              label: "Local session lifetime",
              value: runtime ? `${runtime.session / 3600} hours` : "—",
            },
            { label: "Current branch", value: branchName },
            { label: "Device identity", value: runtime?.device || "—" },
            {
              label: "Cloud transport",
              value: runtime?.transport
                ? "Enabled"
                : "Disabled by rollout gates",
            },
            { label: "Currency", value: "EGP" },
            {
              label: "Release",
              value: runtime
                ? `${runtime.release.version} · ${runtime.release.channel} · ${runtime.release.buildId}`
                : "—",
            },
            {
              label: "Database schema",
              value: runtime?.release.schemaVersion ?? "—",
            },
            {
              label: "License",
              value:
                license.status?.state === "TRIAL"
                  ? `Trial · ${license.status.daysRemaining ?? 0} days remaining`
                  : license.status?.state ?? "—",
            },
            {
              label: "Updater",
              value: runtime?.release.updaterEnabled
                ? "Enabled"
                : "Not deployed for Early Access",
            },
          ]}
        />
        <Button
          disabled={busy}
          onClick={() =>
            void run(
              async () => {
                await exportDiagnostics();
              },
              "Privacy-safe diagnostics were exported to the application diagnostics folder.",
            )
          }
        >
          Export diagnostics
        </Button>
        <p className="field-help">
          The export contains release, schema, license-mode, integrity, and
          application lifecycle metadata only—never credentials or pharmacy
          records.
        </p>
      </Card>
    </div>
  );
}
