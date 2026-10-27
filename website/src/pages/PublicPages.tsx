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
import { useLocalization } from "../localization/LocalizationContext";
import type { MessageKey } from "../../../localization";

const features = [
  [ShoppingCart, "home.feature.salesTitle", "home.feature.salesCopy"],
  [Boxes, "home.feature.inventoryTitle", "home.feature.inventoryCopy"],
  [PackageCheck, "home.feature.purchaseTitle", "home.feature.purchaseCopy"],
  [Users, "home.feature.partnersTitle", "home.feature.partnersCopy"],
  [BarChart3, "home.feature.reportsTitle", "home.feature.reportsCopy"],
  [ShieldCheck, "home.feature.rolesTitle", "home.feature.rolesCopy"],
  [ArchiveRestore, "home.feature.backupTitle", "home.feature.backupCopy"],
  [SignalZero, "home.feature.offlineTitle", "home.feature.offlineCopy"],
] as const;

const faqs = [
  ["faq.trialQuestion", "faq.trialAnswer"],
  ["faq.offlineQuestion", "faq.offlineAnswer"],
  ["faq.expiryQuestion", "faq.expiryAnswer"],
  ["faq.storageQuestion", "faq.storageAnswer"],
  ["faq.backupQuestion", "faq.backupAnswer"],
  ["faq.devicesQuestion", "faq.devicesAnswer"],
  ["faq.paymentQuestion", "faq.paymentAnswer"],
  ["faq.signingQuestion", "faq.signingAnswer"],
] as const;

const plans = [
  ["pricing.earlyAccessName", "pricing.earlyAccessPrice", "pricing.earlyAccessCopy", ["pricing.earlyAccessBulletAccount", "pricing.earlyAccessBulletDevice", "pricing.earlyAccessBulletBackup", "pricing.earlyAccessBulletUpgrade"], true],
  ["pricing.singleName", "pricing.singlePrice", "pricing.singleCopy", ["pricing.singleBulletOperations", "pricing.singleBulletRoles", "pricing.singleBulletPortal", "pricing.singleBulletOnboarding"], false],
  ["pricing.multiName", "pricing.multiPrice", "pricing.multiCopy", ["pricing.multiBulletBranches", "pricing.multiBulletDevices", "pricing.multiBulletAdmin", "pricing.multiBulletSync"], false],
] as const;

function CtaBand() {
  const { t } = useLocalization();
  return <section className="cta-band"><div><span className="eyebrow">{t("home.open")}</span><h2>{t("home.ctaTitle")}</h2><p>{t("home.ctaCopy")}</p></div><div className="button-row"><Link className="button button-light" to="/register">{t("home.createAccount")}</Link><Link className="button button-outline-light" to="/download">{t("home.downloadWindows")}</Link></div></section>;
}

export function HomePage() {
  const { t } = useLocalization();
  return <>
    <section className="hero"><div className="container hero-grid"><div className="hero-copy"><span className="eyebrow">{t("home.eyebrow")}</span><h1>{t("home.title")}</h1><p className="lead">{t("home.lead")}</p><div className="button-row"><Link className="button button-primary button-large" to="/register">{t("home.createAccount")}</Link><Link className="button button-secondary button-large" to="/download"><Download size={18} />{t("home.downloadBeta")}</Link></div><div className="hero-trust"><span><Check />{t("home.noPayment")}</span><span><Check />{t("home.localData")}</span><span><Check />{t("home.backupFirst")}</span></div></div><div className="hero-visual"><div className="visual-glow" /><img src="/images/product-pos.png" alt={t("home.feature.salesTitle")} /><div className="floating-card"><span className="pulse" /><div><strong>{t("home.localFirst")}</strong><small>{t("home.counterReady")}</small></div></div></div></div></section>
    <section className="signal-strip"><div className="container"><span>{t("home.oneWorkspace")}</span><strong>{t("home.sales")}</strong><strong>{t("home.inventory")}</strong><strong>{t("home.purchasing")}</strong><strong>{t("home.reports")}</strong><strong>{t("home.finance")}</strong><strong>{t("home.recovery")}</strong></div></section>
    <section className="section container"><div className="section-heading"><span className="eyebrow">{t("home.builtAround")}</span><h2>{t("home.workflowTitle")}</h2><p>{t("home.workflowCopy")}</p></div><div className="feature-grid">{features.slice(0, 6).map(([Icon, title, copy]) => <article className="feature-card" key={title}><span className="icon-tile"><Icon /></span><h3>{t(title as MessageKey)}</h3><p>{t(copy as MessageKey)}</p></article>)}</div><div className="center-action"><Link to="/features" className="text-link">{t("home.explore")}</Link></div></section>
    <section className="section soft-section"><div className="container split-showcase"><div><span className="eyebrow">{t("home.realDesktop")}</span><h2>{t("home.clearWorkspace")}</h2><p className="lead-small">{t("home.clearWorkspaceCopy")}</p><ul className="check-list"><li><BadgeCheck />{t("home.canonical")}</li><li><FileLock2 />{t("home.authorization")}</li><li><HardDrive />{t("home.sqlite")}</li></ul><Link className="button button-secondary" to="/download">{t("home.release")}</Link></div><div className="image-frame"><img src="/images/product-inventory.png" alt={t("home.feature.inventoryTitle")} /></div></div></section>
    <section className="section container"><div className="three-proof"><article><SignalZero /><h3>{t("home.workLocally")}</h3><p>{t("home.workLocallyCopy")}</p></article><article><ArchiveRestore /><h3>{t("home.recover")}</h3><p>{t("home.recoverCopy")}</p></article><article><Laptop /><h3>{t("home.devices")}</h3><p>{t("home.devicesCopy")}</p></article></div></section>
    <CtaBand />
  </>;
}

