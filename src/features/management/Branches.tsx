import { useState } from "react";
import { useParams } from "react-router-dom";
import {
  FeaturePage,
  Card,
  SectionHeader,
  Status,
  DetailList,
  LinkButton,
  WorkflowDialog,
  Button,
  BackLink,
  Tabs,
  Table,
  Alert,
  EmptyState,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
export function Branches() {
  const { branches, setBranches, notify } = useDemo();
  const [open, setOpen] = useState(false);
  return (
    <FeaturePage
      title="Branches"
      description="Branch identities, teams, devices, and future synchronization health."
      actions={
        <Button variant="primary" onClick={() => setOpen(true)}>
          Add branch
        </Button>
      }
    >
      <div className="equal-columns">
        {branches.map((b) => (
          <Card key={b.id}>
            <SectionHeader
              title={b.name}
              action={<Status value={b.status} />}
            />
            <DetailList
              items={[
                { label: "Code", value: b.code },
                { label: "Manager", value: b.manager },
                { label: "Address", value: b.address },
                { label: "Phone", value: b.phone },
                { label: "Devices", value: b.devices },
                { label: "Last sync", value: b.sync },
              ]}
            />
            <div className="row end">
              <LinkButton to={`/branches/${b.id}`}>Branch details</LinkButton>
            </div>
          </Card>
        ))}
      </div>
      <WorkflowDialog
        open={open}
        onClose={() => setOpen(false)}
        title="Add branch"
        description="Create an in-memory branch profile without provisioning a device or database."
        fields={[
          { name: "name", label: "Branch name", required: true },
          { name: "code", label: "Code", required: true },
          { name: "address", label: "Address", required: true },
          { name: "phone", label: "Phone" },
          { name: "manager", label: "Manager" },
        ]}
        onSubmit={(v) => {
          setBranches([
            ...branches,
            {
              id: `b-${Date.now()}`,
              name: v.name,
              code: v.code,
              address: v.address,
              phone: v.phone,
              manager: v.manager,
              status: "Setup",
              devices: 0,
              sync: "Not connected",
            },
          ]);
          notify("Branch added to this demo session.");
        }}
      />
    </FeaturePage>
  );
}
export function BranchDetails() {
  const { id } = useParams();
  const { branches, employees, products } = useDemo();
  const b = branches.find((b) => b.id === id);
  if (!b)
    return (
      <EmptyState
        title="Branch not found"
        description="Select a branch from the directory."
      />
    );
  return (
    <>
      <BackLink to="/branches" label="Branches" />
      <FeaturePage title={b.name} description={`${b.code} · ${b.address}`}>
        <Card>
          <Tabs
            items={[
              {
                label: "Overview",
                content: (
                  <DetailList
                    items={[
                      { label: "Manager", value: b.manager },
                      { label: "Phone", value: b.phone },
                      {
                        label: "Sales summary",
                        value: "18,450 EGP · illustrative only",
                      },
                      {
                        label: "Inventory summary",
                        value: `${products.length} demo catalog records`,
                      },
                    ]}
                  />
                ),
              },
              {
                label: "Employees",
                content: (
                  <Table
                    label="Branch employees"
                    rows={employees.filter((e) => e.branch === b.name)}
                    rowKey={(r) => r.id}
                    columns={[
                      { key: "name", header: "Name", render: (r) => r.name },
                      { key: "role", header: "Role", render: (r) => r.role },
                      {
                        key: "status",
                        header: "Status",
                        render: (r) => <Status value={r.status} />,
                      },
                    ]}
                  />
                ),
              },
              {
                label: "Devices",
                content: (
                  <Table
                    label="Branch devices"
                    rows={Array.from({ length: b.devices }, (_, i) => ({
                      id: `${b.code}-POS-0${i + 1}`,
                      type: i === 0 ? "Primary terminal" : "POS terminal",
                    }))}
                    rowKey={(r) => r.id}
                    columns={[
                      {
                        key: "id",
                        header: "Device identity",
                        render: (r) => r.id,
                      },
                      { key: "type", header: "Type", render: (r) => r.type },
                      {
                        key: "status",
                        header: "Status",
                        render: () => <Status value="Not connected" />,
                      },
                    ]}
                  />
                ),
              },
              {
                label: "Sync health",
                content: (
                  <div className="stack">
                    <Alert title="Cloud integration is not configured">
                      Local/cloud indicators are design previews. There is no
                      active synchronization service.
                    </Alert>
                    <LinkButton to="/sync">Open sync status</LinkButton>
                  </div>
                ),
              },
            ]}
          />
        </Card>
      </FeaturePage>
    </>
  );
}
