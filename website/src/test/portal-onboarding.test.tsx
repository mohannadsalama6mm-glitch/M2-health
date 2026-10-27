import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, vi } from "vitest";
import { localeStorageKey } from "../../../localization";
import { LocalizationProvider } from "../localization/LocalizationContext";
import { PortalOnboardingChecklist } from "../components/PortalOnboardingChecklist";
import { PortalLayout } from "../components/PortalLayout";
import styles from "../styles.css?raw";
import type { PortalSnapshot } from "../portal/types";

vi.mock("../auth/AuthContext", () => ({
  useAuth: () => ({ user: { email: "owner@example.invalid" }, logout: vi.fn() }),
}));

const snapshot: PortalSnapshot = {
  accountId: "account-1",
  fullName: "Synthetic Owner",
  email: "owner@example.invalid",
  planCode: "EARLY_ACCESS",
  status: "UNACTIVATED",
  trialStartedAt: null,
  trialEndsAt: null,
  licenseStartsAt: null,
  licenseEndsAt: null,
  daysRemaining: null,
  maxDevices: 1,
  devices: [],
  activationRequests: [],
};

describe("portal onboarding checklist", () => {
  beforeEach(() => window.localStorage.clear());

  it("shows state-aware recovery actions and completes from live portal state", () => {
    const view = render(<LocalizationProvider><MemoryRouter><PortalOnboardingChecklist snapshot={snapshot} /></MemoryRouter></LocalizationProvider>);
    expect(screen.getByText("1 of 4 steps complete")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Start trial" })).toHaveAttribute("href", "#portal-start-trial");
    expect(screen.getByText(/No active device is registered/)).toBeInTheDocument();
    view.rerender(<LocalizationProvider><MemoryRouter><PortalOnboardingChecklist snapshot={{ ...snapshot, status: "ACTIVE", devices: [{ id: "device-1", displayName: "Counter", status: "ACTIVE", activatedAt: "2026-10-21T00:00:00Z", lastValidatedAt: null, appVersion: null, buildId: null }] }} /></MemoryRouter></LocalizationProvider>);
    expect(screen.getByText("4 of 4 steps complete")).toBeInTheDocument();
    expect(screen.getByText("An active registered device confirms activation is available.")).toBeInTheDocument();
  });

  it("renders the checklist in Arabic RTL", () => {
    window.localStorage.setItem(localeStorageKey, "ar");
    render(<LocalizationProvider><MemoryRouter><PortalOnboardingChecklist snapshot={snapshot} /></MemoryRouter></LocalizationProvider>);
    expect(screen.getByRole("heading", { name: "إعداد الوصول المبكر" })).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "ar");
    expect(document.documentElement).toHaveAttribute("dir", "rtl");
  });

  it("keeps portal sign-out rendered and visible in the mobile stylesheet", () => {
    render(<LocalizationProvider><MemoryRouter><PortalLayout /></MemoryRouter></LocalizationProvider>);
    expect(screen.getByRole("button", { name: "Sign out" })).toHaveClass("sidebar-signout");
    expect(styles).toMatch(/@media \(max-width: 760px\)[\s\S]*?\.sidebar-signout \{ display:flex;/);
  });
});
