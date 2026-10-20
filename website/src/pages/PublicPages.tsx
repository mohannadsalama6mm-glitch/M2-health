import {
  ArchiveRestore,
  BadgeCheck,
  BarChart3,
  Boxes,
  Building2,
  Check,
  CircleDollarSign,
  Download,
  FileLock2,
  HardDrive,
  Laptop,
  PackageCheck,
  ReceiptText,
  ShieldCheck,
  ShoppingCart,
  SignalZero,
  Users,
} from "lucide-react";
import { Link } from "react-router-dom";
import { currentRelease } from "../content/release";
import { siteConfig } from "../lib/config";

const features = [
  [ShoppingCart, "Fast sales and returns", "A focused POS workflow with stock-aware sales, returns, and clear receipt history."],
  [Boxes, "Inventory you can trust", "Batch balances, expiry monitoring, stock counts, adjustments, and movement history."],
  [PackageCheck, "Purchasing and receiving", "Purchase orders, controlled receiving, supplier returns, and cost visibility."],
  [Users, "Customers and suppliers", "Keep the relationships behind sales and purchasing connected to daily work."],
  [BarChart3, "Reports and finance", "Operational reporting, expenses, cash movements, and business-ready summaries."],
  [ShieldCheck, "Roles and branches", "Role-aware access at the desktop boundary with branch-scoped operations."],
  [ArchiveRestore, "Backup and recovery", "Validated backups, safety copies, restore checks, and recoverable rollback."],
  [SignalZero, "Offline-first by design", "Core pharmacy work stays local. Planned sync remains gated until explicitly enabled."],
] as const;

const faqs = [
  ["When does the seven-day trial start?", "Only when you explicitly start it from your account or activate an eligible device. Creating an account alone does not consume trial time."],
  ["Does M² Health work offline?", "Yes. Core pharmacy operations are local-first. An active signed license can continue within its stated offline validation window."],
  ["What happens when a trial expires?", "Operational writes become read-only. Existing pharmacy data is retained and backup, recovery, diagnostics, and activation remain available."],
  ["Where is pharmacy data stored?", "The Windows app stores operational data locally on the device. Your portal manages account, license, and registered-device state—not pharmacy transaction rows."],
  ["How do backups work?", "The desktop validates backup integrity and schema compatibility. Restore requires confirmation, creates a safety backup, and rolls back if validation fails."],
  ["Can I use more than one device?", "Early Access starts with plan-aware device limits. The portal shows registered devices and lets you revoke one you no longer trust."],
  ["Can I pay online now?", "No. Payments are deliberately not enabled in this phase. Upgrade actions create a manual activation request for the Early Access team."],
  ["Is the Windows installer signed?", "Not yet. The current Early Access installer is unsigned; verify the published SHA-256 checksum before installation."],
] as const;

function CtaBand() {
  return <section className="cta-band"><div><span className="eyebrow">Early Access is open</span><h2>Put the pharmacy workflow on solid ground.</h2><p>Start with seven days free. No card, no simulated payment, and no lock-in of your local data.</p></div><div className="button-row"><Link className="button button-light" to="/register">Create account</Link><Link className="button button-outline-light" to="/download">Download for Windows</Link></div></section>;
}

