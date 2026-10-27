import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Alert, Button, Card, EmptyState, Input, PageHeader, Select, StatCard } from "../../design-system";
import { BarChart3, Boxes, CircleDollarSign, PackageCheck } from "lucide-react";
import { useBranch } from "../../app/BranchContext";
import { phase6, type Phase6Dashboard } from "../../lib/tauri/phase6";
import { useLocalization } from "../../localization/LocalizationContext";
import { ScopeNotice } from "../../app/ScopeNotice";
import "./canonical-dashboard.css";

const errorText = (error: unknown) => error instanceof Error
  ? error.message
  : typeof error === "object" && error && "message" in error
    ? String(error.message)
    : "Dashboard data could not be loaded.";

export function CanonicalDashboard() {
  const { branches, branchId } = useBranch();
  const { t } = useLocalization();
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [branch, setBranch] = useState("");
  const [applied, setApplied] = useState({ dateFrom: "", dateTo: "", branch: "" });
  const [value, setValue] = useState<Phase6Dashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const filter = useMemo(() => ({
    dateFrom: applied.dateFrom || null,
    dateTo: applied.dateTo || null,
    branchId: applied.branch || null,
  }), [applied]);
  useEffect(() => {
    let active = true;
    Promise.resolve().then(() => {
      if (active) {
        setLoading(true);
        setError("");
      }
    });
    phase6.dashboard(filter)
      .then((result) => active && setValue(result))
      .catch((reason) => active && setError(errorText(reason)))
      .finally(() => active && setLoading(false));
    return () => { active = false; };
  }, [attempt, filter]);
  const reset = () => {
    setDateFrom("");
    setDateTo("");
    setBranch("");
    setApplied({ dateFrom: "", dateTo: "", branch: "" });
  };
  const branchName = branches.find((item) => item.id === branchId)?.name ?? branchId;
  return <div className="stack canonical-dashboard">
    <PageHeader title={t("dashboard.title")} description={t("dashboard.description")} actions={<Link className="button button--primary" to="/sales">{t("dashboard.newSale")}</Link>} />
    <Card>
      <div className="dashboard-filters">
        <Input label={t("dashboard.fromDate")} type="date" value={dateFrom} max={dateTo || undefined} onChange={(event) => setDateFrom(event.target.value)} />
        <Input label={t("dashboard.toDate")} type="date" value={dateTo} min={dateFrom || undefined} onChange={(event) => setDateTo(event.target.value)} />
        <Select label={t("dashboard.branch")} showLabel value={branch} onChange={(event) => setBranch(event.target.value)}>
          <option value="">{t("dashboard.allBranches")}</option>
          {branches.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}
        </Select>
        <Button variant="primary" onClick={() => setApplied({ dateFrom, dateTo, branch })}>{t("dashboard.apply")}</Button>
        <Button onClick={reset}>{t("dashboard.reset")}</Button>
      </div>
      <p className="muted">{t("dashboard.filterHelp", { branch: branchName })}</p>
    </Card>
    <ScopeNotice kind="report" reportScope={applied.branch ? branches.find((item) => item.id === applied.branch)?.name ?? applied.branch : t("scope.allBranches")} />
    {error && <Alert title={t("dashboard.loadError")} tone="danger">{error} <Button onClick={() => setAttempt((current) => current + 1)}>{t("common.retry")}</Button></Alert>}
    {loading && <p role="status">{t("dashboard.loading")}</p>}
    {!loading && !error && value && <DashboardBody value={value} />}
  </div>;
}

function DashboardBody({ value }: { value: Phase6Dashboard }) {
  const { currencyMinor, locale, number, t } = useLocalization();
  const operational = value.operational;
  const financial = value.financial;
  const coverage = value.coverage;
  const money = (amount: number | null | undefined) => amount == null ? t("common.unavailable") : currencyMinor(amount);
  const empty = operational.postedSalesCount === 0 && operational.currentStockUnits === 0 && operational.purchaseOrderCount === 0;
  return <>
    {empty && <EmptyState title={t("empty.dashboardTitle")} description={t("empty.dashboardCopy")} />}
    <section>
      <h2>{t("dashboard.operational")}</h2>
      <div className="canonical-dashboard__stats">
        <StatCard label={t("dashboard.netSales")} value={money(operational.netSalesMinor)} detail={t("dashboard.postedSales", { count: number(operational.postedSalesCount), units: number(operational.netUnitsSold) })} icon={CircleDollarSign} />
        <StatCard label={t("dashboard.currentStock")} value={number(operational.currentStockUnits)} detail={t("dashboard.lowStock", { count: number(operational.lowStockPackageCount) })} icon={Boxes} />
        <StatCard label={t("dashboard.received")} value={number(operational.receivedQuantity)} detail={t("dashboard.purchaseOrders", { count: number(operational.purchaseOrderCount) })} icon={PackageCheck} />
        <StatCard label={t("dashboard.supplierReturns")} value={number(operational.purchaseReturnQuantity)} detail={t("dashboard.returnUnits")} icon={PackageCheck} />
      </div>
    </section>
    <section>
      <div className="canonical-dashboard__heading"><h2>{t("dashboard.finance")}</h2><Link to="/finance">{t("dashboard.openFinance")}</Link></div>
      <div className="canonical-dashboard__stats">
        <StatCard label={t("dashboard.netRevenue")} value={money(financial.netSalesRevenueMinor)} detail={t("dashboard.financeJournals")} icon={CircleDollarSign} />
        <StatCard label={t("dashboard.netCogs")} value={money(financial.netCogsMinor)} detail={t("dashboard.financeJournals")} icon={BarChart3} />
        <StatCard label={t("dashboard.grossMargin")} value={money(financial.grossMarginMinor)} detail={financial.grossMarginMinor == null ? t("dashboard.incompleteCost") : t("dashboard.limitedScope")} icon={CircleDollarSign} />
        <StatCard label={t("dashboard.operatingResult")} value={money(financial.operatingResultMinor)} detail={t("dashboard.resultDetail")} icon={CircleDollarSign} />
      </div>
    </section>
    <Alert title={coverage.coverageComplete ? t("dashboard.coverageComplete") : t("dashboard.coveragePartial")} tone={coverage.coverageComplete ? "success" : "warning"}>
      {locale === "en" ? `${coverage.scopeNote} ` : ""}{t("dashboard.coverageCounts", {
        sales: number(coverage.representedSaleCount),
        eligibleSales: number(coverage.eligibleSaleCount),
        returns: number(coverage.representedReturnCount),
        postedReturns: number(coverage.postedReturnCount),
        voids: number(coverage.representedVoidCount),
        voidCount: number(coverage.voidCount),
      })} <Link to="/reports">{t("dashboard.compare")}</Link>.
    </Alert>
  </>;
}
