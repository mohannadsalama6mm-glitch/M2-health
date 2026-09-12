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
export function AppShell() {
  const [collapsed, setCollapsed] = useState(false),
    [query, setQuery] = useState("");
  const { branch, products } = useDemo();
  const searchRef = useRef<HTMLDivElement>(null),
    mainRef = useRef<HTMLElement>(null);
  const navigate = useNavigate(),
    location = useLocation();
  useEffect(() => {
    mainRef.current?.scrollTo(0, 0);
  }, [location.pathname]);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        searchRef.current?.querySelector("input")?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
  const matches = products
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
        Skip to main content
      </a>
      <aside className="sidebar" aria-label="Application sidebar">
        <div className="sidebar-brand">
          <Logo variant={collapsed ? "icon" : "compact"} />
        </div>
        <nav aria-label="Pharmacy navigation">
          {navigation.map((group) => (
            <div className="nav-group" key={group.group}>
              {group.group && <div className="nav-label">{group.group}</div>}
              {group.items.map(({ label, to, icon: Icon }) => (
                <NavLink
                  key={to}
                  to={to}
                  end={to === "/inventory"}
                  className={({ isActive }) =>
                    `nav-item ${isActive || (to === "/employees" && location.pathname === "/roles") || (to === "/inventory" && ["/inventory/movements", "/inventory/counts"].includes(location.pathname)) ? "active" : ""}`
                  }
                  title={label}
                >
                  <Icon size={18} />
                  <span>{label}</span>
                </NavLink>
              ))}
            </div>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="offline-caption">
            <CloudOff size={15} />
            <span>Local workspace preview</span>
          </div>
          <div className="version">
            <span>M² Health</span>
            <Badge>v0.2.0</Badge>
          </div>
        </div>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <IconButton
            label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
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
              placeholder="Search products or barcodes…"
              label="Global search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape") setQuery("");
                if (e.key === "Enter" && matches[0])
                  go(`/catalog/${matches[0].id}`);
              }}
            />
            <kbd>Ctrl K</kbd>
            {query && (
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
                  <p>No matching demo products. Try Panadol.</p>
                )}
              </div>
            )}
          </div>
          <div className="topbar-end">
            <Store size={16} />
            <BranchSelector />
            <span className="topbar-divider" />
            <IconButton
              label="Language options"
              onClick={() => navigate("/settings?section=Language")}
            >
              <Languages size={18} />
            </IconButton>
            <IconButton
              label="Notifications"
              onClick={() => navigate("/notifications")}
            >
              <Bell size={19} />
            </IconButton>
            <span className="topbar-divider" />
            <Dropdown
              label={
                <>
                  <Avatar name="Mohannad" />
                  <span className="user-info">
                    <strong>Mohannad</strong>
                    <small>Owner · demo role</small>
                  </span>
                  <ChevronDown size={14} />
                </>
              }
              items={[
                {
                  label: "Employee profile",
                  onSelect: () => navigate("/employees/e1"),
                },
                {
                  label: "Roles & permissions",
                  onSelect: () => navigate("/roles"),
                },
                {
                  label: "Workspace settings",
                  onSelect: () => navigate("/settings"),
                },
              ]}
            />
          </div>
        </header>
        <main id="main-content" tabIndex={-1} ref={mainRef}>
          <AppRoutes />
        </main>
        <div className="desktop-statusbar">
          <span>
            {branch} <span className="caption-separator">/</span> UI demo · no
            persistent data
          </span>
          <NavLink to="/sync">
            <CloudOff size={12} /> Local / cloud not connected
          </NavLink>
        </div>
      </div>
    </div>
  );
}
