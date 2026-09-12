import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Plus, Pencil } from "lucide-react";
import {
  FeaturePage,
  DataGrid,
  PriceDisplay,
  Status,
  Button,
  WorkflowDialog,
  BackLink,
  Card,
  DetailList,
  Tabs,
  Table,
  EmptyState,
  LinkButton,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import type { Partner } from "../../mock/types";
export function Partners({ kind }: { kind: "suppliers" | "customers" }) {
  const demo = useDemo();
  const rows = demo[kind],
    setRows = kind === "suppliers" ? demo.setSuppliers : demo.setCustomers;
  const [open, setOpen] = useState(false);
  const title = kind === "suppliers" ? "Suppliers" : "Customers";
  return (
    <FeaturePage
      title={title}
      description={
        kind === "suppliers"
          ? "Supplier contacts, purchasing history, and outstanding balances."
          : "Customer relationships, purchase history, and account notes."
      }
      actions={
        <Button variant="primary" onClick={() => setOpen(true)}>
          <Plus size={15} />
          Add {kind === "suppliers" ? "supplier" : "customer"}
        </Button>
      }
    >
      <DataGrid
        label={title}
        rows={rows}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.name} ${r.phone} ${r.email}`}
        filters={[
          {
            label: "Statuses",
            options: ["Active", "On hold"],
            value: (r) => r.status,
          },
        ]}
        columns={[
          {
            key: "name",
            header: "Name / contact",
            render: (r) => (
              <Link className="text-link" to={`/${kind}/${r.id}`}>
                {r.name}
                <span className="cell-secondary">{r.contact}</span>
              </Link>
            ),
          },
          {
            key: "phone",
            header: "Phone / email",
            render: (r) => (
              <>
                {r.phone}
                <span className="cell-secondary">{r.email}</span>
              </>
            ),
          },
          { key: "address", header: "Address", render: (r) => r.address },
          {
            key: "balance",
            header: "Outstanding balance",
            render: (r) => <PriceDisplay amount={r.balance} />,
          },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
        ]}
      />
      <PartnerEditor
        title={`Add ${kind === "suppliers" ? "supplier" : "customer"}`}
        open={open}
        onClose={() => setOpen(false)}
        onSave={(p) => {
          setRows([p, ...rows]);
          demo.notify("Contact added to the demo workspace.");
        }}
      />
    </FeaturePage>
  );
}
function PartnerEditor({
  title,
  open,
  onClose,
  partner,
  onSave,
}: {
  title: string;
  open: boolean;
  onClose: () => void;
  partner?: Partner;
  onSave: (p: Partner) => void;
}) {
  return (
    <WorkflowDialog
      title={title}
      open={open}
      onClose={onClose}
      description="Contact information is stored in this UI session only."
      fields={[
        { name: "name", label: "Name", value: partner?.name, required: true },
        { name: "contact", label: "Contact person", value: partner?.contact },
        {
          name: "phone",
          label: "Phone",
          type: "tel",
          value: partner?.phone,
          required: true,
        },
        { name: "email", label: "Email", type: "email", value: partner?.email },
        { name: "address", label: "Address", value: partner?.address },
        {
          name: "notes",
          label: "Notes",
          type: "textarea",
          value: partner?.notes,
        },
      ]}
      onSubmit={(v) =>
        onSave({
          id: partner?.id ?? `contact-${Date.now()}`,
          name: v.name,
          contact: v.contact || v.name,
          phone: v.phone,
          email: v.email,
          address: v.address,
          notes: v.notes,
          balance: partner?.balance ?? 0,
          status: partner?.status ?? "Active",
        })
      }
    />
  );
}
export function PartnerDetails({ kind }: { kind: "suppliers" | "customers" }) {
  const { id } = useParams();
  const demo = useDemo();
  const rows = demo[kind],
    setRows = kind === "suppliers" ? demo.setSuppliers : demo.setCustomers;
  const p = rows.find((r) => r.id === id);
  const [edit, setEdit] = useState(false);
  if (!p)
    return (
      <EmptyState
        title="Contact not found"
        description="Choose a contact from the directory."
        action={<LinkButton to={`/${kind}`}>Back to directory</LinkButton>}
      />
    );
  const supplier = kind === "suppliers";
  const history = supplier
    ? demo.purchases
        .filter((r) => r.supplier === p.name)
        .map((r) => ({
          id: r.id,
          date: r.date,
          total: r.total,
          status: r.status,
          to: `/purchases/${r.id}`,
        }))
    : demo.sales
        .filter((r) => r.customer === p.name)
        .map((r) => ({
          id: r.id,
          date: r.time,
          total: r.total,
          status: r.status,
          to: `/sales/${r.id}`,
        }));
  return (
    <>
      <BackLink to={`/${kind}`} label={supplier ? "Suppliers" : "Customers"} />
      <FeaturePage
        title={p.name}
        description={
          supplier ? "Supplier account overview" : "Customer account overview"
        }
        actions={
          <Button onClick={() => setEdit(true)}>
            <Pencil size={15} />
            Edit contact
          </Button>
        }
      >
        <div className="content-stack">
          <Card>
            <DetailList
              items={[
                { label: "Contact", value: p.contact },
                { label: "Phone", value: p.phone },
                { label: "Email", value: p.email },
                { label: "Address", value: p.address },
                {
                  label: "Outstanding balance · demo",
                  value: <PriceDisplay amount={p.balance} />,
                },
                { label: "Status", value: <Status value={p.status} /> },
              ]}
            />
          </Card>
          <Card>
            <Tabs
              items={[
                {
                  label: "Purchase history",
                  content: (
                    <Table
                      label="Contact purchase history"
                      rows={history}
                      rowKey={(r) => r.id}
                      columns={[
                        {
                          key: "id",
                          header: "Reference",
                          render: (r) => (
                            <Link className="text-link" to={r.to}>
                              {r.id}
                            </Link>
                          ),
                        },
                        { key: "date", header: "Date", render: (r) => r.date },
                        {
                          key: "total",
                          header: "Total",
                          render: (r) => <PriceDisplay amount={r.total} />,
                        },
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
                  label: supplier ? "Invoices" : "Balance / account",
                  content: (
                    <DetailList
                      items={[
                        {
                          label: "Outstanding",
                          value: <PriceDisplay amount={p.balance} />,
                        },
                        {
                          label: "Payment terms",
                          value: supplier
                            ? "30 days · sample"
                            : "Account settlement · sample",
                        },
                        {
                          label: "Posting status",
                          value: "No accounting integration",
                        },
                        { label: "Last reviewed", value: "11 Sep 2026 · demo" },
                      ]}
                    />
                  ),
                },
                {
                  label: "Returns",
                  content: (
                    <EmptyState
                      title="No returns in this sample"
                      description="Approved return documents will appear here in a future connected workflow."
                    />
                  ),
                },
                ...(supplier
                  ? [
                      {
                        label: "Products supplied",
                        content: (
                          <Table
                            label="Supplier products"
                            rows={demo.products.slice(0, 4)}
                            rowKey={(r) => r.id}
                            columns={[
                              {
                                key: "name",
                                header: "Product",
                                render: (r: (typeof demo.products)[number]) => (
                                  <Link
                                    className="text-link"
                                    to={`/catalog/${r.id}`}
                                  >
                                    {r.name}
                                  </Link>
                                ),
                              },
                              {
                                key: "cost",
                                header: "Last sample cost",
                                render: (r: (typeof demo.products)[number]) => (
                                  <PriceDisplay amount={r.cost} />
                                ),
                              },
                            ]}
                          />
                        ),
                      },
                    ]
                  : []),
                {
                  label: "Notes",
                  content: <p>{p.notes || "No notes added."}</p>,
                },
                {
                  label: "Activity",
                  content: (
                    <DetailList
                      items={[
                        {
                          label: "11 Sep 2026, 10:20",
                          value: "Contact reviewed · sample event",
                        },
                        {
                          label: "10 Sep 2026, 14:35",
                          value: "Account notes updated · sample event",
                        },
                      ]}
                    />
                  ),
                },
              ]}
            />
          </Card>
        </div>
        <PartnerEditor
          title="Edit contact"
          open={edit}
          onClose={() => setEdit(false)}
          partner={p}
          onSave={(updated) => {
            setRows(rows.map((r) => (r.id === p.id ? updated : r)));
            demo.notify("Contact updated in this UI session.");
          }}
        />
      </FeaturePage>
    </>
  );
}