export function FeaturesPage() {
  const { t } = useLocalization();
  return <><PageHero eyebrow={t("features.eyebrow")} title={t("features.title")} copy={t("features.copy")} /><section className="section container"><div className="feature-grid feature-grid-wide">{features.map(([Icon, title, copy]) => <article className="feature-card" key={title}><span className="icon-tile"><Icon /></span><h3>{t(title as MessageKey)}</h3><p>{t(copy as MessageKey)}</p></article>)}</div></section><section className="section dark-section"><div className="container roadmap-note"><div><span className="eyebrow">{t("features.syncEyebrow")}</span><h2>{t("features.syncTitle")}</h2><p>{t("features.syncCopy")}</p></div><Building2 size={92} /></div></section><CtaBand /></>;
}

export function PricingPage() {
  const { t } = useLocalization();
  return <><PageHero eyebrow={t("pricing.eyebrow")} title={t("pricing.title")} copy={t("pricing.copy")} /><section className="section container"><div className="pricing-grid">{plans.map(([name, price, copy, bullets, featured]) => <article className={`price-card ${featured ? "featured" : ""}`} key={name}>{featured && <span className="plan-ribbon">{t("pricing.startHere")}</span>}<h2>{t(name)}</h2><strong className="price">{t(price)}</strong><p>{t(copy)}</p><ul>{bullets.map((item) => <li key={item}><Check />{t(item)}</li>)}</ul><Link className={`button ${featured ? "button-primary" : "button-secondary"}`} to={featured ? "/register" : "/support"}>{featured ? t("home.createAccount") : t("pricing.requestActivation")}</Link></article>)}</div><p className="pricing-disclaimer"><CircleDollarSign /> {t("pricing.disclaimer")}</p></section></>;
}

export function DownloadPage() {
  const { date, number, t } = useLocalization();
  return <><PageHero eyebrow={t("download.eyebrow")} title={t("download.title")} copy={t("download.copy")} /><section className="section container"><div className="download-panel"><div className="release-icon"><Download /></div><div className="release-main"><div className="release-title"><h2>M² Health {currentRelease.version}</h2><span>{t("download.channel")}</span></div><dl><div><dt>{t("download.build")}</dt><dd>{currentRelease.buildId}</dd></div><div><dt>{t("download.platform")}</dt><dd>{t("download.windowsX64")}</dd></div><div><dt>{t("download.published")}</dt><dd>{date(currentRelease.publishedAt)}</dd></div><div><dt>{t("download.size")}</dt><dd>{t("download.megabytes", { size: number(currentRelease.sizeBytes / 1_000_000, { minimumFractionDigits: 1, maximumFractionDigits: 1 }) })}</dd></div></dl><a className="button button-primary button-large" href={currentRelease.downloadUrl} download><Download size={19} />{t("download.installer")}</a><a className="text-link" href={currentRelease.releaseNotesUrl}>{t("download.releaseNotes")}</a></div></div><div className="warning-card"><ShieldCheck /><div><h3>{t("download.unsignedTitle")}</h3><p>{t("download.unsignedCopy")}</p><code>{currentRelease.checksum}</code></div></div><div className="requirements-grid"><article><h3>{t("download.requirementsTitle")}</h3><ul><li>{t("download.requirementOs")}</li><li>{t("download.requirementWebview")}</li><li>{t("download.requirementStorage")}</li><li>{t("download.requirementInternet")}</li></ul></article><article><h3>{t("download.beforeTitle")}</h3><ul><li>{t("download.beforeBackup")}</li><li>{t("download.beforeOfficial")}</li><li>{t("download.beforeDevice")}</li><li>{t("download.beforeNotes")}</li></ul></article></div></section></>;
}

export function FaqPage() {
  const { t } = useLocalization();
  return <><PageHero eyebrow={t("faq.eyebrow")} title={t("faq.title")} copy={t("faq.copy")} /><section className="section container faq-list">{faqs.map(([question, answer]) => <details key={question}><summary>{t(question)}</summary><p>{t(answer)}</p></details>)}</section></>;
}

export function SupportPage() {
  const { t } = useLocalization();
  return <><PageHero eyebrow={t("support.eyebrow")} title={t("support.title")} copy={t("support.copy")} /><section className="section container support-grid"><article className="support-card"><ReceiptText /><h2>{t("support.accountTitle")}</h2><p>{t("support.accountCopy")}</p><Link className="button button-primary" to="/portal/subscription">{t("support.openPortal")}</Link></article><article className="support-card"><ArchiveRestore /><h2>{t("support.recoveryTitle")}</h2><p>{t("support.recoveryCopy")}</p><a className="button button-secondary" href={`mailto:${siteConfig.supportEmail}?subject=${encodeURIComponent(t("support.emailSubject"))}`}>{t("support.email")}</a></article></section></>;
}

function PageHero({ eyebrow, title, copy }: { eyebrow: string; title: string; copy: string }) {
  return <section className="page-hero"><div className="container"><span className="eyebrow">{eyebrow}</span><h1>{title}</h1><p>{copy}</p></div></section>;
}
