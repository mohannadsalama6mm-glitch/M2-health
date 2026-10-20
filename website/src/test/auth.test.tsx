import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, vi } from "vitest";
import { AuthPage } from "../pages/AuthPages";

const register = vi.fn();
const login = vi.fn();
vi.mock("../auth/AuthContext", () => ({
  useAuth: () => ({ user: null, configured: true, register, login }),
}));

describe("customer authentication views", () => {
  beforeEach(() => { register.mockReset(); login.mockReset(); });

  it("submits registration without starting a trial", async () => {
    register.mockResolvedValue({ confirmationRequired: true });
    render(<MemoryRouter><AuthPage mode="register" /></MemoryRouter>);
    fireEvent.change(screen.getByLabelText(/full name/i), { target: { value: "Synthetic Owner" } });
    fireEvent.change(screen.getByLabelText(/email/i), { target: { value: "owner@example.invalid" } });
    fireEvent.change(screen.getByLabelText(/^password/i), { target: { value: "safe-password" } });
    fireEvent.click(screen.getByRole("button", { name: /create account/i }));
    await waitFor(() => expect(register).toHaveBeenCalledWith("owner@example.invalid", "safe-password", "Synthetic Owner"));
    expect(screen.getByText(/does not start the seven-day trial/i)).toBeInTheDocument();
  });

  it("submits a login through Supabase Auth", async () => {
    login.mockResolvedValue(undefined);
    render(<MemoryRouter><AuthPage mode="login" /></MemoryRouter>);
    fireEvent.change(screen.getByLabelText(/email/i), { target: { value: "owner@example.invalid" } });
    fireEvent.change(screen.getByLabelText(/^password/i), { target: { value: "safe-password" } });
    fireEvent.click(screen.getByRole("button", { name: /sign in/i }));
    await waitFor(() => expect(login).toHaveBeenCalledWith("owner@example.invalid", "safe-password"));
  });
});
