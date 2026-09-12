import { useState } from "react";
import {
  FeaturePage,
  RouteTabs,
  DataGrid,
  Button,
  Status,
  Modal,
  Select,
  Input,
  Table,
  Alert,
  WorkflowDialog,
  DetailList,
  ConfirmationModal,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { countSessions } from "../../mock/fixtures";
import { inventoryTabs } from "./Inventory";
export function StockCounts() {
  const { products, branches, branch, notify } = useDemo();
  const [sessions, setSessions] = useState(countSessions),
    [active, setActive] = useState(""),
    [newOpen, setNewOpen] = useState(false),
    [counts, setCounts] = useState<Record<string, number>>({}),
    [review, setReview] = useState(false),
    [complete, setComplete] = useState(false),
    [category, setCategory] = useState("All categories");
  const session = sessions.find((s) => s.id === active);
  const rows = products.filter(
    (p) => category === "All categories" || p.category === category,
  );
  return (
    <FeaturePage
      title="Stock counts"
      description="Count, compare, and review differences before a stock correction."
      actions={
        <Button variant="primary" onClick={() => setNewOpen(true)}>
          New count session
        </Button>
      }
    >
      <RouteTabs items={inventoryTabs} />
      <DataGrid
        label="Count sessions"
        rows={sessions}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.id} ${r.branch} ${r.category}`}
        filters={[
          {
            label: "Statuses",
            options: ["Draft", "In Progress", "Completed"],
            value: (r) => r.status,
          },
        ]}
        columns={[
          {
            key: "id",
            header: "Session",
            render: (r) => (
              <Button
                variant="ghost"
                onClick={() => {
                  setActive(r.id);
                  setCategory(r.category);
                  setCounts({});
                  setReview(false);
                }}
              >
                {r.id}
              </Button>
            ),
          },
          { key: "branch", header: "Branch", render: (r) => r.branch },
          { key: "category", header: "Scope", render: (r) => r.category },
          { key: "date", header: "Started", render: (r) => r.date },
          { key: "items", header: "Items", render: (r) => r.items },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
        ]}
      />
      <WorkflowDialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        title="New stock count"
        description="Start a sample count without locking or changing actual stock."
        fields={[
          {
            name: "branch",
            label: "Count branch",
            options: branches.map((b) => b.name),
            value: branch,
          },
          {
            name: "category",
            label: "Category scope",
            options: [
              "All categories",
              ...new Set(products.map((p) => p.category)),
            ],
          },
        ]}
        onSubmit={(v) => {
          const id = `SC-${Date.now().toString().slice(-4)}`;
          setSessions([
            {
              id,
              branch: v.branch,
              category: v.category,
              date: "2026-09-11",
              status: "Draft",
              items: products.filter(
                (p) =>
                  v.category === "All categories" || p.category === v.category,
              ).length,
            },
            ...sessions,
          ]);
          setActive(id);
          setCategory(v.category);
          setCounts({});
          setReview(false);
        }}
      />
      <Modal
        size="lg"
        open={!!session}
        onOpenChange={() => setActive("")}
        title={`Count ${active}`}
        description="Expected quantities are sample stock. No stock changes are posted."
      >
        <div className="stack">
          <DetailList
            items={[
              { label: "Branch", value: session?.branch },
              { label: "Status", value: session?.status },
            ]}
          />
          {session?.status === "Completed" ? (
            <Alert title="Count completed in this preview" tone="success">
              This session is read-only.
            </Alert>
          ) : (
            <>
              <Select
                label="Count category"
                value={category}
                onChange={(e) => {
                  setCategory(e.target.value);
                  setCounts({});
                  setReview(false);
                }}
              >
                <option>All categories</option>
                {[...new Set(products.map((p) => p.category))].map((c) => (
                  <option key={c}>{c}</option>
                ))}
              </Select>
              <Table
                label="Count quantities"
                rows={
                  review
                    ? rows.filter((p) => (counts[p.id] ?? p.stock) !== p.stock)
                    : rows
                }
                rowKey={(p) => p.id}
                columns={[
                  { key: "name", header: "Product", render: (p) => p.name },
                  {
                    key: "expected",
                    header: "Expected",
                    render: (p) => p.stock,
                  },
                  {
                    key: "count",
                    header: "Counted",
                    render: (p) => (
                      <Input
                        hideLabel
                        label={`Count ${p.name}`}
                        type="number"
                        min={0}
                        value={counts[p.id] ?? p.stock}
                        onChange={(e) => {
                          setCounts({
                            ...counts,
                            [p.id]: Math.max(0, Number(e.target.value)),
                          });
                          setSessions(
                            sessions.map((s) =>
                              s.id === active
                                ? { ...s, status: "In Progress" }
                                : s,
                            ),
                          );
                        }}
                      />
                    ),
                  },
                  {
                    key: "variance",
                    header: "Variance",
                    render: (p) => (counts[p.id] ?? p.stock) - p.stock,
                  },
                ]}
              />
              <div className="row">
                <Button onClick={() => setReview(!review)}>
                  {review ? "Show all items" : "Review differences"}
                </Button>
                <Button
                  variant="primary"
                  disabled={!review}
                  onClick={() => setComplete(true)}
                >
                  Complete count
                </Button>
              </div>
            </>
          )}
        </div>
      </Modal>
      <ConfirmationModal
        open={complete}
        onOpenChange={setComplete}
        title="Complete demo count?"
        description="Review differences before completion. This only updates the session status; no inventory correction is posted."
        onConfirm={() => {
          setSessions(
            sessions.map((s) =>
              s.id === active ? { ...s, status: "Completed" } : s,
            ),
          );
          notify("Demo count completed. Inventory remains unchanged.");
        }}
      />
    </FeaturePage>
  );
}
