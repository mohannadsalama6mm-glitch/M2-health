import { useState } from "react";
import {
  FeaturePage,
  Alert,
  Button,
  SummaryStrip,
  Card,
  SectionHeader,
  Switch,
  Input,
  DataGrid,
  Status,
  WorkflowDialog,
  DetailList,
  Drawer,
} from "../../design-system";
import { backupHistory, syncQueue } from "../../mock/fixtures";
import { useDemo } from "../../app/DemoContext";
export function Backup() {
  const { notify } = useDemo();
  const [automatic, setAutomatic] = useState(true),
    [location, setLocation] = useState("D:\\M2Health\\Backups"),
    [action, setAction] = useState(""),
    [selected, setSelected] = useState("BK-008"),
    [reviewed, setReviewed] = useState(false);
  return (
    <FeaturePage
      title="Backup & restore"
      description="Local recovery workflow and backup policy preview."
      actions={
        <Button variant="primary" onClick={() => setAction("Create backup")}>
          Create backup
        </Button>
      }
    >
      <Alert title="No database or backup files exist in this phase">
        Backup history below is sample data. No disk operation will run.
      </Alert>
      <SummaryStrip
        items={[
          { label: "Last backup · sample", value: "10 Sep · 23:00" },
          { label: "Actual backup status", value: "Not configured" },
          { label: "Retention preview", value: "30 days" },
        ]}
      />
      <Card>
        <SectionHeader title="Backup policy" />
        <div className="form-grid">
          <Switch
            label="Automatic backup"
            checked={automatic}
            onChange={setAutomatic}
          />
          <Input
            label="Storage location preview"
            value={location}
            onChange={(e) => setLocation(e.target.value)}
          />
          <Input
            label="Scheduled time"
            type="time"
            defaultValue="23:00"
            disabled={!automatic}
          />
          <Button
            onClick={() =>
              notify(
                "Backup preferences reviewed. No scheduler was configured.",
              )
            }
          >
            Save policy preview
          </Button>
        </div>
      </Card>
      <DataGrid
        label="Backup history"
        rows={backupHistory}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.id} ${r.date}`}
        columns={[
          { key: "id", header: "Backup", render: (r) => r.id },
          { key: "date", header: "Created", render: (r) => r.date },
          { key: "type", header: "Type", render: (r) => r.type },
          { key: "size", header: "Size", render: (r) => r.size },
          { key: "location", header: "Location", render: (r) => r.location },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
          {
            key: "restore",
            header: "Action",
            render: (r) => (
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setSelected(r.id);
                  setAction("Restore backup");
                }}
              >
                Restore
              </Button>
            ),
          },
        ]}
      />
      {reviewed && (
        <Alert title="Restore preview reviewed" tone="success">
          No files or database records were replaced.
        </Alert>
      )}
      <WorkflowDialog
        key={action}
        open={!!action}
        onClose={() => setAction("")}
        title={action}
        description={
          action === "Restore backup"
            ? `Selected backup ${selected}. A real restore would replace local operational data and require exclusive access.`
            : "Review the destination for a future local backup."
        }
        danger={action === "Restore backup"}
        fields={
          action === "Restore backup"
            ? [
                {
                  name: "confirmation",
                  label: "Type RESTORE to review this destructive workflow",
                  required: true,
                  pattern: "RESTORE",
                },
                {
                  name: "reason",
                  label: "Restore reason",
                  type: "textarea",
                  required: true,
                },
              ]
            : [
                {
                  name: "destination",
                  label: "Destination",
                  value: location,
                  required: true,
                },
                { name: "notes", label: "Notes", type: "textarea" },
              ]
        }
        submitLabel={
          action === "Restore backup" ? "Review restore" : "Review backup"
        }
        onSubmit={(v) => {
          if (action === "Restore backup" && v.confirmation !== "RESTORE") {
            notify("Restore not reviewed: confirmation must be RESTORE.");
            return;
          }
          if (action === "Restore backup") setReviewed(true);
          notify("Preview reviewed. No backup or restore operation ran.");
        }}
      />
    </FeaturePage>
  );
}
export function Sync() {
  const { branch, notify } = useDemo();
  const [queue, setQueue] = useState(false),
    [scenario, setScenario] = useState("Offline"),
    [detail, setDetail] = useState("");
  return (
    <FeaturePage
      title="Sync / cloud status"
      description="Offline-first workspace health and future synchronization controls."
      actions={<Button onClick={() => setDetail("Sync now")}>Sync now</Button>}
    >
      <Alert title="Local and cloud services are not connected" tone="info">
        These are UI scenarios, not an active synchronization engine. The future
        local database will allow offline operations.
      </Alert>
      <div className="section-menu">
        {["Offline", "Online", "Attention needed"].map((s) => (
          <Button
            key={s}
            variant={scenario === s ? "primary" : "secondary"}
            onClick={() => setScenario(s)}
          >
            {s} scenario
          </Button>
        ))}
      </div>
      <SummaryStrip
        items={[
          { label: "Connection scenario", value: scenario },
          { label: "Pending sample changes", value: 1 },
          { label: "Failed / conflicted samples", value: 2 },
          { label: "Last successful sync · demo", value: "10:20 AM" },
        ]}
      />
      <div className="equal-columns">
        <Card>
          <SectionHeader title="Local workspace" />
          <DetailList
            items={[
              { label: "Database status", value: "Not implemented" },
              { label: "Device identity", value: "MAIN-POS-01 · demo" },
              { label: "Branch", value: branch },
              { label: "Pending queue", value: "3 sample records" },
            ]}
          />
        </Card>
        <Card>
          <SectionHeader title="Cloud service" />
          <DetailList
            items={[
              { label: "Platform", value: "Not configured" },
              { label: "Authentication", value: "Not connected" },
              { label: "Scenario", value: scenario },
              { label: "Backup integration", value: "Not configured" },
            ]}
          />
        </Card>
      </div>
      <div className="row">
        <Button onClick={() => setQueue(true)}>View queue</Button>
        <Button onClick={() => setDetail("Retry failed changes")}>
          Retry failed
        </Button>
      </div>
      <DataGrid
        label="Sync errors"
        rows={syncQueue.filter((r) => r.status !== "Pending")}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.entity} ${r.reason}`}
        columns={[
          { key: "entity", header: "Entity", render: (r) => r.entity },
          { key: "reason", header: "Reason", render: (r) => r.reason },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
        ]}
      />
      <Drawer
        open={queue}
        onOpenChange={setQueue}
        title="Pending synchronization queue"
        description="Illustrative changes only"
      >
        <div className="stack">
          {syncQueue.map((r) => (
            <Card key={r.id}>
              <SectionHeader
                title={r.entity}
                action={<Status value={r.status} />}
              />
              <p className="muted">
                {r.type} · {r.branch}
              </p>
              <p>{r.reason}</p>
            </Card>
          ))}
        </div>
      </Drawer>
      <WorkflowDialog
        open={!!detail}
        onClose={() => setDetail("")}
        title={detail}
        description="No sync request will be sent. Review the proposed operation."
        fields={[
          {
            name: "scope",
            label: "Scope",
            options: ["Current device", "Current branch"],
          },
          { name: "note", label: "Operator note" },
        ]}
        onSubmit={() =>
          notify("Sync action reviewed. No local/cloud service is connected.")
        }
      />
    </FeaturePage>
  );
}
