import { useState } from "react";
import {
  Card,
  SectionHeader,
  SummaryStrip,
  DetailList,
  Status,
  FormSection,
  Input,
  RepeatableRows,
  RouteTabs,
  DataGrid,
  Button,
  WorkflowDialog,
  OutputPreview,
  Alert,
} from "../../design-system";
import { products } from "../../mock/fixtures";
export function WorkspaceShowcase() {
  const [rows, setRows] = useState(["Box · 24 tablets"]),
    [open, setOpen] = useState("");
  return (
    <div className="content-stack">
      <Card>
        <SectionHeader title="14 / Phase 1B workspace patterns" />
        <p className="muted">
          FeaturePage provides loading, empty, error, and retry previews on
          every operational module. Shared patterns below are used by catalog,
          stock, purchases, relationships, and system screens.
        </p>
        <RouteTabs
          items={[
            { label: "Catalog", to: "/catalog" },
            { label: "Inventory", to: "/inventory" },
            { label: "Purchases", to: "/purchases" },
          ]}
        />
        <SummaryStrip
          items={[
            { label: "Record summary", value: 12 },
            { label: "Review status", value: <Status value="In Progress" /> },
          ]}
        />
        <DetailList
          items={[
            { label: "Field / value pair", value: "Product information" },
            { label: "Branch", value: "Main branch" },
          ]}
        />
      </Card>
      <FormSection title="FormSection & RepeatableRows">
        <Input label="Shared form field" placeholder="Product name" />
        <Input label="Disabled field" disabled value="Preview only" />
        <RepeatableRows
          label="Package examples"
          values={rows}
          onChange={setRows}
        />
      </FormSection>
      <DataGrid
        label="Pattern demo products"
        rows={products}
        rowKey={(p) => p.id}
        searchText={(p) => p.name}
        pageSize={3}
        filters={[
          {
            label: "Statuses",
            options: ["Active", "Inactive"],
            value: (p) => p.status,
          },
        ]}
        columns={[
          { key: "name", header: "Product", render: (p) => p.name },
          {
            key: "status",
            header: "Status",
            render: (p) => <Status value={p.status} />,
          },
        ]}
      />
      <Card>
        <SectionHeader title="Workflow & output previews" />
        <div className="row">
          <Button onClick={() => setOpen("workflow")}>
            Workflow dialog example
          </Button>
          <Button onClick={() => setOpen("output")}>
            Output preview example
          </Button>
        </div>
      </Card>
      <WorkflowDialog
        title="Reusable workflow form"
        open={open === "workflow"}
        onClose={() => setOpen("")}
        description="Shared validated form, used by contact, stock, branch and system actions."
        fields={[
          { name: "reference", label: "Required reference", required: true },
          { name: "reason", label: "Reason", type: "textarea" },
        ]}
        onSubmit={() => setOpen("")}
      />
      <OutputPreview
        title="Output preview example"
        open={open === "output"}
        onClose={() => setOpen("")}
      >
        <Alert title="Preview only">
          Printing and exporting remain disconnected.
        </Alert>
      </OutputPreview>
    </div>
  );
}
