import { Activity } from "lucide-react";
import { Link } from "react-router-dom";
import { useLocalization } from "../localization/LocalizationContext";

export function Brand() {
  const { t } = useLocalization();
  return (
    <Link className="brand" to="/" aria-label={t("site.homeAria")}>
      <span className="brand-mark"><Activity size={21} strokeWidth={2.5} /></span>
      <span>M² Health</span>
    </Link>
  );
}
