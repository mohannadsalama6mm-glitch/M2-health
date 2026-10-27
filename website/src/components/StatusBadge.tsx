import { useLocalization } from "../localization/LocalizationContext";

export function StatusBadge({ value }: { value: string }) {
  const { state } = useLocalization();
  const tone = ["ACTIVE", "TRIAL"].includes(value) ? "good" : value === "UNACTIVATED" ? "neutral" : "warn";
  return <span className={`status-badge status-${tone}`}>{state(value)}</span>;
}
