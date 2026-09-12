import { useState } from "react";
import { ChartNoAxesColumnIncreasing, TrendingUp } from "lucide-react";
import { Card, SectionHeader, Select, Badge } from "../../design-system";
import { demoChart } from "../../data/demo";
export function SalesOverview() {
  const [period, setPeriod] = useState("week");
  const values = period === "week" ? demoChart.week : demoChart.previous;
  return (
    <Card className="sales-overview">
      <SectionHeader
        title="Sales overview"
        icon={ChartNoAxesColumnIncreasing}
        action={
          <Select
            label="Sales period"
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
          >
            <option value="week">This week</option>
            <option value="previous">Previous week</option>
          </Select>
        }
      />
      <div className="chart-summary">
        <strong>
          {period === "week" ? "92,450" : "78,470"} <small>EGP</small>
        </strong>
        <Badge tone="success">
          <TrendingUp size={12} /> {period === "week" ? "17.8%" : "9.2%"}
        </Badge>
        <span className="muted">vs. previous week</span>
        <span className="chart-legend">
          <i /> Sales
        </span>
      </div>
      <div
        className="bar-chart"
        role="img"
        aria-label={`Demo daily sales in thousands of EGP, Monday to Sunday: ${values.join(", ")}`}
      >
        <div className="y-axis">
          {["20k", "15k", "10k", "5k", "0"].map((v) => (
            <span key={v}>{v}</span>
          ))}
        </div>
        <div className="plot">
          <div className="gridlines">
            {[0, 1, 2, 3, 4].map((i) => (
              <span key={i} />
            ))}
          </div>
          <div className="bars">
            {values.map((v, i) => (
              <div className="bar-column" key={i}>
                <div
                  className={`bar ${i === 6 ? "current" : ""}`}
                  style={{ height: `${(v / 20) * 100}%` }}
                  title={`${["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][i]}: ${v * 1000} EGP`}
                />
                <span>
                  {["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][i]}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </Card>
  );
}
