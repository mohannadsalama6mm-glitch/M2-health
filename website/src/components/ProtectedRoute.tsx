import { Navigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { useLocalization } from "../localization/LocalizationContext";

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();
  const { t } = useLocalization();
  const location = useLocation();
  if (loading) return <main className="centered-page"><p role="status">{t("portal.checking")}</p></main>;
  if (!user) return <Navigate to="/login" replace state={{ from: location.pathname }} />;
  return children;
}
