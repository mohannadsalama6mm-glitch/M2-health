import { CheckCircle2, Circle, ExternalLink } from "lucide-react";
import { useState, type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { Button, Modal } from "../../design-system";
import { useAuth } from "../../app/AuthContext";
import { useBranch } from "../../app/BranchContext";
import { useLicense } from "../../app/LicenseContext";
import { useLocalization } from "../../localization/LocalizationContext";

export const desktopOnboardingStorageKey = "m2-health.onboarding.desktop.v1";

interface SavedSetup {
  dismissed: boolean;
  portalAccountConfirmed: boolean;
  essentialConfirmed: boolean;
}

const initialSavedSetup: SavedSetup = {
  dismissed: false,
  portalAccountConfirmed: false,
  essentialConfirmed: false,
};

function readSavedSetup(): SavedSetup {
  try {
    const value = JSON.parse(window.localStorage.getItem(desktopOnboardingStorageKey) ?? "null") as Partial<SavedSetup> | null;
    return value ? { ...initialSavedSetup, ...value } : initialSavedSetup;
  } catch {
    return initialSavedSetup;
  }
}

export function shouldOpenDesktopOnboarding() {
  return !readSavedSetup().dismissed;
}

export function DesktopOnboarding({ open, onOpenChange }: { open: boolean; onOpenChange(open: boolean): void }) {
  const auth = useAuth();
  const branch = useBranch();
  const license = useLicense();
  const { locale, setLocale, state: stateLabel, t } = useLocalization();
  const navigate = useNavigate();
  const [saved, setSaved] = useState(readSavedSetup);
  const updateSaved = (patch: Partial<SavedSetup>) => {
    const next = { ...saved, ...patch };
    setSaved(next);
    try {
      window.localStorage.setItem(desktopOnboardingStorageKey, JSON.stringify(next));
    } catch {
      // Keep the current session usable if storage is unavailable.
    }
  };
  const licenseComplete = Boolean(license.status?.canOperate && !license.status.requiresActivation);
  const branchComplete = !branch.loading && Boolean(branch.branchId && branch.branches.length);
  const steps = [
    true,
    Boolean(auth.user),
    saved.portalAccountConfirmed,
    licenseComplete,
    branchComplete,
    saved.essentialConfirmed,
  ];
  const completed = steps.filter(Boolean).length;
  const go = (path: string) => {
    onOpenChange(false);
    navigate(path);
  };
  const closeForNow = () => {
    updateSaved({ dismissed: true });
    onOpenChange(false);
  };
  return (
    <Modal open={open} onOpenChange={onOpenChange} title={t("onboarding.title")} description={t("onboarding.description")} size="lg">
      <div className="onboarding-progress" aria-label={t("onboarding.progress", { complete: completed, total: steps.length })}>
        <strong>{t("onboarding.progress", { complete: completed, total: steps.length })}</strong>
        <progress max={steps.length} value={completed} />
      </div>
      <div className="onboarding-checklist">
        <SetupStep complete title={t("onboarding.language")} copy={t("onboarding.languageCopy")} t={t}>
          <select aria-label={t("language.label")} value={locale} onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}>
            <option value="en">{t("language.english")}</option>
            <option value="ar">{t("language.arabic")}</option>
          </select>
        </SetupStep>
        <SetupStep complete={Boolean(auth.user)} title={t("onboarding.localAdmin")} copy={t("onboarding.localAdminCopy")} t={t} />
        <SetupStep complete={saved.portalAccountConfirmed} title={t("onboarding.portalAccount")} copy={t("onboarding.portalAccountCopy")} t={t}>
          <div className="onboarding-actions">
            <a className="button button--secondary" href="https://m2-health.vercel.app/register" target="_blank" rel="noreferrer">{t("onboarding.openPortal")}<ExternalLink size={15} /></a>
            <label><input type="checkbox" checked={saved.portalAccountConfirmed} onChange={(event) => updateSaved({ portalAccountConfirmed: event.target.checked })} />{t("onboarding.portalAccountConfirm")}</label>
          </div>
        </SetupStep>
        <SetupStep complete={licenseComplete} title={t("onboarding.license")} copy={t("onboarding.licenseCopy")} t={t}>
          <span className="onboarding-state">{license.status ? stateLabel(license.status.state) : t("common.loading")}</span>
          {!licenseComplete && <p className="onboarding-recovery">{t("onboarding.activationAction")}</p>}
        </SetupStep>
        <SetupStep complete={branchComplete} title={t("onboarding.branch")} copy={t("onboarding.branchCopy")} t={t}>
          {branchComplete && <span className="onboarding-state">{branch.branchName}</span>}
        </SetupStep>
        <SetupStep complete={saved.essentialConfirmed} title={t("onboarding.essential")} copy={t("onboarding.essentialCopy")} t={t}>
          <div className="onboarding-actions">
            <Button onClick={() => go("/settings")}>{t("onboarding.settingsAction")}</Button>
            <Button onClick={() => go("/catalog")}>{t("onboarding.catalogAction")}</Button>
            <Button onClick={() => go("/backup")}>{t("onboarding.backupAction")}</Button>
            <label><input type="checkbox" checked={saved.essentialConfirmed} onChange={(event) => updateSaved({ essentialConfirmed: event.target.checked })} />{t("onboarding.essentialConfirm")}</label>
          </div>
        </SetupStep>
      </div>
      <p className="onboarding-saved">{t("onboarding.saved")}</p>
      <div className="onboarding-footer">
        <Button onClick={closeForNow}>{t("onboarding.skip")}</Button>
        <Button variant="primary" onClick={() => { closeForNow(); navigate("/dashboard"); }}>{t("onboarding.dashboard")}</Button>
      </div>
    </Modal>
  );
}

function SetupStep({ complete, title, copy, t, children }: { complete: boolean; title: string; copy: string; t: ReturnType<typeof useLocalization>["t"]; children?: ReactNode }) {
  return (
    <section className={`onboarding-step ${complete ? "is-complete" : ""}`}>
      <div className="onboarding-step__icon" aria-hidden="true">{complete ? <CheckCircle2 /> : <Circle />}</div>
      <div>
        <div className="onboarding-step__heading"><h3>{title}</h3><span>{complete ? t("onboarding.complete") : t("onboarding.incomplete")}</span></div>
        <p>{copy}</p>
        {children}
      </div>
    </section>
  );
}
