import { useEffect, useState } from "react";
import { BrowserRouter } from "react-router-dom";
import { Toast } from "../design-system";
import { DemoProvider } from "./DemoContext";
import { BranchProvider } from "./BranchContext";
import { AppShell } from "./AppShell";
import { AuthProvider } from "./AuthContext";
import { AuthGate } from "../features/auth/AuthGate";
import { FeedbackProvider } from "./FeedbackContext";
import { isDesktopRuntime } from "../lib/tauri/client";
import { LicenseProvider } from "./LicenseContext";
import { LicenseGate } from "../features/licensing/LicenseGate";
import { Alert, Logo } from "../design-system";
import { useLocalization } from "../localization/LocalizationContext";

export function App() {
  const { t } = useLocalization();
  const [toast, setToast] = useState("");
  const desktop = isDesktopRuntime();
  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(""), 5000);
    return () => window.clearTimeout(timer);
  }, [toast]);
  if (!desktop && !import.meta.env.DEV)
    return (
      <main className="auth-screen">
        <section className="auth-card">
          <Logo variant="compact" />
          <Alert title={t("app.desktopRequired")} tone="warning">
            {t("app.desktopRequiredCopy")}
          </Alert>
        </section>
      </main>
    );
  const application = (
    <LicenseProvider>
      <LicenseGate>
        <AuthProvider>
          <AuthGate>
            <BranchProvider>
              <AppShell />
            </BranchProvider>
          </AuthGate>
        </AuthProvider>
      </LicenseGate>
    </LicenseProvider>
  );
  return (
    <BrowserRouter>
      <FeedbackProvider notify={setToast}>
        {desktop ? (
          application
        ) : (
          <DemoProvider notify={setToast}>{application}</DemoProvider>
        )}
        {toast && <Toast message={toast} onClose={() => setToast("")} />}
      </FeedbackProvider>
    </BrowserRouter>
  );
}
