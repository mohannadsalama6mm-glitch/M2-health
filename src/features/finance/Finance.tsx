import { useState } from "react";
import {
  FeaturePage,
  Alert,
  Button,
  SummaryStrip,
  PriceDisplay,
  DataGrid,
  Status,
  WorkflowDialog,
} from "../../design-system";
import { financeRows, financeSections } from "../../mock/fixtures";
import { useDemo } from "../../app/DemoContext";
export function Finance() {
  const { notify } = useDemo();
  const [section, setSection] = useState("Overview"),
    [action, setAction] = useState("");
  return (
    <FeaturePage
      title="Finance"
      description="Accounting workspace structure · no ledger or financial engine."
      actions={
        <Button
          onClick={() =>
            setAction(
              section === "Expenses" ? "Record expense" : "Record payment",
            )
          }
        >
          {section === "Expenses" ? "Record expense" : "Record payment"}
        </Button>
      }
    >
      <Alert title="Accounting UI shell" tone="info">
        All balances are illustrative. No entries are balanced, posted, settled,
        or calculated by an accounting system.
      </Alert>
      <div className="section-menu">
        {financeSections.map((s) => (
          <Button
            key={s}
            variant={section === s ? "primary" : "secondary"}
            onClick={() => setSection(s)}
          >
            {s}
          </Button>
        ))}
      </div>
      <SummaryStrip
        items={[
          { label: "Cash · sample", value: <PriceDisplay amount={8450} /> },
          {
            label: "Supplier balances · sample",
            value: <PriceDisplay amount={22050} />,
          },
          {
            label: "Customer balances · sample",
            value: <PriceDisplay amount={415} />,
          },
          { label: "Expenses · sample", value: <PriceDisplay amount={1250} /> },
        ]}
      />
      <DataGrid
        label={section}
        rows={financeRows}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.account} ${r.description}`}
        columns={[
          {
            key: "date",
            header: "Date / reference",
            render: (r) => (
              <>
                {r.date}
                <span className="cell-secondary">{r.id}</span>
              </>
            ),
          },
          {
            key: "account",
            header:
              section === "Chart of Accounts" ? "Account name" : "Account",
            render: (r) => r.account,
          },
          {
            key: "description",
            header: "Description",
            render: (r) => r.description,
          },
          { key: "debit", header: "Debit · sample", render: (r) => r.debit },
          { key: "credit", header: "Credit · sample", render: (r) => r.credit },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
        ]}
      />
      <WorkflowDialog
        open={!!action}
        onClose={() => setAction("")}
        title={action}
        description="Form preview only. No financial record will be created."
        fields={[
          {
            name: "account",
            label: "Account",
            options: [
              "Cash on hand",
              "Bank",
              "Operating expenses",
              "Supplier payables",
            ],
          },
          {
            name: "amount",
            label: "Amount (EGP)",
            type: "number",
            min: 0.01,
            required: true,
          },
          { name: "reference", label: "Reference", required: true },
          {
            name: "notes",
            label: "Description",
            type: "textarea",
            required: true,
          },
        ]}
        submitLabel="Review preview"
        onSubmit={() =>
          notify("Financial form reviewed. No accounting entry was posted.")
        }
      />
    </FeaturePage>
  );
}
