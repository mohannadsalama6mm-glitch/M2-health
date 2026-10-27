import { useEffect, useRef, useState } from "react";
import { NavLink, useLocation, useNavigate } from "react-router-dom";
import {
  PanelLeftClose,
  PanelLeftOpen,
  Bell,
  ChevronDown,
  Store,
  Languages,
  CloudOff,
} from "lucide-react";
import {
  Logo,
  AppBackground,
  IconButton,
  SearchInput,
  Avatar,
  Dropdown,
  Badge,
} from "../design-system";
import { navigation } from "./navigation";
import { useDemo } from "./DemoContext";
import { AppRoutes } from "./AppRoutes";
import { BranchSelector } from "./BranchSelector";
import { useBranch } from "./BranchContext";
import { isDesktopRuntime } from "../lib/tauri/client";
import { useAuth } from "./AuthContext";
import type { Permission } from "../lib/permissions";
import type { Product } from "../mock/types";
import { useLicense } from "./LicenseContext";
import { release } from "../lib/release";
import { useLocalization } from "../localization/LocalizationContext";
import { DesktopOnboarding, shouldOpenDesktopOnboarding } from "../features/onboarding/DesktopOnboarding";
import { GuidedTutorial, shouldStartGuidedTutorial } from "../features/onboarding/GuidedTutorial";

export function AppShell() {
  if (isDesktopRuntime())
    return <AppShellContent desktop branch="" products={[]} />;
  return <BrowserAppShell />;
}

function BrowserAppShell() {
  const { branch, products } = useDemo();
  return (
    <AppShellContent desktop={false} branch={branch} products={products} />
  );
}

