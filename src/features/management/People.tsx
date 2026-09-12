import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  FeaturePage,
  LinkButton,
  DataGrid,
  Status,
  Button,
  WorkflowDialog,
  BackLink,
  Card,
  DetailList,
  EmptyState,
  Select,
  Switch,
  Alert,
  FormSection,
  Input,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { roles, permissionGroups } from "../../mock/fixtures";
export function Employees() {
  const { employees, setEmployees, branches, notify } = useDemo();
  const [open, setOpen] = useState(false);
  return (
    <FeaturePage
      title="Employees"
      description="Your team, assigned branches, and workspace roles."
      actions={
        <>
          <LinkButton to="/roles">Roles & permissions</LinkButton>
          <Button variant="primary" onClick={() => setOpen(true)}>
            Add employee
          </Button>
        </>
      }
    >
      <DataGrid
        label="Employees"
        rows={employees}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.name} ${r.email}`}
        filters={[
          { label: "Roles", options: roles, value: (r) => r.role },
          {
            label: "Branches",
            options: branches.map((b) => b.name),
            value: (r) => r.branch,
          },
        ]}
        columns={[
          {
            key: "name",
            header: "Employee",
            render: (r) => (
              <Link className="text-link" to={`/employees/${r.id}`}>
                {r.name}
                <span className="cell-secondary">{r.email}</span>
              </Link>
            ),
          },
          { key: "role", header: "Role", render: (r) => r.role },
          { key: "branch", header: "Branch", render: (r) => r.branch },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
          { key: "phone", header: "Phone", render: (r) => r.phone },
          {
            key: "activity",
            header: "Last activity",
            render: (r) => r.activity,
          },
        ]}
      />
      <WorkflowDialog
        open={open}
        onClose={() => setOpen(false)}
        title="Add employee"
        description="This adds a demo profile. No invitation or account is created."
        fields={[
          { name: "name", label: "Full name", required: true },
          { name: "email", label: "Email", type: "email", required: true },
          { name: "phone", label: "Phone", type: "tel" },
          { name: "role", label: "Role", options: roles },
          {
            name: "branch",
            label: "Assigned branch",
            options: branches.map((b) => b.name),
          },
        ]}
        onSubmit={(v) => {
          setEmployees([
            ...employees,
            {
              id: `e-${Date.now()}`,
              name: v.name,
              email: v.email,
              phone: v.phone,
              role: v.role,
              branch: v.branch,
              status: "Invited",
              activity: "Not yet active",
            },
          ]);
          notify("Demo employee added. No invitation was sent.");
        }}
      />
    </FeaturePage>
  );
}
export function EmployeeDetails() {
  const { id } = useParams();
  const { employees, setEmployees, notify } = useDemo();
  const p = employees.find((e) => e.id === id);
  if (!p)
    return (
      <EmptyState
        title="Employee not found"
        description="Return to the employee directory."
      />
    );
  return (
    <>
      <BackLink to="/employees" label="Employees" />
      <FeaturePage
        title={p.name}
        description={`${p.role} · ${p.branch}`}
        actions={<LinkButton to="/roles">Review permissions</LinkButton>}
      >
        <div className="content-stack">
          <Card>
            <DetailList
              items={[
                { label: "Email", value: p.email },
                { label: "Contact", value: p.phone },
                { label: "Branch", value: p.branch },
                { label: "Status", value: <Status value={p.status} /> },
                { label: "Last activity", value: p.activity },
                {
                  label: "Permissions summary",
                  value: `${p.role} role · preview only`,
                },
              ]}
            />
          </Card>
          <FormSection title="Profile assignment">
            <Select
              showLabel
              label="Employee role"
              value={p.role}
              onChange={(e) => {
                setEmployees(
                  employees.map((v) =>
                    v.id === p.id ? { ...v, role: e.target.value } : v,
                  ),
                );
                notify("Demo role assignment updated.");
              }}
            >
              {roles.map((r) => (
                <option key={r}>{r}</option>
              ))}
            </Select>
            <Input
              label="Authentication"
              value="No account backend connected"
              disabled
            />
          </FormSection>
        </div>
      </FeaturePage>
    </>
  );
}
export function Roles() {
  const { notify } = useDemo();
  const [role, setRole] = useState("Pharmacist"),
    [customName, setCustomName] = useState("Custom Role"),
    [permissions, setPermissions] = useState<Record<string, boolean>>({});
  const base = (group: string, action: string) =>
    role === "Owner" ||
    role === "Manager" ||
    (role === "Pharmacist" &&
      ["Sales", "Inventory"].includes(group) &&
      action !== "Manage") ||
    (role === "Cashier" && group === "Sales" && action !== "Manage") ||
    (role === "Inventory Staff" &&
      group === "Inventory" &&
      action !== "Manage");
  return (
    <FeaturePage
      title="Roles & permissions"
      description="Review grouped access policies before connecting real authorization."
      actions={
        <Button
          variant="primary"
          onClick={() =>
            notify(
              "Permission preview saved in this screen. No authorization policy was changed.",
            )
          }
        >
          Save role preview
        </Button>
      }
    >
      <Alert title="Access-control design preview" tone="info">
        These switches do not enforce permissions. Real authorization belongs to
        a later phase.
      </Alert>
      <div className="section-menu">
        {roles.map((r) => (
          <Button
            key={r}
            variant={r === role ? "primary" : "secondary"}
            onClick={() => setRole(r)}
          >
            {r}
          </Button>
        ))}
      </div>
      {role === "Custom Role" && (
        <Input
          label="Custom role name"
          value={customName}
          onChange={(e) => setCustomName(e.target.value)}
        />
      )}
      <div className="permission-grid">
        {permissionGroups.map((group) => (
          <Card key={group}>
            <h3>{group}</h3>
            <div className="stack">
              {["View", "Create / edit", "Manage"].map((action) => {
                const key = `${role}:${group}:${action}`;
                return (
                  <Switch
                    key={action}
                    label={action}
                    checked={permissions[key] ?? base(group, action)}
                    disabled={role === "Owner"}
                    onChange={(value) =>
                      setPermissions({ ...permissions, [key]: value })
                    }
                  />
                );
              })}
            </div>
          </Card>
        ))}
      </div>
    </FeaturePage>
  );
}
