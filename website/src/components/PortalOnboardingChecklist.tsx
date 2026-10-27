import { CheckCircle2, Circle } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import type { PortalSnapshot } from "../portal/types";
import { useAuth } from "../auth/AuthContext";
import { useLocalization } from "../localization/LocalizationContext";

export function PortalOnboardingChecklist({ snapshot }: { snapshot: PortalSnapshot }) {
  const { user } = useAuth();
  const { t } = useLocalization();
  const trialComplete = snapshot.status !== "UNACTIVATED";
  const deviceComplete = snapshot.devices.length > 0;
  const activationComplete = snapshot.devices.some((device) => device.status === "ACTIVE");
  const steps = [Boolean(user), trialComplete, deviceComplete, activationComplete];
  const completed = steps.filter(Boolean).length;
  return (
    <section className="portal-card portal-onboarding" aria-labelledby="portal-onboarding-title">
      <div className="portal-onboarding__heading">
        <div><h2 id="portal-onboarding-title">{t("portalChecklist.title")}</h2><p>{t("portalChecklist.description")}</p></div>
        <strong>{t("onboarding.progress", { complete: completed, total: steps.length })}</strong>
      </div>
      <progress max={steps.length} value={completed} aria-label={t("onboarding.progress", { complete: completed, total: steps.length })} />
      <div className="portal-onboarding__steps">
        <PortalStep complete={Boolean(user)} title={t("portalChecklist.account")} copy={t("portalChecklist.accountCopy")} />
        <PortalStep complete={trialComplete} title={t("portalChecklist.trial")} copy={t("portalChecklist.trialCopy")}>
          {!trialComplete && <a className="text-link" href="#portal-start-trial">{t("portalChecklist.startTrial")}</a>}
        </PortalStep>
        <PortalStep complete={deviceComplete} title={t("portalChecklist.device")} copy={t("portalChecklist.deviceCopy")}>
          {!deviceComplete && trialComplete && <Link className="text-link" to="/portal/devices">{t("portalChecklist.manageDevices")}</Link>}
        </PortalStep>
        <PortalStep complete={activationComplete} title={t("portalChecklist.activation")} copy={t("portalChecklist.activationCopy")}>
          <p className={`portal-onboarding__recovery ${activationComplete ? "success" : ""}`}>{activationComplete ? t("portalChecklist.activationReady") : t("portalChecklist.activationNeeded")}</p>
          {!activationComplete && trialComplete && <Link className="text-link" to="/portal/devices">{t("portalChecklist.manageDevices")}</Link>}
        </PortalStep>
      </div>
    </section>
  );
}

function PortalStep({ complete, title, copy, children }: { complete: boolean; title: string; copy: string; children?: ReactNode }) {
  const { t } = useLocalization();
  return (
    <article className={complete ? "is-complete" : ""}>
      <span className="portal-onboarding__icon" aria-hidden="true">{complete ? <CheckCircle2 /> : <Circle />}</span>
      <div><div className="portal-onboarding__step-heading"><h3>{title}</h3><span>{complete ? t("onboarding.complete") : t("onboarding.incomplete")}</span></div><p>{copy}</p>{children}</div>
    </article>
  );
}
