import { fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { vi } from "vitest";
import { PortalLayout } from "../components/PortalLayout";
import { ProtectedRoute } from "../components/ProtectedRoute";

const logout = vi.fn();
let authenticated = false;
vi.mock("../auth/AuthContext", () => ({
  useAuth: () => ({ user: authenticated ? { email: "owner@example.invalid" } : null, loading: false, logout }),
}));

describe("protected portal session", () => {
  it("redirects anonymous visitors to login", () => {
    authenticated = false;
    render(<MemoryRouter initialEntries={["/portal"]}><Routes><Route path="/portal" element={<ProtectedRoute><div>Private portal</div></ProtectedRoute>} /><Route path="/login" element={<div>Login destination</div>} /></Routes></MemoryRouter>);
    expect(screen.getByText("Login destination")).toBeInTheDocument();
  });

  it("exposes a real logout action for authenticated customers", () => {
    authenticated = true;
    render(<MemoryRouter initialEntries={["/portal"]}><Routes><Route path="/portal" element={<PortalLayout />}><Route index element={<div>Private portal</div>} /></Route></Routes></MemoryRouter>);
    fireEvent.click(screen.getByRole("button", { name: /sign out/i }));
    expect(logout).toHaveBeenCalledOnce();
  });
});
