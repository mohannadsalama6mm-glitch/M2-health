import { Menu, X } from "lucide-react";
import { useState } from "react";
import { Link, NavLink, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { Brand } from "./Brand";

const links = [
  ["Features", "/features"],
  ["Pricing", "/pricing"],
  ["Download", "/download"],
  ["FAQ", "/faq"],
  ["Support", "/support"],
] as const;

export function PublicLayout() {
  const [open, setOpen] = useState(false);
  const { user } = useAuth();
  return (
    <div className="site-shell">
      <header className="site-header">
        <div className="container header-inner">
          <Brand />
          <button className="menu-button" onClick={() => setOpen((value) => !value)} aria-label="Toggle navigation">
            {open ? <X /> : <Menu />}
          </button>
          <nav className={open ? "nav-open" : ""} aria-label="Main navigation">
            {links.map(([label, path]) => <NavLink key={path} to={path} onClick={() => setOpen(false)}>{label}</NavLink>)}
            <Link className="button button-ghost" to={user ? "/portal" : "/login"}>{user ? "Portal" : "Sign in"}</Link>
            <Link className="button button-primary" to={user ? "/portal" : "/register"}>Start free trial</Link>
          </nav>
        </div>
      </header>
      <main><Outlet /></main>
      <footer className="site-footer">
        <div className="container footer-grid">
          <div><Brand /><p>Local-first pharmacy operations for Windows.</p></div>
          <div><strong>Product</strong><Link to="/features">Features</Link><Link to="/download">Download</Link><Link to="/pricing">Pricing</Link></div>
          <div><strong>Help</strong><Link to="/faq">FAQ</Link><Link to="/support">Support</Link><Link to="/login">Customer portal</Link></div>
          <div><strong>Early Access</strong><p>Beta software. Keep verified backups and review release notes before updating.</p></div>
        </div>
        <div className="container footer-bottom">© 2026 M² Health. Payments are not enabled.</div>
      </footer>
    </div>
  );
}