export function HomePage() {
  return <>
    <section className="hero"><div className="container hero-grid"><div className="hero-copy"><span className="eyebrow">Windows pharmacy management · Early Access</span><h1>Your pharmacy stays moving—even when the internet doesn’t.</h1><p className="lead">M² Health brings sales, stock, purchasing, finance, and recovery into one local-first desktop workspace built for the realities of daily pharmacy operations.</p><div className="button-row"><Link className="button button-primary button-large" to="/register">Start 7-day free trial</Link><Link className="button button-secondary button-large" to="/download"><Download size={18} />Download beta</Link></div><div className="hero-trust"><span><Check />No payment required</span><span><Check />Local pharmacy data</span><span><Check />Backup-first recovery</span></div></div><div className="hero-visual"><div className="visual-glow" /><img src="/images/product-pos.png" alt="M² Health point-of-sale workspace" /><div className="floating-card"><span className="pulse" /><div><strong>Local-first</strong><small>Ready for the counter</small></div></div></div></div></section>
    <section className="signal-strip"><div className="container"><span>One desktop workspace</span><strong>Sales</strong><strong>Inventory</strong><strong>Purchasing</strong><strong>Reports</strong><strong>Finance</strong><strong>Recovery</strong></div></section>
    <section className="section container"><div className="section-heading"><span className="eyebrow">Built around the work</span><h2>From the sale to the stock ledger, without loose ends.</h2><p>Every core flow is designed to leave a durable, reviewable trail.</p></div><div className="feature-grid">{features.slice(0, 6).map(([Icon, title, copy]) => <article className="feature-card" key={title}><span className="icon-tile"><Icon /></span><h3>{title}</h3><p>{copy}</p></article>)}</div><div className="center-action"><Link to="/features" className="text-link">Explore every capability →</Link></div></section>
    <section className="section soft-section"><div className="container split-showcase"><div><span className="eyebrow">Real desktop software</span><h2>A clear workspace for the whole operation.</h2><p className="lead-small">Purpose-built screens keep the important actions close, while roles and branches define who can do what.</p><ul className="check-list"><li><BadgeCheck />Canonical sales, purchasing, partner, reporting, and finance routes</li><li><FileLock2 />Authorization enforced at the native IPC boundary</li><li><HardDrive />Local SQLite with explicit backup and restore controls</li></ul><Link className="button button-secondary" to="/download">See the current release</Link></div><div className="image-frame"><img src="/images/product-inventory.png" alt="M² Health inventory workspace" /></div></div></section>
    <section className="section container"><div className="three-proof"><article><SignalZero /><h3>Work locally</h3><p>Daily operations do not pretend a browser is a database.</p></article><article><ArchiveRestore /><h3>Recover deliberately</h3><p>Integrity checks and safety backups protect restore operations.</p></article><article><Laptop /><h3>Control devices</h3><p>Account-bound device state can be reviewed and revoked.</p></article></div></section>
    <CtaBand />
  </>;
}

export function FeaturesPage() {
  return <><PageHero eyebrow="Product capabilities" title="The operational core a pharmacy needs." copy="M² Health brings transactions, stock, people, reporting, and recovery together without turning local work into a cloud dependency." /><section className="section container"><div className="feature-grid feature-grid-wide">{features.map(([Icon, title, copy]) => <article className="feature-card" key={title}><span className="icon-tile"><Icon /></span><h3>{title}</h3><p>{copy}</p></article>)}</div></section><section className="section dark-section"><div className="container roadmap-note"><div><span className="eyebrow">Planned, not overstated</span><h2>Multi-device sync remains controlled.</h2><p>Its security and replication foundation is verified, but rollout and transport gates stay disabled until a separate approved launch step. Early Access marketing does not claim otherwise.</p></div><Building2 size={92} /></div></section><CtaBand /></>;
}

export function PricingPage() {
  const plans = [
    ["Early Access", "7 days free", "Evaluate the complete local desktop workflow.", ["One account", "Plan-aware device registration", "Backup and diagnostics", "Manual upgrade request"], true],
    ["Single Pharmacy", "Pricing announced later", "For an owner and team operating one pharmacy.", ["Core desktop operations", "Roles and branch controls", "License/device portal", "Priority onboarding"], false],
    ["Multi-branch", "Contact us", "Prepared for organizations that need controlled expansion.", ["Multiple branches", "Per-plan device limits", "Advanced administration", "Planned gated sync"], false],
  ] as const;
  return <><PageHero eyebrow="Simple Early Access" title="Try the real workflow before choosing a plan." copy="No payment gateway is connected. Upgrade actions create a manual access request—never a fake checkout." /><section className="section container"><div className="pricing-grid">{plans.map(([name, price, copy, bullets, featured]) => <article className={`price-card ${featured ? "featured" : ""}`} key={name}>{featured && <span className="plan-ribbon">Start here</span>}<h2>{name}</h2><strong className="price">{price}</strong><p>{copy}</p><ul>{bullets.map((item) => <li key={item}><Check />{item}</li>)}</ul><Link className={`button ${featured ? "button-primary" : "button-secondary"}`} to={featured ? "/register" : "/support"}>{featured ? "Start free trial" : "Request activation"}</Link></article>)}</div><p className="pricing-disclaimer"><CircleDollarSign /> Prices and recurring billing are not active. No card details are collected.</p></section></>;
}

