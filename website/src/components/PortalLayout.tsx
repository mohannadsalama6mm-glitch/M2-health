import { Download, Gauge, KeyRound, Laptop, LogOut, UserRound } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { Brand } from "./Brand";

const links = [
  ["Overview", "/portal", Gauge],
  ["Devices", "/portal/devices", Laptop],
  ["Downloads", "/portal/downloads", Download],
  ["Subscription", "/portal/subscription", KeyRound],
  ["Account", "/portal/account", UserRound],
] as const;

export function PortalLayout() {
  const { user, logout } = useAuth();
  return (
    <div className="portal-shell">
      <aside className="portal-sidebar">
        <Brand />
        <div className="portal-account"><span>Customer portal</span><strong>{user?.email}</strong></div>
        <nav>{links.map(([label, path, Icon]) => <NavLink key={path} to={path} end={path === "/portal"}><Icon size={18} />{label}</NavLink>)}</nav>
        <button className="sidebar-signout" onClick={() => void logout()}><LogOut size={18} />Sign out</button>
      </aside>
      <main className="portal-main"><Outlet /></main>
    </div>
  );
}