function AppShellContent({
  desktop,
  branch,
  products,
}: {
  desktop: boolean;
  branch: string;
  products: Product[];
}) {
  const [collapsed, setCollapsed] = useState(false),
    [query, setQuery] = useState(""),
    [setupOpen, setSetupOpen] = useState(() => desktop && shouldOpenDesktopOnboarding()),
    [tutorialOpen, setTutorialOpen] = useState(() => desktop && !shouldOpenDesktopOnboarding() && shouldStartGuidedTutorial());
  const { branchName } = useBranch();
  const auth = useAuth();
  const license = useLicense();
  const { locale, setLocale, state: stateLabel, t } = useLocalization();
  const searchRef = useRef<HTMLDivElement>(null),
    mainRef = useRef<HTMLElement>(null);
  const navigate = useNavigate(),
    location = useLocation();
  const nativeWorkspace = desktop;
  const globalSearchAvailable = !nativeWorkspace;
  const nativeWorkspaceName = location.pathname.startsWith("/dashboard")
    ? t("nav.dashboard")
    : location.pathname.startsWith("/finance")
      ? t("nav.finance")
      : location.pathname.startsWith("/sales")
        ? t("home.sales")
        : location.pathname.startsWith("/reports")
          ? t("nav.reports")
          : location.pathname.startsWith("/purchases")
            ? t("home.purchasing")
            : location.pathname.startsWith("/suppliers")
              ? t("nav.suppliers")
              : location.pathname.startsWith("/customers")
                ? t("nav.customers")
                : desktop
                  ? "M² Health"
                  : t("nav.inventory");
  useEffect(() => {
    mainRef.current?.scrollTo(0, 0);
  }, [location.pathname]);
  useEffect(() => {
    if (!globalSearchAvailable) return;
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        searchRef.current?.querySelector("input")?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [globalSearchAvailable]);
  const matches = (desktop ? [] : products)
    .filter((p) =>
      `${p.name} ${(p.barcodes ?? [p.barcode]).join(" ")}`
        .toLowerCase()
        .includes(query.toLowerCase()),
    )
    .slice(0, 5);
  const go = (to: string) => {
    setQuery("");
    navigate(to);
  };
  return (
    <div className={`app-shell ${collapsed ? "collapsed" : ""}`}>
      <AppBackground />
      <a className="skip-link" href="#main-content">
        {t("shell.skip")}
      </a>
      <aside className="sidebar" aria-label={t("shell.sidebar")}>
        <div className="sidebar-brand">
          <Logo variant={collapsed ? "icon" : "compact"} />
        </div>
        <nav aria-label={t("shell.pharmacyNav")} data-tutorial="navigation">
          {navigation
            .map((group) => ({
              ...group,
              items: group.items.filter(
                (item) =>
                  !("permission" in item) ||
                  auth.hasPermission(item.permission as Permission),
              ),
            }))
            .filter((group) => group.items.length)
            .map((group) => (
              <div
                className={`nav-group nav-group--${group.priority}`}
                data-navigation-priority={group.priority}
                key={group.groupKey ?? "home"}
              >
                {group.groupKey && <div className="nav-label">{t(group.groupKey)}</div>}
                {group.items.map(({ labelKey, to, icon: Icon }) => (
                  <NavLink
                    key={to}
                    to={to}
                    end={to === "/inventory"}
                    className={({ isActive }) =>
                      `nav-item ${isActive || (to === "/employees" && location.pathname === "/roles") || (to === "/inventory" && ["/inventory/movements", "/inventory/counts"].includes(location.pathname)) ? "active" : ""}`
                    }
                    title={t(labelKey)}
                    data-tutorial={`nav-${to.split("/")[1] || "dashboard"}`}
                  >
                    <Icon size={18} />
                    <span>{t(labelKey)}</span>
                  </NavLink>
                ))}
              </div>
            ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="offline-caption">
            <CloudOff size={15} />
            <span>
              {desktop ? t("shell.localWorkspace") : t("shell.browserPreview")}
            </span>
          </div>
          <div className="version">
            <span>M² Health</span>
            <Badge>v{release.version}</Badge>
          </div>
          {desktop && license.status && (
            <div className="offline-caption" title={license.status.reason}>
              <span>
                {license.status.state === "TRIAL"
                      ? t("shell.trialDays", { count: license.status.daysRemaining ?? 0 })
                  : license.status.state === "OFFLINE_GRACE"
                    ? t("state.offlineGrace")
                    : license.status.state === "ACTIVE"
                      ? t("shell.licensed")
                      : `${stateLabel(license.status.state)} · ${t("state.readOnly")}`}
              </span>
            </div>
          )}
        </div>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <IconButton
            label={collapsed ? t("shell.expand") : t("shell.collapse")}
            onClick={() => setCollapsed(!collapsed)}
          >
            {collapsed ? (
              <PanelLeftOpen size={19} />
            ) : (
              <PanelLeftClose size={19} />
            )}
          </IconButton>
          <div className="global-search" ref={searchRef}>
            <SearchInput
              placeholder={
                nativeWorkspace
                  ? t("shell.workspaceSearch")
                  : t("shell.previewSearch")
              }
              label={t("shell.globalSearch")}
              disabled={nativeWorkspace}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape") setQuery("");
                if (e.key === "Enter" && matches[0])
                  go(`/catalog/${matches[0].id}`);
              }}
            />
            {globalSearchAvailable && <kbd>Ctrl K</kbd>}
            {query && !desktop && (
              <div className="search-results global-results">
                {matches.length ? (
                  matches.map((p) => (
                    <NavLink
                      key={p.id}
                      to={`/catalog/${p.id}`}
                      onClick={() => setQuery("")}
                    >
                      <strong>{p.name}</strong>
                      <span>{p.barcode}</span>
                    </NavLink>
                  ))
                ) : (
                  <p>{t("shell.noPreviewProducts")}</p>
                )}
              </div>
            )}
          </div>
          <div className="topbar-end">
            <Store size={16} />
            <div data-tutorial="branch"><BranchSelector /></div>
            <span className="topbar-divider" />
            <label className="language-selector" title={t("shell.languageOptions")}>
              <Languages size={18} />
              <span className="sr-only">{t("language.label")}</span>
              <select
                aria-label={t("language.label")}
                value={locale}
                onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}
              >
                <option value="en">EN</option>
                <option value="ar">AR</option>
              </select>
            </label>
            <IconButton
              label={t("nav.notifications")}
              onClick={() => navigate("/notifications")}
            >
              <Bell size={19} />
            </IconButton>
            <span className="topbar-divider" />
            <div data-tutorial="profile-menu"><Dropdown
              label={
                <>
                  <Avatar name={auth.user?.displayName ?? t("shell.localUser")} />
                  <span className="user-info">
                    <strong>{auth.user?.displayName ?? t("shell.localUser")}</strong>
                    <small>
                      {auth.user
                        ? auth.user.roles
                            .map((role) => role.name)
                            .join(" · ") || t("shell.noRole")
                        : t("shell.noSession")}
                    </small>
                  </span>
                  <ChevronDown size={14} />
                </>
              }
              items={[
                {
                  label: t("shell.employeeProfile"),
                  onSelect: () =>
                    navigate(
                      auth.user
                        ? `/employees/${auth.user.id}`
                        : "/employees/e1",
                    ),
                },
                {
                  label: t("shell.roles"),
                  onSelect: () => navigate("/roles"),
                },
                {
                  label: t("shell.workspaceSettings"),
                  onSelect: () => navigate("/settings"),
                },
                ...(desktop
                  ? [
                      { label: t("onboarding.open"), onSelect: () => setSetupOpen(true) },
                      { label: t("tutorial.restart"), onSelect: () => setTutorialOpen(true) },
                    ]
                  : []),
                ...(auth.user
                  ? [
                      {
                        label: t("shell.signOut"),
                        onSelect: () => {
                          void auth.signOut();
                        },
                      },
                    ]
                  : []),
              ]}
            /></div>
          </div>
        </header>
        <main id="main-content" tabIndex={-1} ref={mainRef}>
          <AppRoutes />
        </main>
        <div className="desktop-statusbar">
          <span>
            {desktop ? branchName : branch}{" "}
            <span className="caption-separator">/</span>{" "}
            {desktop
              ? t("shell.persisted", { workspace: nativeWorkspaceName })
              : t("shell.previewOnly")}
          </span>
          <NavLink to="/sync">
            <CloudOff size={12} /> {t("shell.cloudDisconnected")}
          </NavLink>
        </div>
      </div>
      {desktop && <DesktopOnboarding open={setupOpen} onOpenChange={(open) => {
        setSetupOpen(open);
        if (!open && shouldStartGuidedTutorial()) window.setTimeout(() => setTutorialOpen(true), 200);
      }} />}
      {desktop && tutorialOpen && !setupOpen && <GuidedTutorial onOpenChange={setTutorialOpen} />}
    </div>
  );
}
