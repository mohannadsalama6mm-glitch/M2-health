// UI DEMO FIXTURES ONLY. No live data, persistence, APIs, or pharmacy business logic.
export const demoSales = [
  {
    invoice: "#1042",
    time: "10:24 AM",
    customer: "Ahmed Ali",
    items: 3,
    total: 245,
    payment: "Cash",
  },
  {
    invoice: "#1041",
    time: "10:18 AM",
    customer: "Sara Mohamed",
    items: 5,
    total: 780,
    payment: "Card",
  },
  {
    invoice: "#1040",
    time: "10:12 AM",
    customer: "Walk-in customer",
    items: 2,
    total: 112,
    payment: "Cash",
  },
  {
    invoice: "#1039",
    time: "10:06 AM",
    customer: "Omar Hassan",
    items: 4,
    total: 540,
    payment: "Card",
  },
  {
    invoice: "#1038",
    time: "09:58 AM",
    customer: "Layla Khaled",
    items: 1,
    total: 95,
    payment: "Cash",
  },
];
export const demoStock = [
  { name: "Paracetamol 500 mg", stock: 12, min: 50 },
  { name: "Amoxicillin 500 mg", stock: 8, min: 30 },
  { name: "Vitamin D3 1000 IU", stock: 5, min: 20 },
  { name: "Salbutamol inhaler", stock: 3, min: 10 },
  { name: "Omeprazole 20 mg", stock: 9, min: 30 },
];
export const demoProducts = [
  {
    name: "Panadol 500 mg",
    category: "Pain relief",
    units: 320,
    revenue: 3200,
  },
  {
    name: "Vitamin C 1000 mg",
    category: "Vitamins & supplements",
    units: 210,
    revenue: 2940,
  },
  { name: "Brufen 400 mg", category: "Pain relief", units: 180, revenue: 2160 },
  {
    name: "Cetirizine 10 mg",
    category: "Allergy care",
    units: 150,
    revenue: 1950,
  },
];
export const demoExpiry = [
  { name: "Cetirizine 10 mg", date: "18 Sep 2026", days: 7 },
  { name: "Vitamin C 1000 mg", date: "25 Sep 2026", days: 14 },
  { name: "Ibuprofen 400 mg", date: "03 Oct 2026", days: 22 },
];
export const demoChart = {
  week: [7, 12, 15, 11, 16, 13, 18.45],
  previous: [6, 9, 12, 10, 14, 11, 16.47],
};
export const demoInventory = [
  { name: "In stock", count: 412, color: "var(--color-primary)" },
  { name: "Low stock", count: 18, color: "var(--color-warning)" },
  { name: "Out of stock", count: 6, color: "var(--color-danger)" },
  { name: "Expiring soon", count: 12, color: "var(--color-chart-blue)" },
  { name: "Inactive", count: 0, color: "var(--color-chart-pale)" },
];
