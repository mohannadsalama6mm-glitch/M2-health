import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { DownloadPage, HomePage, PricingPage } from "../pages/PublicPages";

describe("public Early Access website", () => {
  it("renders the landing value proposition and truthful account CTA", () => {
    render(<MemoryRouter><HomePage /></MemoryRouter>);
    expect(screen.getByRole("heading", { name: /pharmacy stays moving/i })).toBeInTheDocument();
    expect(screen.getAllByRole("link", { name: /create early access account/i })[0]).toHaveAttribute("href", "/register");
    expect(screen.queryByRole("link", { name: /start.*trial/i })).not.toBeInTheDocument();
    expect(screen.getByText(/local pharmacy data/i)).toBeInTheDocument();
  });

  it("never presents the placeholder pricing flow as payment", () => {
    render(<MemoryRouter><PricingPage /></MemoryRouter>);
    expect(screen.getByText(/prices and recurring billing are not active/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /pay|checkout/i })).not.toBeInTheDocument();
    expect(screen.getAllByRole("link", { name: /request activation/i })[0]).toHaveAttribute("href", "/support");
  });

  it("renders immutable release metadata and unsigned warning", () => {
    render(<MemoryRouter><DownloadPage /></MemoryRouter>);
    expect(screen.getByText("20261022.1")).toBeInTheDocument();
    expect(screen.getByText(/unsigned early access build/i)).toBeInTheDocument();
    expect(screen.getByText(/3EF9F2190B64B760/)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /download installer/i })).toHaveAttribute("download");
  });
});
