export type SettingField = {
  label: string;
  type?: string;
  value?: string;
  options?: string[];
  disabled?: boolean;
};
export const settingFields: Record<string, SettingField[]> = {
  General: [
    { label: "Workspace name", value: "M² Health" },
    { label: "Time zone", options: ["Africa/Cairo", "UTC"] },
    { label: "Date format", options: ["DD MMM YYYY", "YYYY-MM-DD"] },
    { label: "Currency", value: "EGP", disabled: true },
  ],
  "Pharmacy Information": [
    { label: "Pharmacy name", value: "M² Health Pharmacy" },
    { label: "Address", value: "18 El Nozha Street, Cairo" },
    { label: "Phone", value: "02 0000 0101" },
    { label: "Registration number", value: "DEMO-001" },
  ],
  Appearance: [
    { label: "Theme", options: ["Light mint"] },
    { label: "Density", options: ["Comfortable", "Compact"] },
    { label: "Motion", options: ["Follow system", "Reduced"] },
    { label: "Accent", value: "Emerald", disabled: true },
  ],
  Language: [
    { label: "Interface language", options: ["English"] },
    {
      label: "Arabic support",
      value: "Font and logical layout prepared",
      disabled: true,
    },
    { label: "Number format", options: ["1,234.56", "1 234,56"] },
    {
      label: "Translation status",
      value: "Full Arabic translation is a future task",
      disabled: true,
    },
  ],
  Receipt: [
    { label: "Header", value: "M² Health Pharmacy" },
    { label: "Footer", value: "Thank you for choosing M² Health." },
    { label: "Paper width", options: ["80 mm", "58 mm", "A4"] },
    { label: "Copies", type: "number", value: "1" },
  ],
  Barcode: [
    { label: "Label format", options: ["38 × 25 mm", "50 × 30 mm"] },
    { label: "Barcode standard", options: ["EAN-13", "Code 128"] },
    { label: "Copies per product", type: "number", value: "1" },
    { label: "Printer", value: "Not connected", disabled: true },
  ],
  Inventory: [
    { label: "Default reorder level", type: "number", value: "10" },
    { label: "Expiry alert window (days)", type: "number", value: "30" },
    {
      label: "Negative stock policy",
      options: ["Block", "Require manager review"],
    },
    {
      label: "Batch selection",
      options: ["Earliest expiry first", "Manual selection"],
    },
  ],
  Sales: [
    { label: "Default payment", options: ["Cash", "Card"] },
    {
      label: "Discount approval",
      options: ["Manager approval", "Owner approval"],
    },
    { label: "Receipt after checkout", options: ["Preview", "Ask every time"] },
    { label: "Tax configuration", value: "Not configured", disabled: true },
  ],
  Users: [
    {
      label: "Default role",
      options: ["Cashier", "Pharmacist", "Inventory Staff"],
    },
    { label: "Invitation status", value: "No account backend", disabled: true },
  ],
  Security: [
    { label: "Idle lock (minutes)", type: "number", value: "15" },
    { label: "Sensitive action approval", options: ["Manager", "Owner"] },
    { label: "Authentication", value: "Not implemented", disabled: true },
    { label: "Session policy", value: "Preview only", disabled: true },
  ],
  Backup: [
    { label: "Backup schedule", options: ["Daily", "Weekly"] },
    { label: "Retention (days)", type: "number", value: "30" },
    { label: "Destination", value: "D:\\M2Health\\Backups" },
    { label: "Backup engine", value: "Not connected", disabled: true },
  ],
  Sync: [
    { label: "Sync preference", options: ["When connected", "Manual review"] },
    { label: "Conflict handling", options: ["Require review"] },
    { label: "Cloud endpoint", value: "Not configured", disabled: true },
    { label: "Device identity", value: "MAIN-POS-01 · demo", disabled: true },
  ],
  Hardware: [
    { label: "Receipt printer", options: ["Not connected"] },
    { label: "Barcode scanner", options: ["Keyboard input preview"] },
    { label: "Cash drawer", options: ["Not connected"] },
    { label: "Customer display", options: ["Not connected"] },
  ],
  Advanced: [
    { label: "Diagnostic level", options: ["Standard", "Verbose preview"] },
    {
      label: "Runtime",
      value: "Browser preview · Tauri deferred",
      disabled: true,
    },
    { label: "Local database", value: "Not implemented", disabled: true },
    { label: "Build", value: "Phase 1B · UI foundation", disabled: true },
  ],
};
