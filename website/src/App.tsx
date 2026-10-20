import { Navigate, Route, Routes } from "react-router-dom";
import { PublicLayout } from "./components/PublicLayout";
import { PortalLayout } from "./components/PortalLayout";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { AuthPage, ForgotPasswordPage, ResetPasswordPage } from "./pages/AuthPages";
import { DownloadPage, FaqPage, FeaturesPage, HomePage, PricingPage, SupportPage } from "./pages/PublicPages";
import { AccountPage, DevicesPage, DownloadsPage, OverviewPage, SubscriptionPage } from "./pages/PortalPages";

export function App() {
  return (
    <Routes>
      <Route element={<PublicLayout />}>
        <Route index element={<HomePage />} />
        <Route path="features" element={<FeaturesPage />} />
        <Route path="pricing" element={<PricingPage />} />
        <Route path="download" element={<DownloadPage />} />
        <Route path="faq" element={<FaqPage />} />
        <Route path="support" element={<SupportPage />} />
        <Route path="login" element={<AuthPage mode="login" />} />
        <Route path="register" element={<AuthPage mode="register" />} />
        <Route path="forgot-password" element={<ForgotPasswordPage />} />
        <Route path="account/reset-password" element={<ResetPasswordPage />} />
        <Route path="auth/callback" element={<Navigate to="/portal" replace />} />
      </Route>
      <Route path="portal" element={<ProtectedRoute><PortalLayout /></ProtectedRoute>}>
        <Route index element={<OverviewPage />} />
        <Route path="devices" element={<DevicesPage />} />
        <Route path="downloads" element={<DownloadsPage />} />
        <Route path="subscription" element={<SubscriptionPage />} />
        <Route path="account" element={<AccountPage />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
