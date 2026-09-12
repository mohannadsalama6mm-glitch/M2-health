import { useState } from "react";
import { Bell } from "lucide-react";
import {
  FeaturePage,
  Card,
  Badge,
  Button,
  Select,
  EmptyState,
  LinkButton,
  DataGrid,
  Status,
  Drawer,
  DetailList,
  Input,
} from "../../design-system";
import { notifications, auditRows } from "../../mock/fixtures";
import { useDemo } from "../../app/DemoContext";
export function Notifications() {
  const [rows, setRows] = useState(notifications),
    [category, setCategory] = useState("All"),
    [read, setRead] = useState("All"),
    [priority, setPriority] = useState("All");
  const filtered = rows.filter(
    (r) =>
      (category === "All" || r.category === category) &&
      (read === "All" || (read === "Unread" ? !r.read : r.read)) &&
      (priority === "All" || r.priority === priority),
  );
  return (
    <FeaturePage
      title="Notifications"
      description="Stock, expiry, purchasing, and system alerts in one place."
      actions={
        <Button
          disabled={rows.every((r) => r.read)}
          onClick={() => setRows(rows.map((r) => ({ ...r, read: true })))}
        >
          Mark all read
        </Button>
      }
    >
      <div className="section-menu">
        <Select
          label="Notification category"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
        >
          <option>All</option>
          {notifications.map((n) => (
            <option key={n.category}>{n.category}</option>
          ))}
        </Select>
        <Select
          label="Read status"
          value={read}
          onChange={(e) => setRead(e.target.value)}
        >
          <option>All</option>
          <option>Unread</option>
          <option>Read</option>
        </Select>
        <Select
          label="Priority"
          value={priority}
          onChange={(e) => setPriority(e.target.value)}
        >
          <option>All</option>
          <option>High</option>
          <option>Medium</option>
          <option>Normal</option>
        </Select>
        <Badge>{rows.filter((r) => !r.read).length} unread</Badge>
      </div>
      <Card>
        {filtered.length ? (
          filtered.map((n) => (
            <article
              key={n.id}
              className={`notification-item ${n.read ? "" : "unread"}`}
            >
              <Bell size={18} />
              <div>
                <h3>{n.title}</h3>
                <p className="muted">{n.detail}</p>
                <div className="row">
                  <Badge
                    tone={
                      n.priority === "High"
                        ? "danger"
                        : n.priority === "Medium"
                          ? "warning"
                          : "info"
                    }
                  >
                    {n.priority}
                  </Badge>
                  <Badge>{n.category}</Badge>
                  <span className="muted">{n.time}</span>
                </div>
              </div>
              <div className="stack">
                <LinkButton to={n.path}>Review</LinkButton>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() =>
                    setRows(
                      rows.map((r) =>
                        r.id === n.id ? { ...r, read: !r.read } : r,
                      ),
                    )
                  }
                >
                  {n.read ? "Mark unread" : "Mark read"}
                </Button>
              </div>
            </article>
          ))
        ) : (
          <EmptyState
            title="No matching notifications"
            description="Change the category, priority, or read filter."
          />
        )}
      </Card>
    </FeaturePage>
  );
}
export function AuditLog() {
  const { employees, branches } = useDemo();
  const [date, setDate] = useState("2026-09-11"),
    [detail, setDetail] = useState<(typeof auditRows)[number] | null>(null);
  return (
    <FeaturePage
      title="Activity / audit log"
      description="An inspectable history of fictional workspace events."
    >
      <DataGrid
        label="Audit events"
        rows={auditRows.filter((r) => !date || r.time.startsWith(date))}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.entity} ${r.id}`}
        filters={[
          {
            label: "Users",
            options: employees.map((e) => e.name),
            value: (r) => r.user,
          },
          {
            label: "Modules",
            options: [...new Set(auditRows.map((r) => r.module))],
            value: (r) => r.module,
          },
          {
            label: "Actions",
            options: [...new Set(auditRows.map((r) => r.action))],
            value: (r) => r.action,
          },
          {
            label: "Branches",
            options: branches.map((b) => b.name),
            value: (r) => r.branch,
          },
        ]}
        actions={
          <Input
            label="Audit date"
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
        }
        columns={[
          { key: "time", header: "Time", render: (r) => r.time },
          { key: "user", header: "User", render: (r) => r.user },
          { key: "action", header: "Action", render: (r) => r.action },
          { key: "module", header: "Module", render: (r) => r.module },
          { key: "entity", header: "Entity", render: (r) => r.entity },
          { key: "branch", header: "Branch", render: (r) => r.branch },
          {
            key: "result",
            header: "Result",
            render: (r) => <Status value={r.result} />,
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
      <Drawer
        open={!!detail}
        onOpenChange={() => setDetail(null)}
        title={detail?.id ?? "Event details"}
        description="Audit event preview"
      >
        <DetailList
          items={
            detail
              ? Object.entries(detail).map(([label, value]) => ({
                  label,
                  value,
                }))
              : []
          }
        />
      </Drawer>
    </FeaturePage>
  );
}
