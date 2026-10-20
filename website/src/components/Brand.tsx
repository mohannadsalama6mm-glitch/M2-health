import { Activity } from "lucide-react";
import { Link } from "react-router-dom";

export function Brand() {
  return (
    <Link className="brand" to="/" aria-label="M² Health home">
      <span className="brand-mark"><Activity size={21} strokeWidth={2.5} /></span>
      <span>M² Health</span>
    </Link>
  );
}
