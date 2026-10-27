import { useEffect, useMemo, useState } from "react";
import { Button } from "../../design-system";
import { useLocalization } from "../../localization/LocalizationContext";

export const guidedTutorialStorageKey = "m2-health.tutorial.desktop.v1";

type TutorialState = { dismissed: boolean; completed: boolean };
const initialState: TutorialState = { dismissed: false, completed: false };

function readTutorialState(): TutorialState {
  try {
    const stored = JSON.parse(window.localStorage.getItem(guidedTutorialStorageKey) ?? "null") as Partial<TutorialState> | null;
    return stored ? { ...initialState, ...stored } : initialState;
  } catch {
    return initialState;
  }
}

function saveTutorialState(next: TutorialState) {
  try {
    window.localStorage.setItem(guidedTutorialStorageKey, JSON.stringify(next));
  } catch {
    // The tour remains usable for the current session when storage is unavailable.
  }
}

export function shouldStartGuidedTutorial() {
  return !readTutorialState().dismissed;
}

const steps = [
  { target: "navigation", title: "tutorial.navigationTitle", copy: "tutorial.navigationCopy" },
  { target: "branch", title: "tutorial.branchTitle", copy: "tutorial.branchCopy" },
  { target: "profile-menu", title: "tutorial.setupTitle", copy: "tutorial.setupCopy" },
  { target: "nav-catalog", title: "tutorial.catalogTitle", copy: "tutorial.catalogCopy" },
  { target: "nav-purchases", title: "tutorial.purchasingTitle", copy: "tutorial.purchasingCopy" },
  { target: "nav-sales", title: "tutorial.salesTitle", copy: "tutorial.salesCopy" },
  { target: "nav-reports", title: "tutorial.reportsTitle", copy: "tutorial.reportsCopy" },
] as const;

export function GuidedTutorial({ onOpenChange }: { onOpenChange(open: boolean): void }) {
  const { t } = useLocalization();
  const [step, setStep] = useState(0);
  const [rect, setRect] = useState<DOMRect | null>(null);
  const current = steps[step];

  useEffect(() => {
    const update = () => {
      const element = document.querySelector<HTMLElement>(`[data-tutorial="${current.target}"]`);
      setRect(element?.getBoundingClientRect() ?? null);
    };
    update();
    window.addEventListener("resize", update);
    document.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      document.removeEventListener("scroll", update, true);
    };
  }, [current.target]);

  const spotlightStyle = useMemo(() => rect ? {
    left: Math.max(8, rect.left - 5),
    top: Math.max(8, rect.top - 5),
    width: Math.min(window.innerWidth - 16, rect.width + 10),
    height: Math.min(window.innerHeight - 16, rect.height + 10),
  } : undefined, [rect]);

  const finish = (completed: boolean) => {
    saveTutorialState({ dismissed: true, completed });
    onOpenChange(false);
  };
  const last = step === steps.length - 1;

  return (
    <aside className="guided-tutorial" aria-label={t("tutorial.label")} aria-live="polite">
      {rect && <div className="guided-tutorial__spotlight" style={spotlightStyle} aria-hidden="true" />}
      <div className="guided-tutorial__panel" role="dialog" aria-modal="false" aria-labelledby="guided-tutorial-title">
        <div className="guided-tutorial__progress">
          <span>{t("tutorial.progress", { current: step + 1, total: steps.length })}</span>
          <Button size="sm" variant="ghost" onClick={() => finish(false)}>{t("tutorial.skip")}</Button>
        </div>
        <h2 id="guided-tutorial-title">{t(current.title)}</h2>
        <p>{t(current.copy)}</p>
        <div className="guided-tutorial__actions">
          <Button disabled={step === 0} onClick={() => setStep((value) => value - 1)}>{t("tutorial.back")}</Button>
          <Button variant="primary" onClick={() => last ? finish(true) : setStep((value) => value + 1)}>
            {last ? t("tutorial.finish") : t("tutorial.next")}
          </Button>
        </div>
      </div>
    </aside>
  );
}
