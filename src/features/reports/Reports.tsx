import { useState } from "react";
import { Download, Printer } from "lucide-react";
import {
  FeaturePage,
  Button,
  Input,
  Select,
  SummaryStrip,
  PriceDisplay,
  Card,
  SectionHeader,
  DataGrid,
  OutputPreview,
  DetailList,
} from "../../design-system";
import { reportCategories, reportRows } from "../../mock/fixtures";
import { useDemo } from "../../app/DemoContext";
export function Reports() {
  const { branches } = useDemo();
  const [category, setCategory] = useState("Sales"),
    [branch, setBranch] = useState("All branches"),
    [from, setFrom] = useState("2026-09-01"),
    [to, setTo] = useState("2026-09-11"),
    [output, setOutput] = useState("");
  const rows = reportRows.filter(
    (r) =>
      r.category === category &&
      (branch === "All branches" || r.label === branch),
  );
  return (
    <FeaturePage
      title="Reports"
      description="A reporting workspace for each part of pharmacy operations."
      actions={
        <>
          <Button onClick={() => setOutput("Export report")}>
            <Download size={15} />
            Export
          </Button>
          <Button onClick={() => setOutput("Print report")}>
            <Printer size={15} />
            Print
          </Button>
        </>
      }
    >
      <div className="section-menu">
        {reportCategories.map((c) => (
          <Button
            key={c}
            variant={category === c ? "primary" : "secondary"}
            onClick={() => setCategory(c)}
          >
            {c}
          </Button>
        ))}
      </div>
      <Card>
        <div className="form-grid">
          <Input
            label="From date"
            type="date"
            value={from}
            max={to}
            onChange={(e) => setFrom(e.target.value)}
          />
          <Input
            label="To date"
            type="date"
            value={to}
            min={from}
            onChange={(e) => setTo(e.target.value)}
          />
          <Select
            label="Report branch"
            value={branch}
            onChange={(e) => setBranch(e.target.value)}
          >
            <option>All branches</option>
            {branches.map((b) => (
              <option key={b.id}>{b.name}</option>
            ))}
          </Select>
          <p className="muted">
            Date fields define a preview request. The sample totals do not
            recalculate from real records.
          </p>
        </div>
      </Card>
      <SummaryStrip
        items={[
          {
            label: `${category} value · sample`,
            value: <PriceDisplay amount={18450} />,
          },
          { label: "Records · sample", value: 142 },
          {
            label: "Comparison · sample",
            value: "+12%",
            hint: "Illustrative prior-period comparison",
          },
        ]}
      />
      <Card>
        <SectionHeader
          title={`${category} overview`}
          subtitle={`${from} → ${to} · ${branch}`}
        />
        <div
          className="report-chart"
          role="img"
          aria-label={`${category} sample chart: four illustrative values, no live calculations`}
        >
          {rows.map((r, i) => (
            <div key={r.id} style={{ height: `${45 + i * 14}%` }}>
              <span>{r.label}</span>
            </div>
          ))}
        </div>
      </Card>
      <DataGrid
        label={`${category} report`}
        rows={rows}
        rowKey={(r) => r.id}
        searchText={(r) => r.label}
        columns={[
          { key: "branch", header: "Branch", render: (r) => r.label },
          { key: "period", header: "Sample date", render: (r) => r.period },
          {
            key: "transactions",
            header: "Records",
            render: (r) => r.transactions,
          },
          {
            key: "value",
            header: "Value (illustrative)",
            render: (r) => <PriceDisplay amount={r.amount} />,
          },
        ]}
      />
      <OutputPreview
        title={output}
        open={!!output}
        onClose={() => setOutput("")}
      >
        <DetailList
          items={[
            { label: "Report", value: category },
            { label: "Branch", value: branch },
            { label: "Date range", value: `${from} → ${to}` },
            { label: "Rows in preview", value: rows.length },
          ]}
        />
      </OutputPreview>
    </FeaturePage>
  );
}
