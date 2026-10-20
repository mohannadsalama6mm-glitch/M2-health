export function StatusBadge({ value }: { value: string }) {
  const tone = ["ACTIVE", "TRIAL"].includes(value) ? "good" : value === "UNACTIVATED" ? "neutral" : "warn";
  return <span className={`status-badge status-${tone}`}>{value.replaceAll("_", " ")}</span>;
}
