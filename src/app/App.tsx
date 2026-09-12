import { useEffect, useState } from "react";
import { BrowserRouter } from "react-router-dom";
import { Toast } from "../design-system";
import { DemoProvider } from "./DemoContext";
import { AppShell } from "./AppShell";

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
        <AppShell />
        {toast && <Toast message={toast} onClose={() => setToast("")} />}
      </DemoProvider>
    </BrowserRouter>
  );
}
