import { Download, Gauge, KeyRound, Laptop, LogOut, UserRound } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { Brand } from "./Brand";
import { useLocalization } from "../localization/LocalizationContext";
import type { MessageKey } from "../../../localization";

const links = [
  ["portal.overview", "/portal", Gauge],
  ["portal.devices", "/portal/devices", Laptop],
  ["portal.downloads", "/portal/downloads", Download],
  ["portal.subscription", "/portal/subscription", KeyRound],
  ["portal.account", "/portal/account", UserRound],
] as const satisfies readonly (readonly [MessageKey, string, typeof Gauge])[];

export function PortalLayout() {
  const { user, logout } = useAuth();
  const { locale, setLocale, t } = useLocalization();
  return (
    <div className="portal-shell">
      <aside className="portal-sidebar">
        <Brand />
        <div className="portal-account"><span>{t("portal.customerPortal")}</span><strong>{user?.email}</strong></div>
        <label className="portal-language"><span>{t("language.label")}</span><select aria-label={t("language.label")} value={locale} onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}><option value="en">EN</option><option value="ar">AR</option></select></label>
        <nav>{links.map(([label, path, Icon]) => <NavLink key={path} to={path} end={path === "/portal"}><Icon size={18} />{t(label)}</NavLink>)}</nav>
        <button className="sidebar-signout" onClick={() => void logout()}><LogOut size={18} />{t("portal.signOut")}</button>
      </aside>
      <main className="portal-main"><Outlet /></main>
    </div>
  );
}
