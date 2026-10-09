import { useEffect, useState } from "react";
import { BrowserRouter } from "react-router-dom";
import { Toast } from "../design-system";
import { DemoProvider } from "./DemoContext";
import { BranchProvider } from "./BranchContext";
import { AppShell } from "./AppShell";
import { AuthProvider } from "./AuthContext";
import { AuthGate } from "../features/auth/AuthGate";

export function App() {
  const [toast, setToast] = useState("");
  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(""), 5000);
    return () => window.clearTimeout(timer);
  }, [toast]);
  return (
    <BrowserRouter>
      <DemoProvider notify={setToast}>
        <AuthProvider>
          <AuthGate>
            <BranchProvider>
              <AppShell />
            </BranchProvider>
          </AuthGate>
        </AuthProvider>
        {toast && <Toast message={toast} onClose={() => setToast("")} />}
      </DemoProvider>
    </BrowserRouter>
  );
}