export function DownloadPage() {
  return <><PageHero eyebrow="Windows Early Access" title="Download M² Health for Windows." copy="Install the current beta, verify its checksum, and keep a recent backup before updating." /><section className="section container"><div className="download-panel"><div className="release-icon"><Download /></div><div className="release-main"><div className="release-title"><h2>M² Health {currentRelease.version}</h2><span>{currentRelease.channel}</span></div><dl><div><dt>Build</dt><dd>{currentRelease.buildId}</dd></div><div><dt>Platform</dt><dd>{currentRelease.platform}</dd></div><div><dt>Published</dt><dd>{currentRelease.publishedAt}</dd></div><div><dt>Size</dt><dd>{(currentRelease.sizeBytes / 1_000_000).toFixed(1)} MB</dd></div></dl><a className="button button-primary button-large" href={currentRelease.downloadUrl} download><Download size={19} />Download installer</a><a className="text-link" href={currentRelease.releaseNotesUrl}>Read release notes</a></div></div><div className="warning-card"><ShieldCheck /><div><h3>Unsigned Early Access build</h3><p>Windows may show a publisher warning because production code signing has not been purchased. Download only from this page and compare the SHA-256 value.</p><code>{currentRelease.checksum}</code></div></div><div className="requirements-grid"><article><h3>System requirements</h3><ul><li>Windows 10 or Windows 11, x64</li><li>WebView2 runtime (installer checks availability)</li><li>At least 500 MB available storage</li><li>Internet for account activation; core operations remain local</li></ul></article><article><h3>Before installing</h3><ul><li>Keep a verified backup of any existing pharmacy database</li><li>Do not install over an unknown or unofficial build</li><li>Record your device ID for activation support</li><li>Review the current beta release notes</li></ul></article></div></section></>;
}

export function FaqPage() {
  return <><PageHero eyebrow="Clear answers" title="Early Access, without the fine-print fog." copy="What the trial does, where your data lives, and what is deliberately not automated yet." /><section className="section container faq-list">{faqs.map(([question, answer]) => <details key={question}><summary>{question}</summary><p>{answer}</p></details>)}</section></>;
}

export function SupportPage() {
  return <><PageHero eyebrow="Support and activation" title="Tell us what you need." copy="Use the customer portal for account-specific activation requests, or contact the Early Access team for installation and recovery guidance." /><section className="section container support-grid"><article className="support-card"><ReceiptText /><h2>Account or license request</h2><p>Signed-in customers can create a traceable manual activation request and see its status.</p><Link className="button button-primary" to="/portal/subscription">Open customer portal</Link></article><article className="support-card"><ArchiveRestore /><h2>Installation or recovery</h2><p>Include the app version, build ID, and sanitized diagnostic export. Never send your pharmacy database or activation signing material by email.</p><a className="button button-secondary" href={`mailto:${siteConfig.supportEmail}?subject=M2%20Health%20Early%20Access%20support`}>Email support</a></article></section></>;
}

function PageHero({ eyebrow, title, copy }: { eyebrow: string; title: string; copy: string }) {
  return <section className="page-hero"><div className="container"><span className="eyebrow">{eyebrow}</span><h1>{title}</h1><p>{copy}</p></div></section>;
}
