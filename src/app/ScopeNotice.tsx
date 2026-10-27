import { LinkButton } from "../design-system";
import { useLocalization } from "../localization/LocalizationContext";
import { useBranch } from "./BranchContext";

type ScopeKind = "operational" | "report" | "catalog" | "purchasing" | "partners";

export function ScopeNotice({ kind = "operational", reportScope }: { kind?: ScopeKind; reportScope?: string }) {
  const { branchId, branchName } = useBranch();
  const { t } = useLocalization();

  if (!branchId) {
    return (
      <aside className="scope-notice scope-notice--warning" aria-label={t("scope.noBranch")}>
        <div><strong>{t("scope.noBranch")}</strong><p>{t("scope.noBranchCopy")}</p></div>
        <LinkButton to="/branches" primary>{t("scope.manageBranches")}</LinkButton>
      </aside>
    );
  }

  const content = kind === "report"
    ? { title: t("scope.report"), value: reportScope || t("scope.allBranches"), copy: t("scope.reportCopy", { branch: branchName }) }
    : kind === "catalog"
      ? { title: t("scope.catalog"), value: branchName, copy: t("scope.catalogCopy", { branch: branchName }) }
      : kind === "purchasing"
        ? { title: t("scope.purchasing"), value: branchName, copy: t("scope.purchasingCopy", { branch: branchName }) }
        : kind === "partners"
          ? { title: t("scope.partners"), value: branchName, copy: t("scope.partnersCopy") }
          : { title: t("scope.active"), value: branchName, copy: t("scope.operationalCopy") };

  return (
    <aside className="scope-notice" aria-label={content.title}>
      <div className="scope-notice__heading"><strong>{content.title}</strong><span>{content.value}</span></div>
      <p>{content.copy}</p>
    </aside>
  );
}
