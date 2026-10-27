import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach } from "vitest";
import { localeStorageKey, localizeSemanticState } from "../../../localization";
import { LocalizationProvider, useLocalization } from "../localization/LocalizationContext";
import {
  DownloadPage,
  FaqPage,
  FeaturesPage,
  HomePage,
  PricingPage,
  SupportPage,
} from "../pages/PublicPages";
import { StatusBadge } from "../components/StatusBadge";

function LanguageControl() {
  const { locale, setLocale, t } = useLocalization();
  return <label>{t("language.label")}<select aria-label={t("language.label")} value={locale} onChange={(event) => setLocale(event.target.value === "ar" ? "ar" : "en")}><option value="en">EN</option><option value="ar">AR</option></select></label>;
}

function LocalizedLanding() {
  return <LocalizationProvider><MemoryRouter><LanguageControl /><HomePage /></MemoryRouter></LocalizationProvider>;
}

describe("Arabic and English localization foundation", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.lang = "en";
    document.documentElement.dir = "ltr";
  });

  it("switches translated landing content and root language/direction", async () => {
    const user = userEvent.setup();
    render(<LocalizedLanding />);
    expect(screen.getByRole("heading", { name: /pharmacy stays moving/i })).toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Language"), "ar");
    expect(await screen.findByRole("heading", { name: "صيدليتك تواصل العمل حتى عند انقطاع الإنترنت." })).toBeInTheDocument();
    await waitFor(() => {
      expect(document.documentElement).toHaveAttribute("lang", "ar");
      expect(document.documentElement).toHaveAttribute("dir", "rtl");
    });
  });

  it("persists the selected language", async () => {
    const user = userEvent.setup();
    const first = render(<LocalizedLanding />);
    await user.selectOptions(screen.getByLabelText("Language"), "ar");
    await waitFor(() => expect(window.localStorage.getItem(localeStorageKey)).toBe("ar"));
    first.unmount();
    render(<LocalizedLanding />);
    expect(screen.getByLabelText("اللغة")).toHaveValue("ar");
  });

  it("localizes semantic license and device states without displaying raw enums", () => {
    expect(localizeSemanticState("en", "OFFLINE_GRACE")).toBe("Offline grace");
    expect(localizeSemanticState("ar", "REVOKED")).toBe("ملغى");
    window.localStorage.setItem(localeStorageKey, "ar");
    render(<LocalizationProvider><StatusBadge value="PENDING" /></LocalizationProvider>);
    expect(screen.getByText("قيد المراجعة")).toBeInTheDocument();
    expect(screen.queryByText("PENDING")).not.toBeInTheDocument();
  });

  it("uses truthful account wording before the trial starts in both languages", async () => {
    const user = userEvent.setup();
    render(<LocalizedLanding />);
    expect(screen.getAllByRole("link", { name: "Create Early Access account" }).length).toBeGreaterThan(0);
    expect(screen.queryByRole("link", { name: /start.*trial/i })).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Language"), "ar");
    expect(screen.getAllByRole("link", { name: "إنشاء حساب الوصول المبكر" }).length).toBeGreaterThan(0);
  });

  it("fully localizes secondary public Early Access pages in Arabic", () => {
    window.localStorage.setItem(localeStorageKey, "ar");
    const pages = [
      [<DownloadPage />, "تنزيل M² Health لنظام Windows.", "Download M² Health for Windows."],
      [<FeaturesPage />, "الأساس التشغيلي الذي تحتاجه الصيدلية.", "The operational core a pharmacy needs."],
      [<PricingPage />, "جرّب سير العمل الحقيقي قبل اختيار الخطة.", "Try the real workflow before choosing a plan."],
      [<FaqPage />, "وصول مبكر دون غموض التفاصيل الدقيقة.", "Early Access, without the fine-print fog."],
      [<SupportPage />, "أخبرنا بما تحتاج إليه.", "Tell us what you need."],
    ] as const;

    for (const [page, arabicHeading, englishHeading] of pages) {
      const view = render(<LocalizationProvider><MemoryRouter>{page}</MemoryRouter></LocalizationProvider>);
      expect(screen.getByRole("heading", { name: arabicHeading })).toBeInTheDocument();
      expect(screen.queryByRole("heading", { name: englishHeading })).not.toBeInTheDocument();
      expect(document.documentElement).toHaveAttribute("lang", "ar");
      expect(document.documentElement).toHaveAttribute("dir", "rtl");
      view.unmount();
    }
  });
});
