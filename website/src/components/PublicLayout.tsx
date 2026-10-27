import { Languages, Menu, X } from "lucide-react";
import { useState } from "react";
import { Link, NavLink, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { useLocalization } from "../localization/LocalizationContext";
import { Brand } from "./Brand";
import type { MessageKey } from "../../../localization";

const links: [MessageKey, string][] = [
  ["site.nav.features", "/features"],
  ["site.nav.pricing", "/pricing"],
  ["site.nav.download", "/download"],
  ["site.nav.faq", "/faq"],
  ["site.nav.support", "/support"],
];

export function PublicLayout() {
  const [open, setOpen] = useState(false);
  const { user } = useAuth();
  const { locale, setLocale, t } = useLocalization();
  return <div className="site-shell">
    <header className="site-header">
      <div className="container header-inner">
        <Brand />
        <button className="menu-button" onClick={() => setOpen((value) => !value)} aria-label={t("site.nav.toggle")}>
          {open ? <X /> : <Menu />}
        </button>
        <nav className={open ? "nav-open" : ""} aria-label={t("site.nav.main")}>
          {links.map(([key, path]) => <NavLink key={path} to={path} onClick={() => setOpen(false)}>{t(key)}</NavLink>)}
          <label className="site-language"><Languages size={17} /><span className="sr-only">{t("language.label")}</span><select aria-label={t("language.label")} value={locale} onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}><option value="en">EN</option><option value="ar">AR</option></select></label>
          <Link className="button button-ghost" to={user ? "/portal" : "/login"}>{user ? t("site.nav.portal") : t("site.nav.signIn")}</Link>
          <Link className="button button-primary" to={user ? "/portal" : "/register"}>{user ? t("site.nav.portal") : t("site.nav.createAccount")}</Link>
        </nav>
      </div>
    </header>
    <main><Outlet /></main>
    <footer className="site-footer">
      <div className="container footer-grid">
        <div><Brand /><p>{t("site.footer.tagline")}</p></div>
        <div><strong>{t("site.footer.product")}</strong><Link to="/features">{t("site.nav.features")}</Link><Link to="/download">{t("site.nav.download")}</Link><Link to="/pricing">{t("site.nav.pricing")}</Link></div>
        <div><strong>{t("site.footer.help")}</strong><Link to="/faq">{t("site.nav.faq")}</Link><Link to="/support">{t("site.nav.support")}</Link><Link to="/login">{t("site.footer.portal")}</Link></div>
        <div><strong>{t("site.footer.earlyAccess")}</strong><p>{t("site.footer.beta")}</p></div>
      </div>
      <div className="container footer-bottom">{t("site.footer.legal")}</div>
    </footer>
  </div>;
}
